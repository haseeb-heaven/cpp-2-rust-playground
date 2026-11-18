use crate::config::ApplicationConfiguration;
use crate::executor::execute_program;
use crate::models::{
    ExecutionLanguage, ExecutionMode, ExecutionProvider, ExecutionRequest, PostProcessingSummary,
    SourceLanguage,
};
use crate::transpiler::{workspace_root_dir, TempEngineDirectory};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

static CPP2RUST_LINK_ARGS: Lazy<Vec<String>> = Lazy::new(|| {
    let build_dir = workspace_root_dir().join(".tools/cpp2rust/build");
    let mut crate_map: HashMap<String, PathBuf> = HashMap::new();
    let mut dep_dirs: BTreeSet<PathBuf> = BTreeSet::new();

    for target_name in ["libc-dep-target", "libcc2rs-target"] {
        let artifacts_path = build_dir.join(target_name).join("artifacts.json");
        let Ok(contents) = fs::read_to_string(&artifacts_path) else {
            continue;
        };
        for line in contents.lines() {
            let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            if val.get("reason").and_then(|r| r.as_str()) != Some("compiler-artifact") {
                continue;
            }
            let Some(name) = val
                .get("target")
                .and_then(|t| t.get("name"))
                .and_then(|n| n.as_str())
            else {
                continue;
            };
            let Some(filenames) = val.get("filenames").and_then(|f| f.as_array()) else {
                continue;
            };
            for file_val in filenames {
                let Some(file_str) = file_val.as_str() else {
                    continue;
                };
                let path = PathBuf::from(file_str);
                if matches!(
                    path.extension().and_then(|e| e.to_str()),
                    Some("rlib" | "dylib" | "so")
                ) {
                    if let Some(parent) = path.parent() {
                        dep_dirs.insert(parent.to_path_buf());
                    }
                    crate_map.insert(name.to_string(), path);
                }
            }
        }
    }

    let mut args = Vec::new();
    for dir in dep_dirs {
        args.push("-L".to_string());
        args.push(format!("dependency={}", dir.display()));
    }
    for crate_name in ["libcc2rs", "libc", "nix", "jiff"] {
        if let Some(lib_path) = crate_map.get(crate_name) {
            args.push("--extern".to_string());
            args.push(format!("{}={}", crate_name, lib_path.display()));
        }
    }
    args
});

pub fn rustc_extra_args_for_code(rust_code: &str) -> Vec<String> {
    if rust_code.contains("libcc2rs") || rust_code.contains("extern crate libc") {
        let mut args = vec![
            "--edition".to_string(),
            "2024".to_string(),
            "-A".to_string(),
            "warnings".to_string(),
        ];
        args.extend(CPP2RUST_LINK_ARGS.iter().cloned());
        args
    } else if rust_code.contains("extern \"C\"")
        || rust_code.contains("::core::ffi")
        || rust_code.contains("pub struct Arena")
        || rust_code.contains("#![allow(")
    {
        vec![
            "--edition".to_string(),
            "2021".to_string(),
            "-A".to_string(),
            "warnings".to_string(),
            "-W".to_string(),
            "unused-mut".to_string(),
        ]
    } else {
        vec!["--edition".to_string(), "2021".to_string()]
    }
}

static LIBC_TYPE_MAP: &[(&str, &str)] = &[
    ("libc::c_int", "i32"),
    ("libc::c_uint", "u32"),
    ("libc::c_long", "i64"),
    ("libc::c_ulong", "usize"),
    ("libc::size_t", "usize"),
    ("libc::c_double", "f64"),
    ("libc::c_float", "f32"),
    ("libc::c_char", "u8"),
];

static BOILERPLATE_EXACT_LINES: &[&str] = &[
    "// using namespace std;",
    "// return 0;",
    "// #define VectorCopy",
    "// #define VectorMA (expanded inline)",
    "// null check omitted in safe Rust",
    "// list push handled below",
    "// list traversal start",
    "// handled by Rust iterator",
    "// memory freed automatically by Rust",
    "// linked into global cvar list",
];

static USIZE_VAR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(?:let\s+(?:mut\s+)?|fn\s+\w+\([^)]*\b)([A-Za-z_][A-Za-z0-9_]*)\s*:\s*usize\b").unwrap());
static USIZE_PARAM_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*usize\b").unwrap());
static LET_MUT_DECL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\blet\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\b").unwrap());

/// Fixed-width Rust types that replace the C ABI types c2rust emits.
static C_ABI_TYPE_MAP: &[(&str, &str)] = &[
    ("::core::ffi::c_uint", "u32"),
    ("::core::ffi::c_int", "i32"),
    ("::core::ffi::c_ushort", "u16"),
    ("::core::ffi::c_short", "i16"),
    ("::core::ffi::c_ulonglong", "u64"),
    ("::core::ffi::c_longlong", "i64"),
    ("::core::ffi::c_ulong", "u64"),
    ("::core::ffi::c_long", "i64"),
    ("::core::ffi::c_uchar", "u8"),
    ("::core::ffi::c_schar", "i8"),
    ("::core::ffi::c_char", "i8"),
    ("::core::ffi::c_float", "f32"),
    ("::core::ffi::c_double", "f64"),
    ("::core::ffi::c_size_t", "usize"),
    ("::core::ffi::c_ssize_t", "isize"),
    ("::core::ffi::c_bool", "bool"),
];

/// Generated runtime plumbing of the vendored cpp2rust engine. This is never Rust
/// a reader wrote, so it can never be presented as Idiomatic output.
fn contains_engine_runtime_scaffolding(code: &str) -> bool {
    code.contains("libcc2rs")
        || code.contains("Ptr::<")
        || code.contains("Value<Vec<")
        || code.contains("__cpp2rust_init_globals")
}

/// Idiomatic output is plain safe Rust: no `unsafe`, no `extern "C"`, no C ABI
/// types, no libc/engine calls. Anything that still needs them is a raw
/// transcription and belongs in Generated Rust only.
fn contains_unsafe_or_c_interop(code: &str) -> bool {
    static C_INTEROP_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?m)\bunsafe\b|\bextern\s*\"C\"|#\[no_mangle\]|::core::ffi|\bno_mangle\b|\bfrom_raw\b|\btransmute\b"#,
        )
        .unwrap()
    });
    C_INTEROP_REGEX.is_match(code)
}

/// Text that is still a literal transcription of the C source: C ABI types the
/// normalizer failed to rewrite, or engine plumbing. Accepting it as Idiomatic
/// would claim a human-readable translation that was never produced.
fn still_raw_engine_transcription(faithful_rust: &str, idiomatic_candidate: &str) -> bool {
    if contains_engine_runtime_scaffolding(idiomatic_candidate)
        || contains_unsafe_or_c_interop(idiomatic_candidate)
    {
        return true;
    }
    idiomatic_candidate == faithful_rust
        && (faithful_rust.contains("::core::ffi")
            || faithful_rust.contains("extern crate libcc2rs"))
}

fn rewrite_c2rust_entrypoint(code: &str) -> Option<String> {
    static ENTRYPOINT_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?s)pub\s+fn\s+main\s*\(\s*\)\s*\{\s*unsafe\s*\{\s*::std::process::exit\(\s*main_0\(\)(?:\s*as\s+i32)?\s*\)\s*\}\s*\}"#,
        )
        .unwrap()
    });
    if !ENTRYPOINT_REGEX.is_match(code) {
        return None;
    }
    Some(
        ENTRYPOINT_REGEX
            .replace(
                code,
                "fn main() {\n    let exit_code = unsafe { main_0() };\n    std::process::exit(exit_code);\n}",
            )
            .to_string(),
    )
}

fn find_matching_delimiter(text: &str, open_index: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string: Option<char> = None;
    let mut escaped = false;
    for (offset, ch) in text[open_index..].char_indices() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                in_string = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => in_string = Some(ch),
            _ if ch == open => depth += 1,
            _ if ch == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_index + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level_arguments(text: &str) -> Vec<&str> {
    let mut arguments = Vec::new();
    let mut depth = 0usize;
    let mut in_string: Option<char> = None;
    let mut escaped = false;
    let mut start = 0usize;
    for (offset, ch) in text.char_indices() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                in_string = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => in_string = Some(ch),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                arguments.push(text[start..offset].trim());
                start = offset + 1;
            }
            _ => {}
        }
    }
    arguments.push(text[start..].trim());
    arguments
}

/// Translates a C `printf` format body into a Rust format body.
/// Returns `None` for conversions that cannot be preserved by a plain Rust
/// format argument (`%s` pointer operands, `%p`, `%c`, `*` width, ...).
fn c_printf_format_to_rust(format_body: &str) -> Option<(String, bool)> {
    use std::fmt::Write as _;

    let without_nul = format_body.strip_suffix("\\0")?;
    let (body, println) = match without_nul.strip_suffix("\\n") {
        Some(rest) => (rest, true),
        None => (without_nul, false),
    };

    let mut output = String::with_capacity(body.len());
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            output.push(ch);
            if let Some(next) = chars.next() {
                output.push(next);
            }
            continue;
        }
        if ch != '%' {
            match ch {
                '{' => output.push_str("{{"),
                '}' => output.push_str("}}"),
                _ => output.push(ch),
            }
            continue;
        }

        let mut flags = String::new();
        while matches!(chars.peek(), Some('-') | Some('+') | Some(' ') | Some('0') | Some('#')) {
            flags.push(chars.next().unwrap());
        }
        let mut width = String::new();
        while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
            width.push(chars.next().unwrap());
        }
        let mut precision: Option<String> = None;
        if chars.peek() == Some(&'.') {
            chars.next();
            let mut digits = String::new();
            while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
                digits.push(chars.next().unwrap());
            }
            precision = Some(digits);
        }
        while matches!(chars.peek(), Some('h') | Some('l') | Some('z') | Some('j') | Some('t') | Some('L')) {
            chars.next();
        }
        let conversion = chars.next()?;

        if conversion == '%' {
            output.push('%');
            continue;
        }
        if !flags.is_empty() || !width.is_empty() {
            return None;
        }
        match conversion {
            'd' | 'i' | 'u' => output.push_str("{}"),
            'f' | 'F' | 'e' | 'E' | 'g' | 'G' => match precision.as_deref() {
                Some(digits) if !digits.is_empty() => {
                    let _ = write!(output, "{{:.{digits}}}");
                }
                // C prints six fractional digits by default.
                _ => output.push_str("{:.6}"),
            },
            _ => return None,
        }
    }
    Some((output, println))
}

/// Rewrites `printf` statements into `print!`/`println!` when the call keeps the
/// same output. Unsupported calls are left untouched for the C ABI path.
fn lower_printf_calls(code: &str) -> (String, bool) {
    static OTHER_C_STDIO_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\b(?:puts|putchar|fputs|fwrite|fprintf|scanf|fscanf|fgets|fflush|perror)\s*\(")
            .unwrap()
    });
    // Rust's stdout buffer and the C stdio buffer are independent, so lowering only
    // some of the writes would reorder the program's output.
    if OTHER_C_STDIO_REGEX.is_match(code) {
        return (code.to_string(), false);
    }

    static PRINTF_FORMAT_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?s)^b"((?:[^"\\]|\\.)*)"\s*(?:as\s+\*const\s+u8\s+as\s+\*const\s+(?:::core::ffi::)?(?:c_char|i8))?\s*$"#,
        )
        .unwrap()
    });

    let mut result = String::with_capacity(code.len());
    let mut rest = code;
    let mut changed = false;

    while let Some(position) = rest.find("printf") {
        let is_boundary = position == 0
            || !rest[..position]
                .chars()
                .last()
                .map(|ch| ch.is_alphanumeric() || ch == '_')
                .unwrap_or(false);
        if !is_boundary {
            result.push_str(&rest[..position + "printf".len()]);
            rest = &rest[position + "printf".len()..];
            continue;
        }

        result.push_str(&rest[..position]);
        let call = &rest[position..];
        let Some(open_offset) = call.find('(') else {
            result.push_str(call);
            rest = "";
            break;
        };
        if call[..open_offset].trim() != "printf" {
            result.push_str(&call[..open_offset + 1]);
            rest = &call[open_offset + 1..];
            continue;
        }
        let Some(close_offset) = find_matching_delimiter(call, open_offset, '(', ')') else {
            result.push_str(call);
            rest = "";
            break;
        };
        let trailing = &call[close_offset + 1..];
        let statement_end = trailing.find(';');
        let Some(statement_end) = statement_end else {
            result.push_str(&call[..close_offset + 1]);
            rest = &call[close_offset + 1..];
            continue;
        };
        if !trailing[..statement_end].trim().is_empty() {
            result.push_str(&call[..close_offset + 1]);
            rest = &call[close_offset + 1..];
            continue;
        }

        let arguments = split_top_level_arguments(&call[open_offset + 1..close_offset]);
        let lowerable = arguments
            .first()
            .and_then(|format| PRINTF_FORMAT_REGEX.captures(format))
            .and_then(|captures| c_printf_format_to_rust(&captures[1]));

        let Some((format, println)) = lowerable else {
            result.push_str(&call[..close_offset + 1]);
            rest = &call[close_offset + 1..];
            continue;
        };

        let macro_name = if println { "println!" } else { "print!" };
        let placeholder_count = format.matches('{').count();
        let operands: Vec<&str> = arguments[1..]
            .iter()
            .copied()
            .filter(|argument| !argument.is_empty())
            .take(placeholder_count)
            .collect();
        if operands.is_empty() {
            result.push_str(&format!("{macro_name}(\"{format}\")"));
        } else {
            result.push_str(&format!(
                "{macro_name}(\"{format}\", {})",
                operands.join(", ")
            ));
        }
        result.push_str(&trailing[statement_end..=statement_end]);
        changed = true;
        rest = &trailing[statement_end + 1..];
    }
    result.push_str(rest);
    (result, changed)
}

/// Splits an `extern "C"` body into items that end at a top-level `;`.
fn split_ffi_declaration_items(body: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (offset, ch) in body.char_indices() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ';' if depth == 0 => {
                items.push(&body[start..=offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    if !body[start..].trim().is_empty() {
        items.push(&body[start..]);
    }
    items
}

/// Drops generated C-header type aliases and constants that nothing references
/// any more (for example `__DARWIN_NULL`, which carries `c_void` but is dead once
/// the pointer it described became an owned `Vec`).
fn prune_unused_c_type_declarations(code: &str) -> (String, bool) {
    static DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^\s*pub\s+(?:const|type)\s+([A-Za-z_][A-Za-z0-9_]*)\s*[:=][^;]*;\s*$").unwrap()
    });

    let mut changed = false;
    let mut updated = code.to_string();
    loop {
        let snapshot = updated.clone();
        let analysis = code_without_literals(&snapshot);
        let mut removed_any = false;
        let mut result = String::with_capacity(snapshot.len());
        let mut cursor = 0usize;
        for captures in DECL_REGEX.captures_iter(&snapshot) {
            let whole = captures.get(0).unwrap();
            let name = &captures[1];
            let mentions = Regex::new(&format!(r"\b{}\b", regex::escape(name)))
                .map(|pattern| pattern.find_iter(&analysis).count())
                .unwrap_or(usize::MAX);
            if mentions > 1 {
                continue; // declaration + at least one real use
            }
            result.push_str(&snapshot[cursor..whole.start()]);
            cursor = whole.end();
            removed_any = true;
        }
        result.push_str(&snapshot[cursor..]);
        if !removed_any {
            break;
        }
        updated = result;
        changed = true;
    }

    (updated, changed)
}

/// Drops `extern "C"` declarations that nothing calls any more, and the whole
/// block once it is empty.
fn prune_unused_c_ffi_blocks(code: &str) -> (String, bool) {
    static DECLARED_FN_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap());
    static OTHER_ITEM_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^\s*(?:pub\s+)?(?:static|type|const|use|struct|enum)\b").unwrap());

    let mut result = String::with_capacity(code.len());
    let mut remaining = code;
    let mut changed = false;

    while let Some(position) = remaining.find("extern \"C\" {") {
        result.push_str(&remaining[..position]);
        let block = &remaining[position..];
        let open_offset = block.find('{').unwrap();
        let Some(close_offset) = find_matching_delimiter(block, open_offset, '{', '}') else {
            result.push_str(block);
            return (result, changed);
        };
        let body = &block[open_offset + 1..close_offset];
        let outer = format!("{}{}", result, &remaining[position + close_offset + 1..]);

        let mut kept_items = String::new();
        let mut dropped_any = false;
        for item in split_ffi_declaration_items(body) {
            let declared = DECLARED_FN_REGEX.captures(item).map(|c| c[1].to_string());
            if OTHER_ITEM_REGEX.is_match(item) {
                kept_items.push_str(item);
                continue;
            }
            let Some(name) = declared else {
                kept_items.push_str(item);
                continue;
            };
            let used = Regex::new(&format!(r"\b{}\b", regex::escape(&name)))
                .map(|pattern| pattern.is_match(&outer))
                .unwrap_or(true);
            if used {
                kept_items.push_str(item);
            } else {
                dropped_any = true;
            }
        }

        let block_empty = kept_items.trim().is_empty();
        if block_empty || dropped_any {
            changed = true;
            if !block_empty {
                result.push_str("extern \"C\" {");
                result.push_str(&kept_items);
                result.push('}');
            }
        } else {
            result.push_str(&block[..close_offset + 1]);
        }
        remaining = &remaining[position + close_offset + 1..];
    }
    result.push_str(remaining);
    (result, changed)
}

/// Masks string/char literals and comments so identifier analysis is not fooled
/// by text that merely mentions a name.
fn code_without_literals(code: &str) -> String {
    static LITERAL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"(?s)/\*.*?\*/|//[^\n]*|"(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'"#).unwrap()
    });
    LITERAL_REGEX.replace_all(code, " ").to_string()
}

/// Turns `if true { .. }` into a plain block, but only when there is no `else`
/// branch (dropping the condition of an `if/else` would change the program).
fn collapse_true_blocks(code: &str) -> (String, bool) {
    static TRUE_IF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bif\s+true\s*\{").unwrap());
    let mut updated = code.to_string();
    let mut changed = false;

    loop {
        let snapshot = updated.clone();
        let mut target: Option<(usize, usize)> = None;
        for found in TRUE_IF_REGEX.find_iter(&snapshot) {
            let brace = found.end() - 1;
            let Some(close) = find_matching_delimiter(&snapshot, brace, '{', '}') else {
                continue;
            };
            let rest = snapshot[close + 1..].trim_start();
            if rest.starts_with("else") {
                continue;
            }
            target = Some((found.start(), close + 1));
            break;
        }
        let Some((start, end)) = target else {
            break;
        };
        let inner = snapshot[start..end]
            .trim_start_matches("if")
            .trim_start()
            .trim_start_matches("true")
            .trim_start()
            .to_string();
        updated = format!("{}{}{}", &snapshot[..start], inner, &snapshot[end..]);
        changed = true;
    }

    (updated, changed)
}

/// Lowers `puts`/`fputs(stdout, ...)` on a known byte-string literal.
fn lower_puts_literals(code: &str) -> (String, bool) {
    static PUTS_LITERAL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?s)\bputs\s*\(\s*b"((?:[^"\\]|\\.)*)"\s*(?:as\s+\*const\s+u8\s+as\s+\*const\s+(?:::core::ffi::)?(?:c_char|i8))?\s*\)"#,
        )
        .unwrap()
    });

    let mut changed = false;
    let updated = PUTS_LITERAL_REGEX
        .replace_all(code, |captures: &regex::Captures| {
            let body = captures[1]
                .strip_suffix("\\0")
                .unwrap_or(&captures[1])
                .to_string();
            changed = true;
            // `puts` always terminates the line, so a trailing `\n` just becomes
            // `println!` without doubling the newline.
            match body.strip_suffix("\\n") {
                Some(rest) => format!("println!(\"{rest}\")"),
                None => format!("println!(\"{body}\")"),
            }
        })
        .to_string();

    (updated, changed)
}

/// Lowers the classic C line-input/output trio on a fixed `char` buffer while
/// keeping the C buffer-size semantics:
///
/// ```text
/// char data[10]; gets(data); puts(data);
/// ```
///
/// becomes a fixed `[u8; 10]` buffer read with `stdin().lock().take(len).read(..)`
/// and printed through `String::from_utf8_lossy`. The rewrite applies only when
/// the buffer is used exclusively by those calls, so no other code can observe
/// the raw bytes; otherwise the C-style code is left untouched.
fn lower_char_buffer_stdio(code: &str) -> (String, bool) {
    static BUFFER_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?m)^(\s*)let\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*\[(?:i8|u8);\s*(\d+)\]\s*=\s*\[0(?:i8|u8)?;\s*\d+\];",
        )
        .unwrap()
    });

    let mut updated = code.to_string();
    let mut changed = false;

    loop {
        let mut applied_this_round = false;
        for captures in BUFFER_DECL_REGEX.captures_iter(&updated.clone()) {
            let whole = captures.get(0).unwrap();
            let indent = captures[1].to_string();
            let name = captures[2].to_string();
            let length = captures[3].to_string();

            let escaped = regex::escape(&name);
            let read_regex = Regex::new(&format!(
                r"(?:gets\s*\(\s*|fgets\s*\(\s*)&raw\s+mut\s+{escaped}\s+as\s*\*mut\s*(?:i8|u8)(?:\s*,\s*\d+\s*,\s*[A-Za-z_][A-Za-z0-9_]*)?\s*\)"
            ))
            .unwrap();
            let write_regex = Regex::new(&format!(
                r"puts\s*\(\s*&raw\s+(?:mut|const)\s+{escaped}\s+as\s*\*(?:mut|const)\s*(?:i8|u8)\s*\)"
            ))
            .unwrap();

            if !read_regex.is_match(&updated) {
                continue;
            }

            // Every use of the buffer must be one of the lowered calls.
            let all_uses = Regex::new(&format!(r"\b{escaped}\b"))
                .unwrap()
                .find_iter(&updated)
                .count();
            let handled_uses = read_regex.find_iter(&updated).count()
                + write_regex.find_iter(&updated).count();
            if all_uses != handled_uses + 1 {
                // one occurrence is the declaration itself
                continue;
            }

            let counter = format!("__read_{name}");
            let mut rewritten = sentence_replace(
                &updated,
                whole.as_str(),
                &format!(
                    "{indent}let mut {name}: [u8; {length}] = [0u8; {length}];\n{indent}let mut {counter}: usize = 0;"
                ),
            );
            rewritten = read_regex
                .replace_all(
                    &rewritten,
                    format!(
                        "{counter} = {{\n        use std::io::Read as _;\n        std::io::stdin()\n            .lock()\n            .take({name}.len() as u64)\n            .read(&mut {name})\n            .expect(\"read stdin\")\n    }}"
                    )
                    .as_str(),
                )
                .to_string();
            rewritten = write_regex
                .replace_all(
                    &rewritten,
                    format!(
                        "println!(\"{{}}\", String::from_utf8_lossy(&{name}[..{counter}]).trim_end())"
                    )
                    .as_str(),
                )
                .to_string();

            updated = rewritten;
            changed = true;
            applied_this_round = true;
            break;
        }
        if !applied_this_round {
            break;
        }
    }

    (updated, changed)
}

fn sentence_replace(haystack: &str, needle: &str, replacement: &str) -> String {
    match haystack.find(needle) {
        Some(position) => {
            let mut output = String::with_capacity(haystack.len());
            output.push_str(&haystack[..position]);
            output.push_str(replacement);
            output.push_str(&haystack[position + needle.len()..]);
            output
        }
        None => haystack.to_string(),
    }
}

/// Lowers a single C heap allocation that behaves like an array into a `Vec`.
///
/// c2rust turns `int *arr = malloc(sizeof(int) * n);` into a raw pointer with
/// `offset`/deref arithmetic and a `free`. When every use of that pointer is the
/// allocation, indexed access, a null check, and `free`, the owned `Vec` is an
/// exact replacement and the result is plain safe Rust. Anything else (pointer
/// passed to a function, `realloc`, aliasing) keeps the C-shaped code for the
/// Faithful view.
fn lower_malloc_arrays_to_vec(code: &str) -> (String, bool) {
    static MALLOC_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?m)^(\s*)let\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*\*mut\s*(i8|i16|i32|i64|u8|u16|u32|u64|f32|f64|usize|isize|bool)\s*=\s*([^;]*);",
        )
        .unwrap()
    });
    static COUNT_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?s)size_of::<[A-Za-z0-9_:]+>\s*\(\s*\)\s*as\s+size_t\s*\)\s*\.\s*wrapping_mul\s*\(\s*(\d+)[a-z0-9]*\s+as\s+size_t\s*\)|\(\s*(\d+)[a-z0-9]*\s+as\s+size_t\s*\)\s*\.\s*wrapping_mul\s*\(\s*size_of::<[A-Za-z0-9_:]+>",
        )
        .unwrap()
    });

    let mut updated = code.to_string();
    let mut changed = false;

    loop {
        let mut applied_this_round = false;
        for captures in MALLOC_DECL_REGEX.captures_iter(&updated.clone()) {
            let whole = captures.get(0).unwrap();
            let indent = captures[1].to_string();
            let name = captures[2].to_string();
            let element_type = captures[3].to_string();
            let initializer = captures[4].to_string();
            let escaped = regex::escape(&name);

            if !initializer.contains("malloc") && !initializer.contains("calloc") {
                continue;
            }
            // `malloc(sizeof(int))` allocates exactly one element.
            let count = COUNT_REGEX
                .captures(&initializer)
                .and_then(|c| c.get(1).or_else(|| c.get(2)))
                .map(|m| m.as_str().to_string())
                .unwrap_or_else(|| "1".to_string());

            let offset_regex = Regex::new(&format!(
                r"\*\s*{escaped}\s*\.\s*offset\s*\(\s*(.+?)\s+as\s+isize\s*\)"
            ))
            .unwrap();
            let deref_assign_regex =
                Regex::new(&format!(r"\*\s*{escaped}\s*=")).unwrap();
            let deref_read_regex = Regex::new(&format!(r"\*\s*{escaped}\b")).unwrap();
            let null_check_regex =
                Regex::new(&format!(r"!\s*{escaped}\s*\.\s*is_null\s*\(\s*\)")).unwrap();
            let is_null_regex =
                Regex::new(&format!(r"{escaped}\s*\.\s*is_null\s*\(\s*\)")).unwrap();
            let free_regex = Regex::new(&format!(
                r"(?m)^[ \t]*free\s*\(\s*{escaped}\s*(?:as\s*\*mut\s*(?:[A-Za-z0-9_:]+))?\s*\)\s*;[ \t]*\n?"
            ))
            .unwrap();

            let mut rewritten = sentence_replace(
                &updated,
                whole.as_str(),
                &format!(
                    "{indent}let mut {name}: Vec<{element_type}> = vec![Default::default(); {count}];"
                ),
            );
            rewritten = offset_regex
                .replace_all(&rewritten, format!("{name}[($1) as usize]").as_str())
                .to_string();
            rewritten = deref_assign_regex
                .replace_all(&rewritten, format!("{name}[0] =").as_str())
                .to_string();
            rewritten = deref_read_regex
                .replace_all(&rewritten, format!("{name}[0]").as_str())
                .to_string();
            // `free` and the null check are meaningless for an owned Vec.
            rewritten = free_regex.replace_all(&rewritten, "").to_string();
            rewritten = null_check_regex.replace_all(&rewritten, "true").to_string();
            rewritten = is_null_regex.replace_all(&rewritten, "false").to_string();

            // Every remaining mention must be the declaration or an index access.
            let analysis = code_without_literals(&rewritten);
            let total_mentions = Regex::new(&format!(r"\b{escaped}\b"))
                .unwrap()
                .find_iter(&analysis)
                .count();
            let indexed_mentions = Regex::new(&format!(r"\b{escaped}\s*\["))
                .unwrap()
                .find_iter(&analysis)
                .count();
            if total_mentions != indexed_mentions + 1 {
                continue;
            }

            updated = rewritten;
            changed = true;
            applied_this_round = true;
            break;
        }
        if !applied_this_round {
            break;
        }
    }

    (updated, changed)
}

/// Lowers the cpp2rust class translation shape to plain Rust.
///
/// cpp2rust models C++ objects with its `libcc2rs` runtime: layout derives,
/// `Value<T>` handles (`Rc<RefCell<T>>`), a `Trait for Ptr<T>` dispatch layer,
/// `field!(..)` accessors and `write!(libcc2rs::cout(), ..)`. For a standalone
/// program none of that is observable, so it can be replaced by an owned struct
/// with inherent methods, `self.field` access and `print!`/`println!`. Every
/// rewrite is language-wide; the caller verifies the result with rustc.
fn normalize_cpp2rust_class_translation(code: &str) -> (String, bool) {
    if !code.contains("libcc2rs") {
        return (code.to_string(), false);
    }

    static RUNTIME_IMPORT_LINES: &[&str] = &[
        "extern crate libcc2rs;",
        "use libcc2rs::*;",
        "use std::cell::RefCell;",
        "use std::collections::BTreeMap;",
        "use std::io::prelude::*;",
        "use std::io::{Read, Seek, Write};",
        "use std::os::fd::AsFd;",
        "use std::rc::{Rc, Weak};",
    ];
    static LAYOUT_DERIVE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^[ \t]*#\[derive\(([^\)]*\b(?:Record|ByteRepr|VaArg|FnPtrArg|CppFrom)\b[^\)]*)\)\][ \t]*\n?")
            .unwrap()
    });
    static CPP2RUST_DERIVE_NAMES: &[&str] = &["Record", "ByteRepr", "VaArg", "FnPtrArg", "CppFrom"];
    static LAYOUT_ATTR_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^[ \t]*#\[(?:cpp_from|byte_size|offset)\([^\)]*\)\][ \t]*\n?").unwrap()
    });
    static VALUE_LET_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^([ \t]*)let\s+(\w+)\s*:\s*Value<(\w+)>\s*=\s*Rc::new\(RefCell::new\((.*)\)\);\s*$")
            .unwrap()
    });
    static TRAIT_DECL_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"pub\s+trait\s+(\w+)Impl\s*\{").unwrap());
    static TRAIT_IMPL_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"impl\s+(\w+)Impl\s+for\s+Ptr<(\w+)>\s*\{").unwrap());

    let mut updated = code.to_string();
    let mut changed = false;

    // Runtime plumbing that a standalone program never observes.
    let without_imports: Vec<&str> = updated
        .lines()
        .filter(|line| !RUNTIME_IMPORT_LINES.contains(&line.trim()))
        .collect();
    let joined = without_imports.join("\n");
    if joined != updated {
        updated = joined;
        changed = true;
    }
    // Keep the standard derives (Clone/Copy/Default/...) that the generated code
    // relies on and drop only the cpp2rust layout derives.
    let stripped = LAYOUT_DERIVE_REGEX
        .replace_all(&updated, |captures: &regex::Captures| {
            let kept: Vec<&str> = captures[1]
                .split(',')
                .map(|entry| entry.trim())
                .filter(|entry| !entry.is_empty() && !CPP2RUST_DERIVE_NAMES.contains(entry))
                .collect();
            if kept.is_empty() {
                String::new()
            } else {
                format!("#[derive({})]\n", kept.join(", "))
            }
        })
        .to_string();
    if stripped != updated {
        updated = stripped;
        changed = true;
    }
    let stripped = LAYOUT_ATTR_REGEX.replace_all(&updated, "").to_string();
    if stripped != updated {
        updated = stripped;
        changed = true;
    }

    // `Value<T> = Rc::new(RefCell::new(x))` is just an owned value here.
    let owned = VALUE_LET_REGEX
        .replace_all(&updated, |captures: &regex::Captures| {
            format!("{}let mut {} = {};", &captures[1], &captures[2], &captures[4])
        })
        .to_string();
    if owned != updated {
        updated = owned;
        changed = true;
    }

    // Trait dispatch through `Ptr<T>` becomes inherent methods on the struct.
    // Every dispatch implementation is lowered, not just the first one.
    loop {
        let trait_impl = TRAIT_IMPL_REGEX
            .captures(&updated)
            .and_then(|captures| {
                let whole = captures.get(0)?;
                let type_name = captures.get(2)?.as_str().to_string();
                let brace = whole.end() - 1;
                let close = find_matching_delimiter(&updated, brace, '{', '}')?;
                Some((whole.start(), brace, close, type_name))
            });
        let Some((start, brace, close, type_name)) = trait_impl else {
            break;
        };
        let body = updated[brace + 1..close].to_string();
        let rewritten = mark_mutating_methods(&rewrite_cpp2rust_method_body(&body));
        updated = format!(
            "{}impl {} {{{}{}",
            &updated[..start],
            type_name,
            rewritten,
            &updated[close..]
        );
        changed = true;
    }

    // The dispatch traits only existed to carry that layer.
    loop {
        let Some(found) = TRAIT_DECL_REGEX.find(&updated) else {
            break;
        };
        let brace = found.end() - 1;
        let Some(close) = find_matching_delimiter(&updated, brace, '{', '}') else {
            break;
        };
        let mut end = close + 1;
        while updated[end..].starts_with('\n') {
            end += 1;
        }
        updated = format!("{}{}", &updated[..found.start()], &updated[end..]);
        changed = true;
    }

    // `PersonImpl::method(&obj.as_pointer(), args)` is `obj.method(args)`.
    // Nested dispatches are rewritten innermost-last, so repeat until stable,
    // and never touch program text inside literals or comments.
    static IMPL_CALL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\b\w+Impl::(\w+)\(\s*&\s*([A-Za-z_]\w*)\.as_pointer\(\)\s*,?\s*((?:[^()]*\([^()]*\)){0,3}[^()]*?)\s*,?\s*\)").unwrap()
    });
    for _ in 0..8 {
        let calls = transform_outside_literals(&updated, |text| {
            IMPL_CALL_REGEX
                .replace_all(text, |captures: &regex::Captures| {
                    let method = &captures[1];
                    let object = &captures[2];
                    let args = captures[3].trim().trim_end_matches(',').trim();
                    if args.is_empty() {
                        format!("{object}.{method}()")
                    } else {
                        format!("{object}.{method}({args})")
                    }
                })
                .to_string()
        });
        if calls == updated {
            break;
        }
        updated = calls;
        changed = true;
    }

    let (without_writes, wrote) = lower_cpp2rust_cout_writes(&updated);
    if wrote {
        updated = without_writes;
        changed = true;
    }

    let (merged_impls, merged_changed) = merge_duplicate_impl_blocks(&updated);
    if merged_changed {
        updated = merged_impls;
        changed = true;
    }

    // Inline the remaining `({ expr })` wrappers in expression position.
    static INLINE_BLOCK_EXPR: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\(\{\s*([^;{}]+?)\s*\}\)").unwrap());
    let inlined = transform_outside_literals(&updated, |text| {
        INLINE_BLOCK_EXPR.replace_all(text, "($1)").to_string()
    });
    if inlined != updated {
        updated = inlined;
        changed = true;
    }

    // A parenthesised method-call statement needs no parentheses.
    static PAREN_STMT_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^([ \t]*)\((?P<call>[A-Za-z_][A-Za-z0-9_]*\.[A-Za-z_][A-Za-z0-9_]*\([^;]*\))\);")
            .unwrap()
    });
    let unparenthesised = transform_outside_literals(&updated, |text| {
        PAREN_STMT_REGEX.replace_all(text, "$1${call};").to_string()
    });
    if unparenthesised != updated {
        updated = unparenthesised;
        changed = true;
    }

    // cpp2rust wraps statement expressions in `({ .. });` on their own line.
    static STATEMENT_EXPR_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^([ \t]*)\(\{\s*([^;{}]*?)\s*\}\);").unwrap()
    });
    let flattened = transform_outside_literals(&updated, |text| {
        STATEMENT_EXPR_REGEX.replace_all(text, "$1$2;").to_string()
    });
    if flattened != updated {
        updated = flattened;
        changed = true;
    }

    // Global-init hook is runtime-only.
    let without_init_call: Vec<&str> = updated
        .lines()
        .filter(|line| line.trim() != "__cpp2rust_init_globals();")
        .collect();
    let joined = without_init_call.join("\n");
    if joined != updated {
        updated = joined;
        changed = true;
    }
    static INIT_FN_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?m)^[ \t]*pub\s+fn\s+__cpp2rust_init_globals\s*\(\s*\)\s*\{\s*\}[ \t]*\n?").unwrap()
    });
    let without_init_fn = INIT_FN_REGEX.replace_all(&updated, "").to_string();
    if without_init_fn != updated {
        updated = without_init_fn;
        changed = true;
    }

    (updated, changed)
}

/// Rewrites cpp2rust field access inside a method body, leaving literals intact.
fn rewrite_cpp2rust_method_body(body: &str) -> String {
    static FIELD_WRITE_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"field!\(\(\*self\),\s*(\w+)\)\.write\(([^;]*)\);").unwrap());
    static FIELD_READ_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"field!\(\(\*self\),\s*(\w+)\)\.read\(\)").unwrap());
    static WITH_ACCESS_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\(\*self\)\.with\(\|__s\|\s*__s\.([A-Za-z_][A-Za-z0-9_]*)\)").unwrap());

    transform_outside_literals(body, |text| {
        let written = FIELD_WRITE_REGEX
            .replace_all(text, "self.$1 = $2;")
            .to_string();
        let read = FIELD_READ_REGEX
            .replace_all(&written, "self.$1")
            .to_string();
        WITH_ACCESS_REGEX
            .replace_all(&read, "self.$1")
            .to_string()
    })
}

/// Takes `&mut self` for methods that assign a field, `&self` otherwise.
fn mark_mutating_methods(body: &str) -> String {
    static FN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap());
    let mut updated = body.to_string();
    let mut edits: Vec<(usize, usize)> = Vec::new();
    for found in FN_REGEX.find_iter(body) {
        let Some(brace) = body[found.end()..].find('{').map(|o| found.end() + o) else {
            continue;
        };
        let Some(close) = find_matching_delimiter(body, brace, '{', '}') else {
            continue;
        };
        let signature = &body[found.start()..brace];
        if signature.contains("&mut self") || !signature.contains("&self") {
            continue;
        }
        let method_body = &body[brace..=close];
        let mutates = method_body.contains("self.")
            && Regex::new(r"\bself\.[A-Za-z_][A-Za-z0-9_]*\s*(?:[-+*/]?=)")
                .map(|pattern| pattern.is_match(method_body))
                .unwrap_or(false);
        if mutates {
            if let Some(receiver) = signature.find("&self") {
                edits.push((found.start() + receiver, found.start() + receiver + "&self".len()));
            }
        }
    }
    for (start, end) in edits.into_iter().rev() {
        updated.replace_range(start..end, "&mut self");
    }
    updated
}

/// Byte ranges of string literals, char literals and comments in `code`.
fn literal_ranges(code: &str) -> Vec<(usize, usize)> {
    let bytes = code.as_bytes();
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < bytes.len() {
        let start = index;
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                ranges.push((start, index));
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                    index += 1;
                }
                index = (index + 2).min(bytes.len());
                ranges.push((start, index));
            }
            b'r' | b'b' | b'c' => {
                // Possibly a raw string such as `r#"..."#` / `br#"..."#`.
                let mut cursor = index + 1;
                if bytes[index] == b'b' && bytes.get(cursor) == Some(&b'r') {
                    cursor += 1;
                } else if bytes[index] == b'c' && bytes.get(cursor) == Some(&b'"') {
                    index = scan_quoted(code, index + 1);
                    ranges.push((start, index));
                    continue;
                } else if bytes[index] != b'r' {
                    index += 1;
                    continue;
                }
                let mut hashes = 0usize;
                while bytes.get(cursor) == Some(&b'#') {
                    hashes += 1;
                    cursor += 1;
                }
                if bytes.get(cursor) != Some(&b'"') {
                    index += 1;
                    continue;
                }
                cursor += 1;
                let closing: Vec<u8> = std::iter::once(b'"')
                    .chain(std::iter::repeat(b'#').take(hashes))
                    .collect();
                while cursor < bytes.len() {
                    if bytes[cursor..].starts_with(&closing) {
                        cursor += closing.len();
                        break;
                    }
                    cursor += 1;
                }
                index = cursor;
                ranges.push((start, index));
            }
            b'"' => {
                index = scan_quoted(code, index);
                ranges.push((start, index));
            }
            b'\'' => {
                // A char literal closes within a couple of characters; a
                // lifetime such as `'a` does not.
                let mut cursor = index + 1;
                if bytes.get(cursor) == Some(&b'\\') {
                    cursor += 2;
                } else {
                    cursor += 1;
                }
                if bytes.get(cursor) == Some(&b'\'') {
                    index = cursor + 1;
                    ranges.push((start, index));
                } else {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }
    ranges
}

fn scan_quoted(code: &str, quote_start: usize) -> usize {
    let bytes = code.as_bytes();
    let mut index = quote_start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'"' => return index + 1,
            _ => index += 1,
        }
    }
    bytes.len()
}

/// Masks every literal/comment so regex passes cannot rewrite program text.
const MASK_OPEN: char = '\u{E000}';
const MASK_CLOSE: char = '\u{E001}';

fn mask_literals(code: &str) -> (String, Vec<String>) {
    let ranges = literal_ranges(code);
    if ranges.is_empty() {
        return (code.to_string(), Vec::new());
    }
    let mut masked = String::with_capacity(code.len());
    let mut literals = Vec::with_capacity(ranges.len());
    let mut cursor = 0usize;
    for (start, end) in ranges {
        masked.push_str(&code[cursor..start]);
        masked.push(MASK_OPEN);
        masked.push_str(&literals.len().to_string());
        masked.push(MASK_CLOSE);
        literals.push(code[start..end].to_string());
        cursor = end;
    }
    masked.push_str(&code[cursor..]);
    (masked, literals)
}

fn unmask_literals(code: &str, literals: &[String]) -> String {
    if literals.is_empty() {
        return code.to_string();
    }
    let mut restored = String::with_capacity(code.len());
    let mut rest = code;
    while let Some(open) = rest.find(MASK_OPEN) {
        restored.push_str(&rest[..open]);
        let after_open = &rest[open + MASK_OPEN.len_utf8()..];
        let Some(close) = after_open.find(MASK_CLOSE) else {
            restored.push_str(&rest[open..]);
            return restored;
        };
        let index: Option<usize> = after_open[..close].trim().parse().ok();
        match index.and_then(|index| literals.get(index)) {
            Some(literal) => restored.push_str(literal),
            None => restored.push_str(&rest[open..open + MASK_OPEN.len_utf8() + close + MASK_CLOSE.len_utf8()]),
        }
        rest = &after_open[close + MASK_CLOSE.len_utf8()..];
    }
    restored.push_str(rest);
    restored
}

/// Runs `transform` over the executable tokens only, leaving literals intact.
fn transform_outside_literals(code: &str, transform: impl Fn(&str) -> String) -> String {
    let (masked, literals) = mask_literals(code);
    if literals.is_empty() {
        return transform(code);
    }
    unmask_literals(&transform(&masked), &literals)
}

/// Detects a trailing `\n` escape and returns the literal without it.
fn strip_trailing_newline_escape(literal: &str) -> Option<&str> {
    let bytes = literal.as_bytes();
    if bytes.last() != Some(&b'n') {
        return None;
    }
    let mut backslashes = 0usize;
    let mut index = bytes.len() - 1;
    while index > 0 && bytes[index - 1] == b'\\' {
        backslashes += 1;
        index -= 1;
    }
    if backslashes % 2 == 1 {
        Some(&literal[..literal.len() - 2])
    } else {
        None
    }
}

/// Lowers `write!(libcc2rs::cout(), "fmt", args)` to `print!`/`println!`.
fn lower_cpp2rust_cout_writes(code: &str) -> (String, bool) {
    static WRITE_CALL_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(?s)\bwrite!\s*\(").unwrap());

    let mut result = String::with_capacity(code.len());
    let mut rest = code;
    let mut changed = false;

    while let Some(found) = WRITE_CALL_REGEX.find(rest) {
        let preceding = &rest[..found.start()];
        if literal_ranges(preceding).last().map(|(_, end)| *end) == Some(found.start()) {
            result.push_str(&rest[..found.end()]);
            rest = &rest[found.end()..];
            continue;
        }
        result.push_str(&rest[..found.start()]);
        let open_relative = found.end() - 1;
        let Some(close) = find_matching_delimiter(rest, open_relative, '(', ')') else {
            result.push_str(&rest[found.start()..]);
            return (result, changed);
        };
        let inner = &rest[open_relative + 1..close];
        let arguments = split_top_level_arguments(inner);
        let is_cout = arguments
            .first()
            .map(|argument| argument.starts_with("libcc2rs::cout()"))
            .unwrap_or(false);
        let format_literal = arguments
            .get(1)
            .and_then(|argument| argument.strip_prefix('"'))
            .and_then(|argument| argument.strip_suffix('"'));

        let (Some(_), Some(format_literal)) = (is_cout.then_some(()), format_literal) else {
            result.push_str(&rest[found.start()..=close]);
            rest = &rest[close + 1..];
            continue;
        };

        let (format, line_break) = match strip_trailing_newline_escape(format_literal) {
            Some(rest) => (rest.to_string(), true),
            None => (format_literal.to_string(), false),
        };
        let macro_name = if line_break { "println!" } else { "print!" };
        let rest_args: Vec<&str> = arguments[2..]
            .iter()
            .copied()
            .filter(|argument| !argument.is_empty())
            .collect();
        if rest_args.is_empty() {
            result.push_str(&format!("{macro_name}(\"{format}\")"));
        } else {
            result.push_str(&format!(
                "{macro_name}(\"{format}\", {})",
                rest_args.join(", ")
            ));
        }
        changed = true;
        rest = &rest[close + 1..];
    }
    result.push_str(rest);
    (result, changed)
}

/// Merges consecutive `impl Type { .. }` blocks into one.
fn merge_duplicate_impl_blocks(code: &str) -> (String, bool) {
    static IMPL_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"impl\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{").unwrap());

    let mut updated = code.to_string();
    let mut changed = false;
    loop {
        let blocks: Vec<(usize, usize, usize, usize, String)> = IMPL_REGEX
            .captures_iter(&updated)
            .filter_map(|captures| {
                let whole = captures.get(0)?;
                let brace = whole.end() - 1;
                let close = find_matching_delimiter(&updated, brace, '{', '}')?;
                Some((whole.start(), brace, close, close + 1, captures[1].to_string()))
            })
            .collect();

        let mut merged = false;
        for pair in blocks.windows(2) {
            let (start_a, brace_a, close_a, end_a, ref name_a) = pair[0];
            let (start_b, brace_b, close_b, end_b, ref name_b) = pair[1];
            if name_a != name_b || start_b < end_a {
                continue;
            }
            if !updated[end_a..start_b].trim().is_empty() {
                continue;
            }
            let inner_a = &updated[brace_a + 1..close_a];
            let inner_b = &updated[brace_b + 1..close_b];
            let combined = format!("impl {name_a} {{{inner_a}{inner_b}}}");
            updated = format!("{}{}{}", &updated[..start_a], combined, &updated[end_b..]);
            merged = true;
            changed = true;
            break;
        }
        if !merged {
            break;
        }
    }
    (updated, changed)
}

/// Drops `unsafe` from functions whose body performs no unsafe operation, and
/// from call sites that no longer need it. Verified by the caller, so a wrong
/// guess is rolled back instead of silently changing behaviour.
fn relax_unnecessary_unsafe_functions(code: &str) -> (String, bool) {
    static DEF_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?x)
            (?P<prefix>
                unsafe\s+extern\s+"C"\s+fn
                | unsafe\s+fn
                | extern\s+"C"\s+fn
            )
            \s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\(
            "#,
        )
        .unwrap()
    });

    let mut updated = code.to_string();
    let mut changed = false;

    loop {
        let snapshot = updated.clone();
        let definitions: Vec<(String, bool)> = DEF_REGEX
            .captures_iter(&snapshot)
            .map(|captures| {
                (
                    captures.name("name").unwrap().as_str().to_string(),
                    captures.name("prefix").unwrap().as_str().contains("unsafe"),
                )
            })
            .collect();

        let mut edits: Vec<(usize, usize, String)> = Vec::new();
        for captures in DEF_REGEX.captures_iter(&snapshot) {
            let whole = captures.get(0).unwrap();
            let prefix = captures.name("prefix").unwrap();
            let name = captures.name("name").unwrap().as_str().to_string();
            let Some(brace) = snapshot[whole.end()..].find('{').map(|o| whole.end() + o) else {
                continue;
            };
            let Some(close) = find_matching_delimiter(&snapshot, brace, '{', '}') else {
                continue;
            };
            let body = &snapshot[brace..=close];

            let calls_unsafe = definitions.iter().any(|(other, is_unsafe)| {
                *is_unsafe
                    && other != &name
                    && Regex::new(&format!(r"\b{}\s*\(", regex::escape(other)))
                        .map(|pattern| pattern.is_match(body))
                        .unwrap_or(true)
            });
            let performs_unsafe = body.contains("&raw")
                || body.contains("*mut")
                || body.contains("*const")
                || body.contains(".offset(")
                || body.contains("transmute")
                || body.contains("libc::")
                || body.contains("from_raw")
                || body.contains("get_unchecked")
                || calls_unsafe;
            if performs_unsafe {
                continue;
            }

            if !prefix.as_str().contains("extern") {
                edits.push((prefix.start(), prefix.end(), "fn".to_string()));
                continue;
            }

            // A `extern "C"` definition can become a plain Rust fn unless its
            // address is handed to C (a comparator, atexit hook, ...). Taking the
            // address shows up as a bare identifier or `&name` outside its own
            // definition and direct calls.
            let escaped = regex::escape(&name);
            // Count identifiers in code, not in literals/comments: a format string
            // that mentions the function name is not a use of it.
            let code_only = code_without_literals(&snapshot);
            let address_taken = Regex::new(&format!(r"&\s*{escaped}\b"))
                .map(|pattern| pattern.is_match(&code_only))
                .unwrap_or(true);
            let call_sites = Regex::new(&format!(r"\b{escaped}\s*\("))
                .map(|pattern| pattern.find_iter(&code_only).count())
                .unwrap_or(0);
            let identifier_sites = Regex::new(&format!(r"\b{escaped}\b"))
                .map(|pattern| pattern.find_iter(&code_only).count())
                .unwrap_or(0);
            let only_called = !address_taken && identifier_sites == call_sites;
            // Plain Rust when only Rust calls it; otherwise keep the C ABI so the
            // contract with C (function-pointer argument) still holds.
            let new_prefix = if only_called {
                "fn".to_string()
            } else {
                "extern \"C\" fn".to_string()
            };
            edits.push((prefix.start(), prefix.end(), new_prefix));
        }

        if edits.is_empty() {
            break;
        }
        for (start, end, replacement) in edits.into_iter().rev() {
            updated.replace_range(start..end, &replacement);
            changed = true;
        }
        if updated == snapshot {
            break;
        }
    }

    (updated, changed)
}

/// Folds `sizeof([T; N]) / sizeof(T)` back into the array length and, when the
/// enclosing loop walks a fixed-size array binding, expresses the bound with
/// `.len()` instead of the literal length.
fn simplify_c_array_length_bounds(code: &str) -> (String, bool) {
    static SIZE_DIV_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?s)\(\s*(?:::core::mem::)?size_of::<\[[^;]+;\s*(\d+)\]>\s*\(\s*\)\s*as\s+usize\s*\)\s*\.\s*wrapping_div\s*\(\s*(?:::core::mem::)?size_of::<[^<>]+>\s*\(\s*\)\s*as\s+usize\s*\)",
        )
        .unwrap()
    });
    static LOOP_BOUND_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"while\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s+as\s+usize\s*\)\s*<\s*(\d+)\s*\{").unwrap()
    });

    let folded = SIZE_DIV_REGEX.replace_all(code, "$1").to_string();
    let mut changed = folded != code;
    let mut updated = folded;

    let mut rewrite_at: Vec<(usize, usize, String)> = Vec::new();
    for captures in LOOP_BOUND_REGEX.captures_iter(&updated) {
        let whole = captures.get(0).unwrap();
        let index_var = captures[1].to_string();
        let length = captures[2].to_string();
        let Some(close) = find_matching_delimiter(&updated, whole.end() - 1, '{', '}') else {
            continue;
        };
        let body = &updated[whole.end()..close];
        let index_regex =
            Regex::new(&format!(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\[\s*{}\s+as\s+usize\s*\]", regex::escape(&index_var))).unwrap();
        let Some(array_name) = index_regex
            .captures(body)
            .map(|captures| captures[1].to_string())
        else {
            continue;
        };
        let declaration_regex = Regex::new(&format!(
            r"let\s+(?:mut\s+)?{}\s*:\s*\[[^;\]]+;\s*{}\]",
            regex::escape(&array_name),
            regex::escape(&length)
        ))
        .unwrap();
        if !declaration_regex.is_match(&updated) {
            continue;
        }
        rewrite_at.push((
            whole.start(),
            whole.end(),
            format!("while ({index_var} as usize) < {array_name}.len() {{"),
        ));
    }

    for (start, end, replacement) in rewrite_at.into_iter().rev() {
        updated.replace_range(start..end, &replacement);
        changed = true;
    }

    (updated, changed)
}

/// Language-wide readability pass over a C translation: replaces the C ABI type
/// names with Rust types, lowers `printf` to Rust formatting macros, and removes
/// the generated entrypoint wrapper. It never inspects identifiers or program
/// shapes, so it applies to any C source the upstream engine accepts.
fn normalize_c_abi_translation(code: &str) -> (String, bool) {
    static DROP_GENERATED_PUB: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"\bpub\s+(unsafe\s+extern\s+"C"\s+fn\s+|extern\s+"C"\s+fn\s+)"#).unwrap()
    });
    static LITERAL_CAST_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"\b(\d+)\s+as\s+(i8|i16|i32|i64|u8|u16|u32|u64|usize|isize)\b").unwrap()
    });

    let mut updated = code.to_string();

    if let Some(rewritten) = rewrite_c2rust_entrypoint(&updated) {
        updated = rewritten;
    }

    updated = updated
        .lines()
        .filter(|line| line.trim() != "#[no_mangle]")
        .collect::<Vec<_>>()
        .join("\n");

    // Type names first: `printf` lowering then sees plain Rust pointer types.
    for (c_type, rust_type) in C_ABI_TYPE_MAP {
        if updated.contains(c_type) {
            updated = updated.replace(c_type, rust_type);
        }
    }

    let (lowered, printf_changed) = lower_printf_calls(&updated);
    if printf_changed {
        updated = lowered;
    }

    let (puts_lowered, puts_changed) = lower_puts_literals(&updated);
    if puts_changed {
        updated = puts_lowered;
    }

    let (stdio_lowered, stdio_changed) = lower_char_buffer_stdio(&updated);
    if stdio_changed {
        updated = stdio_lowered;
    }

    let (heap_lowered, heap_changed) = lower_malloc_arrays_to_vec(&updated);
    if heap_changed {
        updated = heap_lowered;
    }

    let (bounds_simplified, bounds_changed) = simplify_c_array_length_bounds(&updated);
    if bounds_changed {
        updated = bounds_simplified;
    }

    updated = updated.replace("println!(\"\")", "println!()");

    let (collapsed, collapsed_changed) = collapse_true_blocks(&updated);
    if collapsed_changed {
        updated = collapsed;
    }
    updated = Regex::new(r"=\s*\{\s*([A-Za-z_][A-Za-z0-9_]*\(\))\s*\};")
        .unwrap()
        .replace_all(&updated, "= $1;")
        .to_string();

    // c2rust exports every translated function to C; a standalone program keeps the
    // C ABI on its definitions (callbacks such as `qsort` comparators need it) but not
    // the `pub` that only made sense inside the generated crate.
    updated = DROP_GENERATED_PUB.replace_all(&updated, "$1").to_string();

    for _ in 0..3 {
        let rewritten = LITERAL_CAST_REGEX
            .replace_all(&updated, "$1$2")
            .to_string();
        if rewritten == updated {
            break;
        }
        updated = rewritten;
    }

    let (pruned_types, pruned_types_changed) = prune_unused_c_type_declarations(&updated);
    if pruned_types_changed {
        updated = pruned_types;
    }

    let (pruned, pruned_changed) = prune_unused_c_ffi_blocks(&updated);
    if pruned_changed {
        updated = pruned;
    }

    let changed = updated != code;
    (updated, changed)
}

fn simplify_cpp2rust_runtime_scaffolding(code: &str) -> (String, bool) {
    if !code.contains("libcc2rs") || !code.contains("__cpp2rust_init_globals") {
        return (code.to_string(), false);
    }

    let entrypoint_regex = Regex::new(
        r"(?s)pub\s+fn\s+main\s*\(\s*\)\s*\{\s*__cpp2rust_init_globals\(\);\s*std::process::exit\(main_0\(\)\);\s*\}\s*fn\s+main_0\s*\(\s*\)\s*->\s*i32\s*\{",
    )
    .unwrap();
    let stdout_regex = Regex::new(
        r#"(?s)\{\s*let\s+mut\s+__bytes\s*=\s*Ptr::<i8>::from_string_literal\(b\"((?:[^\"\\]|\\.)*)\"\)\.to_c_u8_bytes\(\);\s*__bytes\.push\(b'\\n'\);\s*match\s+libcc2rs::c_stdout\(\)\.with_mut\(\|__f\|\s*__f\.write\(&__bytes\)\)\s*==\s*__bytes\.len\(\)\s*\{\s*true\s*=>\s*0,\s*false\s*=>\s*-1,\s*\}\s*\};"#,
    )
    .unwrap();
    let init_regex = Regex::new(r"(?s)\s*pub\s+fn\s+__cpp2rust_init_globals\s*\(\s*\)\s*\{\s*\}\s*$").unwrap();
    let final_success_regex = Regex::new(r"(?s)\s*return\s+0(?:_i32)?;\s*\}\s*$").unwrap();
    let return_status_regex = Regex::new(r"\breturn\s+(-?\d+)_i32;").unwrap();

    let mut simplified = entrypoint_regex.replace(code, "fn main() {").to_string();
    if simplified == code {
        return (code.to_string(), false);
    }

    simplified = stdout_regex
        .replace_all(&simplified, |captures: &regex::Captures| {
            format!("println!(\"{}\");", captures.get(1).unwrap().as_str())
        })
        .to_string();
    simplified = init_regex.replace(&simplified, "").to_string();
    simplified = final_success_regex.replace(&simplified, "\n}").to_string();
    simplified = return_status_regex
        .replace_all(&simplified, "std::process::exit($1);")
        .to_string();

    let is_runtime_import = |line: &str| {
        matches!(
            line.trim(),
            "extern crate libcc2rs;"
                | "use libcc2rs::*;"
                | "use std::cell::RefCell;"
                | "use std::collections::BTreeMap;"
                | "use std::io::prelude::*;"
                | "use std::io::{Read, Seek, Write};"
                | "use std::os::fd::AsFd;"
                | "use std::rc::{Rc, Weak};"
        )
    };
    let code_without_runtime_imports = simplified
        .lines()
        .filter(|line| !is_runtime_import(line))
        .collect::<Vec<_>>()
        .join("\n");
    if !code_without_runtime_imports.contains("libcc2rs")
        && !code_without_runtime_imports.contains("Ptr::<")
        && !code_without_runtime_imports.contains("__cpp2rust")
    {
        simplified = code_without_runtime_imports;
    }

    (simplified, true)
}

#[derive(Debug, Deserialize)]
pub(crate) struct RustcDiagnosticMessage {
    level: Option<String>,
    code: Option<RustcDiagnosticCode>,
    message: Option<String>,
    spans: Option<Vec<RustcDiagnosticSpan>>,
}

#[derive(Debug, Deserialize)]
struct RustcDiagnosticCode {
    code: String,
}

#[derive(Debug, Deserialize)]
struct RustcDiagnosticSpan {
    line_start: usize,
    column_start: usize,
    line_end: usize,
    column_end: usize,
    suggested_replacement: Option<String>,
}

static RUSTFMT_BINARY: Lazy<Option<String>> = Lazy::new(|| {
    let candidates = [
        "rustfmt",
        "/opt/homebrew/bin/rustfmt",
        "/usr/local/bin/rustfmt",
        "/usr/bin/rustfmt",
    ];
    // Resolving from PATH with a file check avoids a slow `--version` spawn on
    // every conversion; the probe is only the fallback.
    for candidate in candidates {
        if !candidate.contains('/') {
            if let Ok(path) = std::env::var("PATH") {
                for dir in path.split(':') {
                    let full = Path::new(dir).join(candidate);
                    if full.is_file() {
                        return Some(full.to_string_lossy().to_string());
                    }
                }
            }
            continue;
        }
        if Path::new(candidate).is_file() {
            return Some(candidate.to_string());
        }
    }
    for candidate in ["rustfmt", "/opt/homebrew/bin/rustfmt", "/usr/local/bin/rustfmt"] {
        if Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(candidate.to_string());
        }
    }
    None
});

fn find_rustfmt_binary() -> Option<String> {
    RUSTFMT_BINARY.clone()
}

fn find_rustc_binary() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut candidates = vec![
        "rustc".to_string(),
        "/opt/homebrew/bin/rustc".to_string(),
        "/usr/local/bin/rustc".to_string(),
        "/usr/bin/rustc".to_string(),
    ];
    if !home.is_empty() {
        candidates.push(format!("{}/.cargo/bin/rustc", home));
    }
    for candidate in candidates {
        if candidate.contains('/') && !Path::new(&candidate).is_file() {
            continue;
        }
        if Command::new(&candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(candidate);
        }
    }
    None
}

pub fn format_with_rustfmt(code: &str) -> Option<String> {
    let rustfmt = find_rustfmt_binary()?;
    let mut child = Command::new(rustfmt)
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(code.as_bytes()).ok()?;
    }
    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        let formatted = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
        if !formatted.is_empty() {
            return Some(formatted);
        }
    }
    None
}

pub(crate) fn check_rustc_and_collect_diagnostics(
    rust_code: &str,
) -> Result<Vec<RustcDiagnosticMessage>, String> {
    let Some(rustc) = find_rustc_binary() else {
        return Ok(Vec::new());
    };
    let temp_dir = TempEngineDirectory::create().map_err(|e| e.to_string())?;
    let source_path = temp_dir.path.join("verify.rs");
    let output_path = temp_dir.path.join("verify_out");
    fs::write(&source_path, rust_code).map_err(|e| e.to_string())?;

    let crate_type = if rust_code.contains("fn main(") {
        "bin"
    } else {
        "lib"
    };
    let mut cmd = Command::new(rustc);
    cmd.arg("--crate-type").arg(crate_type);
    for extra_arg in rustc_extra_args_for_code(rust_code) {
        cmd.arg(extra_arg);
    }
    let output = cmd
        .arg("--emit=metadata")
        .arg("--error-format=json")
        .arg("-o")
        .arg(&output_path)
        .arg(&source_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;

    let stderr_text = String::from_utf8_lossy(&output.stderr);
    let mut parsed_diagnostics = Vec::new();
    let mut errors = Vec::new();

    for line in stderr_text.lines() {
        if let Ok(msg) = serde_json::from_str::<RustcDiagnosticMessage>(line) {
            if msg.level.as_deref() == Some("error") {
                if let Some(ref text) = msg.message {
                    errors.push(text.clone());
                }
            }
            parsed_diagnostics.push(msg);
        }
    }

    if output.status.success() {
        Ok(parsed_diagnostics)
    } else if errors.is_empty() {
        Err(stderr_text.trim().to_string())
    } else {
        Err(errors.join("; "))
    }
}

fn detect_c2rust_driver_binary() -> Option<String> {
    let root = workspace_root_dir();
    let mut candidates = Vec::new();
    if let Ok(env_bin) = std::env::var("C2RUST_BIN") {
        if !env_bin.trim().is_empty() {
            candidates.push(env_bin.trim().to_string());
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            candidates.push(format!("{}/.cargo/bin/c2rust", home));
        }
    }
    candidates.push(root.join(".tools/bin/c2rust").to_string_lossy().to_string());
    candidates.push(".tools/bin/c2rust".to_string());
    candidates.push("/opt/homebrew/bin/c2rust".to_string());
    candidates.push("/usr/local/bin/c2rust".to_string());
    candidates.push("c2rust".to_string());

    candidates.into_iter().find(|candidate| {
        Command::new(candidate)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

fn try_external_c2rust_tools(code: &str) -> (String, Vec<String>) {
    let mut current = code.to_string();
    let mut applied = Vec::new();

    if current.contains("unsafe") {
        if let Some(c2rust_bin) = detect_c2rust_driver_binary() {
            if let Ok(temp_dir) = TempEngineDirectory::create() {
                let rs_path = temp_dir.path.join("input.rs");
                if fs::write(&rs_path, &current).is_ok() {
                    let status = Command::new(&c2rust_bin)
                        .arg("refactor")
                        .arg("reorganize_definitions")
                        .arg("--")
                        .arg(&rs_path)
                        .arg("--crate-type")
                        .arg("bin")
                        .arg("--edition")
                        .arg("2021")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                    if status.map(|s| s.success()).unwrap_or(false) {
                        if let Ok(refactored) = fs::read_to_string(&rs_path) {
                            if !refactored.trim().is_empty() && refactored != current {
                                current = refactored;
                                applied.push("c2rust_refactor".to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let postprocess_bin = std::env::var("C2RUST_POSTPROCESS_BIN")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            if Command::new("c2rust-postprocess")
                .arg("--help")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok()
            {
                Some("c2rust-postprocess".to_string())
            } else {
                None
            }
        });

    if let Some(bin) = postprocess_bin {
        if let Ok(temp_dir) = TempEngineDirectory::create() {
            let rs_path = temp_dir.path.join("input.rs");
            if fs::write(&rs_path, &current).is_ok() {
                let status = Command::new(&bin)
                    .arg(&rs_path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                if status.map(|s| s.success()).unwrap_or(false) {
                    if let Ok(cleaned) = fs::read_to_string(&rs_path) {
                        if !cleaned.trim().is_empty() && cleaned != current {
                            current = cleaned;
                            applied.push("c2rust_postprocess".to_string());
                        }
                    }
                }
            }
        }
    }

    (current, applied)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StringState {
    None,
    Normal,
    Raw(usize), // number of hashes
    BlockComment(usize), // nesting depth >= 1
}

fn scan_line_string_state(line: &str, mut state: StringState) -> (StringState, Option<usize>) {
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match state {
            StringState::BlockComment(depth) => {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    state = StringState::BlockComment(depth + 1);
                    i += 2;
                    continue;
                }
                if bytes[i] == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    if depth <= 1 {
                        state = StringState::None;
                    } else {
                        state = StringState::BlockComment(depth - 1);
                    }
                    i += 2;
                    continue;
                }
                i += 1;
            }
            StringState::Normal => {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    state = StringState::None;
                }
                i += 1;
            }
            StringState::Raw(hashes) => {
                if bytes[i] == b'"' {
                    let mut matched_hashes = 0;
                    while i + 1 + matched_hashes < bytes.len() && bytes[i + 1 + matched_hashes] == b'#' {
                        matched_hashes += 1;
                    }
                    if matched_hashes >= hashes {
                        state = StringState::None;
                        i += 1 + hashes;
                        continue;
                    }
                }
                i += 1;
            }
            StringState::None => {
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                    state = StringState::BlockComment(1);
                    i += 2;
                    continue;
                }
                if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    return (StringState::None, Some(i));
                }
                if bytes[i] == b'r' && i + 1 < bytes.len() {
                    let mut hash_count = 0;
                    let mut j = i + 1;
                    while j < bytes.len() && bytes[j] == b'#' {
                        hash_count += 1;
                        j += 1;
                    }
                    if j < bytes.len() && bytes[j] == b'"' {
                        state = StringState::Raw(hash_count);
                        i = j + 1;
                        continue;
                    }
                }
                if bytes[i] == b'"' {
                    state = StringState::Normal;
                    i += 1;
                    continue;
                }
                if bytes[i] == b'\'' {
                    i += 1;
                    if i < bytes.len() && bytes[i] == b'\\' {
                        i += 2;
                    } else {
                        i += 1;
                    }
                    if i < bytes.len() && bytes[i] == b'\'' {
                        i += 1;
                    }
                    continue;
                }
                i += 1;
            }
        }
    }
    (state, None)
}

fn strip_transpiler_boilerplate(code: &str) -> (String, bool) {
    let mut output_lines: Vec<String> = Vec::new();
    let mut changed = false;
    let mut prev_blank = false;
    let mut string_state = StringState::None;

    for raw_line in code.lines() {
        let prev_state = string_state;
        let (next_state, comment_pos_opt) = scan_line_string_state(raw_line, string_state);
        string_state = next_state;

        if prev_state != StringState::None || next_state != StringState::None {
            if let Some(pos) = comment_pos_opt {
                let comment = raw_line[pos..].trim();
                let is_pedagogical = comment.starts_with("// & borrows ")
                    || comment.starts_with("// &mut borrows ")
                    || comment.starts_with("// C++ erase-remove")
                    || comment.starts_with("// Vec::insert ")
                    || comment.starts_with("// std::replace ");
                if is_pedagogical {
                    changed = true;
                    output_lines.push(raw_line[..pos].trim_end().to_string());
                    continue;
                }
            }
            output_lines.push(raw_line.to_string());
            continue;
        }

        let trimmed = raw_line.trim();
        let is_boilerplate = BOILERPLATE_EXACT_LINES.contains(&trimmed)
            || trimmed.starts_with("// #include <")
            || trimmed.starts_with("// #include \"")
            || trimmed.starts_with("// Forward declaration: ")
            || trimmed.starts_with("// Global pointer '")
            || trimmed.starts_with("// Pointer alias: ")
            || trimmed.starts_with("// C11 _Generic macro ")
            || trimmed.starts_with("// static_assert(");

        if is_boilerplate {
            changed = true;
            continue;
        }

        let line_without_pedagogical = if let Some(pos) = comment_pos_opt {
            let comment = raw_line[pos..].trim();
            let is_pedagogical = comment.starts_with("// & borrows ")
                || comment.starts_with("// &mut borrows ")
                || comment.starts_with("// C++ erase-remove")
                || comment.starts_with("// Vec::insert ")
                || comment.starts_with("// std::replace ");
            if is_pedagogical {
                changed = true;
                &raw_line[..pos]
            } else {
                raw_line
            }
        } else {
            raw_line
        };

        let cleaned_line = line_without_pedagogical.trim_end().to_string();
        if cleaned_line.is_empty() {
            if prev_blank || output_lines.is_empty() {
                changed = true;
                continue;
            }
            prev_blank = true;
        } else {
            prev_blank = false;
        }

        output_lines.push(cleaned_line);
    }

    while output_lines.last().is_some_and(|line| line.is_empty()) {
        output_lines.pop();
        changed = true;
    }

    (output_lines.join("\n"), changed)
}

fn unwrap_trivial_unsafe_blocks(code: &str) -> (String, usize) {
    static UNSAFE_FN_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r#"\bunsafe\s+(?:extern\s+"C"\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"#).unwrap()
    });
    static UNSAFE_TOKEN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bunsafe\b").unwrap());

    let mut updated = code.replace("pub unsafe fn main(", "fn main(");
    updated = updated.replace("unsafe fn main(", "fn main(");
    let mut count = if updated != code { 1 } else { 0 };

    let unsafe_functions: Vec<String> = UNSAFE_FN_REGEX
        .captures_iter(code)
        .map(|captures| captures[1].to_string())
        .collect();
    let calls_unsafe_function = |inner: &str| {
        unsafe_functions.iter().any(|name| {
            Regex::new(&format!(r"\b{}\s*\(", regex::escape(name)))
                .map(|pattern| pattern.is_match(inner))
                .unwrap_or(true)
        })
    };

    loop {
        let snapshot = updated.clone();
        let mut edits: Vec<(usize, usize)> = Vec::new();
        for token in UNSAFE_TOKEN_REGEX.find_iter(&snapshot) {
            let after = snapshot[token.end()..].trim_start();
            if !after.starts_with('{') {
                continue;
            }
            let brace = snapshot.len() - after.len();
            let Some(close) = find_matching_delimiter(&snapshot, brace, '{', '}') else {
                continue;
            };
            let inner = &snapshot[brace + 1..close];
            // Braces inside format strings must not be treated as block nesting,
            // so the block extent comes from the delimiter matcher.
            let has_unsafe_op = inner.contains('*')
                || inner.contains("transmute")
                || inner.contains("libc::")
                || inner.contains("from_raw")
                || inner.contains("as_ptr")
                || inner.contains("as_mut_ptr")
                || inner.contains("get_unchecked")
                || inner.contains("extern")
                || inner.contains("&raw")
                || inner.contains(".offset(")
                || calls_unsafe_function(inner);
            if !has_unsafe_op {
                edits.push((token.start(), close + 1));
            }
        }

        if edits.is_empty() {
            break;
        }
        for (start, end) in edits.into_iter().rev() {
            let inner = snapshot[start..end]
                .trim_start_matches("unsafe")
                .trim_start()
                .trim_start_matches('{')
                .trim_end_matches('}')
                .to_string();
            updated.replace_range(start..end, &inner);
            count += 1;
        }
        if updated == snapshot {
            break;
        }
    }

    (updated, count)
}

fn is_collection_used(code: &str, var_name: &str) -> bool {
    let escaped = regex::escape(var_name);
    let usage_regex = Regex::new(&format!(
        r"\b{}\s*\.(?:push|push_back|insert|entry)\s*\(",
        escaped
    ))
    .unwrap();
    usage_regex.is_match(code)
}

fn replace_c_types_and_unsafe_wrappers(code: &str) -> (String, usize, bool) {
    let mut updated = code.to_string();
    for &(c_type, rust_type) in LIBC_TYPE_MAP {
        if updated.contains(c_type) {
            updated = updated.replace(c_type, rust_type);
        }
    }

    let (unwrapped, unsafe_removed) = unwrap_trivial_unsafe_blocks(&updated);
    updated = unwrapped;

    let box_default_regex = Regex::new(
        r"\blet\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Box<([A-Za-z_][A-Za-z0-9_]*)>\s*=\s*Box::new\(Default::default\(\)\);",
    )
    .unwrap();
    updated = box_default_regex
        .replace_all(&updated, "let mut $1: Box<$2> = Box::new(Default::default());")
        .to_string();

    let box_struct_regex = Regex::new(
        r"\blet\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Box<([A-Za-z_][A-Za-z0-9_]*)>\s*=\s*Box::new\(([A-Za-z_][A-Za-z0-9_]*\s*\{)",
    )
    .unwrap();
    updated = box_struct_regex
        .replace_all(&updated, "let mut $1 = Box::new($3")
        .to_string();

    let string_from_regex = Regex::new(
        r"\blet\s+(mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*String\s*=\s*String::from\(",
    )
    .unwrap();
    updated = string_from_regex
        .replace_all(&updated, "let $1$2 = String::from(")
        .to_string();

    let string_new_regex = Regex::new(
        r"\blet\s+(mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*String\s*=\s*String::new\(\);",
    )
    .unwrap();
    updated = string_new_regex
        .replace_all(&updated, "let $1$2 = String::new();")
        .to_string();

    let snapshot = updated.clone();
    for (pattern, constructor) in [
        (
            r"\blet\s+(mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Vec<[^>]+>\s*=\s*Vec::new\(\);",
            "Vec::new();",
        ),
        (
            r"\blet\s+(mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*VecDeque<[^>]+>\s*=\s*VecDeque::new\(\);",
            "VecDeque::new();",
        ),
        (
            r"\blet\s+(mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*HashMap<[^>]+>\s*=\s*HashMap::new\(\);",
            "HashMap::new();",
        ),
        (
            r"\blet\s+(mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*HashSet<[^>]+>\s*=\s*HashSet::new\(\);",
            "HashSet::new();",
        ),
    ] {
        let re = Regex::new(pattern).unwrap();
        updated = re
            .replace_all(&updated, |captures: &regex::Captures| {
                let mut_kw = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                let var_name = captures.get(2).unwrap().as_str();
                if is_collection_used(&snapshot, var_name) {
                    format!("let {}{} = {}", mut_kw, var_name, constructor)
                } else {
                    captures.get(0).unwrap().as_str().to_string()
                }
            })
            .to_string();
    }

    let changed = updated != code;
    (updated, unsafe_removed, changed)
}

fn remove_redundant_casts_and_parens(code: &str) -> (String, usize, bool) {
    let mut updated = code.to_string();
    let mut cast_count = 0usize;

    let mut usize_vars: HashSet<String> = HashSet::new();
    for cap in USIZE_VAR_REGEX.captures_iter(&updated) {
        usize_vars.insert(cap.get(1).unwrap().as_str().to_string());
    }
    for cap in USIZE_PARAM_REGEX.captures_iter(&updated) {
        usize_vars.insert(cap.get(1).unwrap().as_str().to_string());
    }

    for var_name in &usize_vars {
        let escaped = regex::escape(var_name);
        let wrapped_re = Regex::new(&format!(r"\(\s*{}\s*\)\s+as\s+usize\b", escaped)).unwrap();
        let next = wrapped_re.replace_all(&updated, var_name.as_str()).to_string();
        if next != updated {
            cast_count += 1;
            updated = next;
        }
        let bare_re = Regex::new(&format!(r"\b{}\s+as\s+usize\b", escaped)).unwrap();
        let next_bare = bare_re.replace_all(&updated, var_name.as_str()).to_string();
        if next_bare != updated {
            cast_count += 1;
            updated = next_bare;
        }
    }

    let single_ident_cast_paren =
        Regex::new(r"\[\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s+as\s+usize\s*\]").unwrap();
    let next_idx = single_ident_cast_paren
        .replace_all(&updated, |captures: &regex::Captures| {
            cast_count += 1;
            format!("[{} as usize]", captures.get(1).unwrap().as_str())
        })
        .to_string();
    updated = next_idx;

    let cond_parens_re = Regex::new(r"\b(if|while)\s+\(([^()]+)\)\s*\{").unwrap();
    updated = cond_parens_re
        .replace_all(&updated, "$1 $2 {")
        .to_string();

    let changed = updated != code;
    (updated, cast_count, changed)
}

const TRIVIAL_PRIMITIVE_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char",
];

#[derive(Default)]
struct LineScanState {
    block_comment_depth: usize,
    in_string: bool,
    string_escaped: bool,
    raw_string_hashes: Option<usize>,
}

fn match_raw_string_start(chars: &[char], i: usize) -> Option<usize> {
    let prev_is_ident = i > 0 && (chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '_');
    if prev_is_ident {
        return None;
    }
    let hash_start = if (chars[i] == 'b' || chars[i] == 'c')
        && i + 1 < chars.len()
        && chars[i + 1] == 'r'
    {
        i + 2
    } else if chars[i] == 'r' {
        i + 1
    } else {
        return None;
    };
    let mut k = hash_start;
    let mut hashes = 0usize;
    while k < chars.len() && chars[k] == '#' {
        hashes += 1;
        k += 1;
    }
    if k < chars.len() && chars[k] == '"' {
        Some(hashes)
    } else {
        None
    }
}

fn strip_comments_and_strings_on_line(line: &str, state: &mut LineScanState) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut output = String::with_capacity(line.len());
    let mut i = 0;

    while i < chars.len() {
        if state.block_comment_depth > 0 {
            if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                state.block_comment_depth += 1;
                i += 2;
            } else if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                state.block_comment_depth -= 1;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if state.in_string {
            if state.string_escaped {
                state.string_escaped = false;
            } else if chars[i] == '\\' {
                state.string_escaped = true;
            } else if chars[i] == '"' {
                state.in_string = false;
            }
            i += 1;
            continue;
        }

        if let Some(hashes) = state.raw_string_hashes {
            if chars[i] == '"'
                && i + 1 + hashes <= chars.len()
                && (0..hashes).all(|k| chars[i + 1 + k] == '#')
            {
                state.raw_string_hashes = None;
                i += 1 + hashes;
            } else {
                i += 1;
            }
            continue;
        }

        if chars[i] == '/' && i + 1 < chars.len() {
            if chars[i + 1] == '/' {
                break;
            }
            if chars[i + 1] == '*' {
                state.block_comment_depth += 1;
                i += 2;
                continue;
            }
        }

        if let Some(hashes) = match_raw_string_start(&chars, i) {
            output.push(' ');
            let prefix_len = if chars[i] == 'r' { 1 } else { 2 };
            state.raw_string_hashes = Some(hashes);
            i += prefix_len + hashes + 1;
            continue;
        }

        if chars[i] == '"' {
            output.push(' ');
            state.in_string = true;
            state.string_escaped = false;
            i += 1;
            continue;
        }

        if chars[i] == '\'' {
            if i + 2 < chars.len() && chars[i + 1] == '\\' {
                let scan_limit = (i + 14).min(chars.len());
                if let Some(end_rel) = chars[(i + 2)..scan_limit].iter().position(|&c| c == '\'') {
                    output.push(' ');
                    i += 2 + end_rel + 1;
                    continue;
                }
            }
            if i + 2 < chars.len() && chars[i + 1] != '\'' && chars[i + 2] == '\'' {
                output.push(' ');
                i += 3;
                continue;
            }
        }

        output.push(chars[i]);
        i += 1;
    }

    output
}

fn recompute_code_only_lines(lines: &[String]) -> Vec<String> {
    let mut state = LineScanState::default();
    lines
        .iter()
        .map(|line| strip_comments_and_strings_on_line(line, &mut state))
        .collect()
}

fn has_balanced_parens(expr: &str) -> bool {
    let mut balance = 0isize;
    for ch in expr.chars() {
        if ch == '(' {
            balance += 1;
        } else if ch == ')' {
            balance -= 1;
            if balance < 0 {
                return false;
            }
        }
    }
    balance == 0
}

static INNER_DECL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^\s*(?:(?:pub(?:\s*\([^)]*\))?\s+)?(?:(?:async|const|unsafe)\s+)*(?:let|fn|struct|enum|type|const|static|mod|use|trait|impl)\b|macro_rules!\s*)",
    )
    .unwrap()
});

fn has_inner_declaration_statement(code_lines: &[String]) -> bool {
    code_lines.iter().any(|line| INNER_DECL_REGEX.is_match(line))
}

fn simplify_idiomatic_box_and_null_checks(code: &str) -> (String, bool) {
    let original = code.to_string();
    let mut lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();

    loop {
        let code_only_lines = recompute_code_only_lines(&lines);
        let mut unwrapped = false;
        let mut i = 0;
        while i < lines.len() {
            if code_only_lines[i].trim() == "if true {" {
                let mut depth = 1isize;
                let mut close_idx = None;
                for (j, code_line) in code_only_lines.iter().enumerate().skip(i + 1) {
                    for ch in code_line.chars() {
                        if ch == '{' {
                            depth += 1;
                        } else if ch == '}' {
                            depth -= 1;
                        }
                    }
                    if depth == 0 && code_line.trim() == "}" {
                        close_idx = Some(j);
                        break;
                    }
                }
                if let Some(j) = close_idx {
                    let has_else_after = code_only_lines
                        .get(j + 1)
                        .is_some_and(|next| next.trim().starts_with("else"));
                    if !has_else_after {
                        if has_inner_declaration_statement(&code_only_lines[(i + 1)..j]) {
                            let indent: String = lines[i]
                                .chars()
                                .take_while(|c| c.is_whitespace())
                                .collect();
                            lines[i] = format!("{}{{", indent);
                        } else {
                            lines.remove(j);
                            lines.remove(i);
                        }
                        unwrapped = true;
                        break;
                    }
                }
            }
            i += 1;
        }
        if !unwrapped {
            break;
        }
    }

    let box_default_line_re = Regex::new(
        r"^\s*let\s+mut\s+([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Box<([^>]+)>\s*=\s*Box::new\(Default::default\(\)\);\s*$",
    )
    .unwrap();
    let deref_init_line_re =
        Regex::new(r"^\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*([^;]+);\s*$").unwrap();

    let mut code_only_lines = recompute_code_only_lines(&lines);
    let mut idx = 0;
    while idx < lines.len() {
        if let Some(decl_cap) = box_default_line_re.captures(&code_only_lines[idx]) {
            let var_name = decl_cap.get(1).unwrap().as_str();
            let box_type = decl_cap.get(2).unwrap().as_str().trim();
            if TRIVIAL_PRIMITIVE_TYPES.contains(&box_type) {
                let mut next_idx = idx + 1;
                while next_idx < lines.len() && code_only_lines[next_idx].trim().is_empty() {
                    next_idx += 1;
                }
                if next_idx < lines.len() {
                    if let Some(assign_cap) = deref_init_line_re.captures(&code_only_lines[next_idx]) {
                        if assign_cap.get(1).unwrap().as_str() == var_name {
                            let expr = assign_cap.get(2).unwrap().as_str().trim();
                            let self_ref_re =
                                Regex::new(&format!(r"\b{}\b", regex::escape(var_name))).unwrap();
                            if !self_ref_re.is_match(expr) && has_balanced_parens(expr) {
                                let indent: String = lines[idx]
                                    .chars()
                                    .take_while(|c| c.is_whitespace())
                                    .collect();
                                lines[idx] = format!(
                                    "{}let mut {}: Box<{}> = Box::new({});",
                                    indent, var_name, box_type, expr
                                );
                                lines.remove(next_idx);
                                code_only_lines = recompute_code_only_lines(&lines);
                            }
                        }
                    }
                }
            }
        }
        idx += 1;
    }

    let drop_line_re =
        Regex::new(r"^\s*drop\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)\s*;\s*$").unwrap();
    let custom_fn_drop_re = Regex::new(r"\bfn\s+drop\b").unwrap();
    let box_typed_decl_re = Regex::new(
        r"^\s*let\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*Box<([^>]+)>\s*=",
    )
    .unwrap();
    let any_let_decl_re = Regex::new(r"^\s*let\s+").unwrap();

    let has_custom_fn_drop = code_only_lines
        .iter()
        .any(|line| custom_fn_drop_re.is_match(line));

    if !has_custom_fn_drop {
        if let Some(main_idx) = code_only_lines
            .iter()
            .position(|l| l.trim() == "fn main() {")
        {
            let mut depth = 0isize;
            let mut main_end_idx = None;
            for (k, code_line) in code_only_lines.iter().enumerate().skip(main_idx) {
                for ch in code_line.chars() {
                    if ch == '{' {
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                }
                if depth == 0 && k > main_idx {
                    main_end_idx = Some(k);
                    break;
                }
            }
            if let Some(end_k) = main_end_idx {
                let mut prev_k = end_k.saturating_sub(1);
                while prev_k > main_idx && code_only_lines[prev_k].trim().is_empty() {
                    prev_k -= 1;
                }
                let has_nested_braces = code_only_lines[(main_idx + 1)..=prev_k]
                    .iter()
                    .any(|line| line.contains('{') || line.contains('}'));
                if prev_k > main_idx && !has_nested_braces {
                    if let Some(drop_cap) = drop_line_re.captures(&code_only_lines[prev_k]) {
                        let drop_var = drop_cap.get(1).unwrap().as_str();
                        if let Some((decl_idx, box_type)) = ((main_idx + 1)..prev_k)
                            .rev()
                            .find_map(|pos| {
                                box_typed_decl_re
                                    .captures(&code_only_lines[pos])
                                    .filter(|cap| cap.get(1).unwrap().as_str() == drop_var)
                                    .map(|cap| (pos, cap.get(2).unwrap().as_str().trim()))
                            })
                        {
                            let is_last_declared = !code_only_lines[(decl_idx + 1)..prev_k]
                                .iter()
                                .any(|line| any_let_decl_re.is_match(line));
                            if TRIVIAL_PRIMITIVE_TYPES.contains(&box_type) && is_last_declared {
                                lines.remove(prev_k);
                            }
                        }
                    }
                }
            }
        }
    }

    let updated = lines.join("\n");
    let changed = updated != original;
    (updated, changed)
}

fn is_variable_mutated(code: &str, var_name: &str) -> bool {
    let escaped = regex::escape(var_name);
    let mut_ref_re = Regex::new(&format!(r"&\s*mut\s+{}\b", escaped)).unwrap();
    if mut_ref_re.is_match(code) {
        return true;
    }

    let deref_assign_re = Regex::new(&format!(
        r"\*\s*{}\s*(?:[+\-*/%&|^]|<<|>>)?=[^=]",
        escaped
    ))
    .unwrap();
    if deref_assign_re.is_match(code) {
        return true;
    }

    let direct_assign_re = Regex::new(&format!(
        r"\b{}(?:\s*\.\s*[A-Za-z_][A-Za-z0-9_]*|\s*\[[^\[\]]+\])*\s*(?:[+\-*/%&|^]|<<|>>)?=[^=]",
        escaped
    ))
    .unwrap();
    for captures in direct_assign_re.find_iter(code) {
        let start = captures.start();
        let prefix = &code[..start];
        let trimmed_prefix = prefix.trim_end();
        if !trimmed_prefix.ends_with("let mut") && !trimmed_prefix.ends_with("let") {
            return true;
        }
        // Check if there is a colon `let mut var: Type =` — that's the initializer, not a mutation;
        // whereas chained `.field =` or `[idx] =` after the ident IS a mutation.
        let matched = captures.as_str();
        if matched.contains('.') || matched.contains('[') {
            return true;
        }
    }

    let method_mutate_re = Regex::new(&format!(
        r"\b{}\.(?:push|push_back|pop|pop_front|pop_back|insert|entry|sort|sort_by|reverse|clear|remove|retain|extend|swap|push_str|truncate|dedup|fill|drain|resize|reserve|shrink_to_fit|splice)\s*\(",
        escaped
    ))
    .unwrap();
    method_mutate_re.is_match(code)
}

fn remove_unnecessary_mut(code: &str) -> (String, usize) {
    let mut decl_counts: HashMap<String, usize> = HashMap::new();
    for captures in LET_MUT_DECL_REGEX.captures_iter(code) {
        let name = captures.get(1).unwrap().as_str().to_string();
        *decl_counts.entry(name).or_insert(0) += 1;
    }

    let mut non_mutated: HashSet<String> = HashSet::new();
    for (name, count) in &decl_counts {
        // Conservatively skip shadowed/repeated names unless handled by rustc JSON diagnostics
        if *count == 1 && !is_variable_mutated(code, name) {
            non_mutated.insert(name.clone());
        }
    }

    if non_mutated.is_empty() {
        return (code.to_string(), 0);
    }

    let mut removed = 0usize;
    let updated = LET_MUT_DECL_REGEX
        .replace_all(code, |captures: &regex::Captures| {
            let name = captures.get(1).unwrap().as_str();
            if non_mutated.contains(name) {
                removed += 1;
                format!("let {}", name)
            } else {
                captures.get(0).unwrap().as_str().to_string()
            }
        })
        .to_string();

    (updated, removed)
}

fn apply_rustc_warning_cleanups(
    code: &str,
    diagnostics: &[RustcDiagnosticMessage],
) -> (String, usize, bool) {
    let mut lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();
    let mut mut_removed = 0usize;
    let mut parens_cleaned = false;

    for diag in diagnostics {
        let warning_code = diag.code.as_ref().map(|c| c.code.as_str()).unwrap_or("");
        if warning_code == "unused_mut" {
            if let Some(spans) = &diag.spans {
                for span in spans {
                    if span.line_start == span.line_end && span.line_start >= 1 {
                        if let Some(line) = lines.get_mut(span.line_start - 1) {
                            if line.contains("mut ") {
                                *line = line.replacen("mut ", "", 1);
                                mut_removed += 1;
                            }
                        }
                    }
                }
            }
        } else if warning_code == "unused_parens" {
            if let Some(spans) = &diag.spans {
                for span in spans {
                    if span.line_start == span.line_end
                        && span.line_start >= 1
                        && span.suggested_replacement.as_deref() == Some("")
                    {
                        if let Some(line) = lines.get_mut(span.line_start - 1) {
                            let start = span.column_start.saturating_sub(1);
                            let end = span.column_end.saturating_sub(1);
                            if end == start + 1 && end <= line.len() {
                                if line.as_bytes()[start] == b'(' || line.as_bytes()[start] == b')' {
                                    // Avoid mutating offsets within the same line when multiple spans share one line
                                    parens_cleaned = true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = parens_cleaned;
    (lines.join("\n"), mut_removed, mut_removed > 0)
}

pub fn verify_or_rollback_transformation(
    faithful_rust: &str,
    candidate_rust: &str,
    pass_name: &str,
    rejected_passes: &mut Vec<String>,
) -> (String, bool) {
    match check_rustc_and_collect_diagnostics(candidate_rust) {
        Ok(_) => (candidate_rust.to_string(), true),
        Err(reason) => {
            rejected_passes.push(format!("{} (rejected: {})", pass_name, reason));
            (faithful_rust.to_string(), false)
        }
    }
}

pub fn postprocess_rust_code(faithful_rust: &str) -> (String, PostProcessingSummary) {
    let mut summary = PostProcessingSummary::default();
    if faithful_rust.contains("// [Unsupported C/C++ syntax in regex fallback]:") {
        let reason =
            "Source contains unsupported C/C++ constructs that the regex fallback cannot translate."
                .to_string();
        summary.compiler_verified = false;
        summary.verification_error = Some(reason.clone());
        summary
            .rejected_passes
            .push(format!("idiomatic_cleanup ({})", reason));
        return (faithful_rust.to_string(), summary);
    }

    let raw_lines: Vec<String> = faithful_rust.lines().map(|l| l.to_string()).collect();
    let code_only = recompute_code_only_lines(&raw_lines).join("\n");
    if faithful_rust.contains("// [Preprocessor] #")
        || code_only.matches('{').count() != code_only.matches('}').count()
    {
        summary.compiler_verified = false;
        summary.verification_error = Some(
            "Source has syntax or preprocessor errors; preserved raw output.".to_string(),
        );
        summary.rejected_passes.push(
            "idiomatic_cleanup (source has syntax/preprocessor errors; preserved Faithful Rust)"
                .to_string(),
        );
        return (faithful_rust.to_string(), summary);
    }

    let mut candidate = faithful_rust.to_string();

    let (cpp2rust_simplified, cpp2rust_changed) = simplify_cpp2rust_runtime_scaffolding(&candidate);
    if cpp2rust_changed {
        let (verified, accepted) = verify_or_rollback_transformation(
            faithful_rust,
            &cpp2rust_simplified,
            "simplify_cpp2rust_runtime_scaffolding",
            &mut summary.rejected_passes,
        );
        if accepted {
            candidate = verified;
            summary
                .applied_passes
                .push("simplify_cpp2rust_runtime_scaffolding".to_string());
        }
    }

    let (class_normalized, class_changed) = normalize_cpp2rust_class_translation(&candidate);
    if class_changed {
        candidate = class_normalized;
        summary
            .applied_passes
            .push("normalize_cpp2rust_class_translation".to_string());
    }

    let (abi_normalized, abi_changed) = normalize_c_abi_translation(&candidate);
    if abi_changed {
        candidate = abi_normalized;
        summary
            .applied_passes
            .push("normalize_c_abi_translation".to_string());
    }

    let (relaxed, relaxed_changed) = relax_unnecessary_unsafe_functions(&candidate);
    if relaxed_changed {
        let (verified, accepted) = verify_or_rollback_transformation(
            &candidate,
            &relaxed,
            "relax_unnecessary_unsafe_functions",
            &mut summary.rejected_passes,
        );
        if accepted {
            candidate = verified;
            summary
                .applied_passes
                .push("relax_unnecessary_unsafe_functions".to_string());
        }
    }

    let (external_out, external_passes) = try_external_c2rust_tools(&candidate);
    if !external_passes.is_empty() {
        candidate = external_out;
        summary.applied_passes.extend(external_passes);
    }

    let (stripped, stripped_changed) = strip_transpiler_boilerplate(&candidate);
    if stripped_changed {
        candidate = stripped;
        summary
            .applied_passes
            .push("strip_transpiler_boilerplate".to_string());
    }

    let (typed, unsafe_removed, typed_changed) = replace_c_types_and_unsafe_wrappers(&candidate);
    if typed_changed || unsafe_removed > 0 {
        candidate = typed;
        summary.unsafe_blocks_removed += unsafe_removed;
        summary
            .applied_passes
            .push("idiomatic_types_and_safe_abstractions".to_string());
    }

    let (box_simplified, box_changed) = simplify_idiomatic_box_and_null_checks(&candidate);
    if box_changed {
        candidate = box_simplified;
        summary
            .applied_passes
            .push("simplify_box_and_null_checks".to_string());
    }

    let (cast_cleaned, cast_removed, cast_changed) = remove_redundant_casts_and_parens(&candidate);
    if cast_changed || cast_removed > 0 {
        candidate = cast_cleaned;
        summary.redundant_casts_removed += cast_removed;
        summary
            .applied_passes
            .push("remove_redundant_casts_and_parens".to_string());
    }

    let (mut_cleaned, mut_removed) = remove_unnecessary_mut(&candidate);
    if mut_removed > 0 {
        let (verified, accepted) = verify_or_rollback_transformation(
            &candidate,
            &mut_cleaned,
            "remove_unnecessary_mut",
            &mut summary.rejected_passes,
        );
        if accepted {
            candidate = verified;
            summary.redundant_mut_removed += mut_removed;
            summary
                .applied_passes
                .push("remove_unnecessary_mut".to_string());
        }
    }

    // `unsafe { f() }` becomes `{ f() }` once `f` was relaxed to a safe fn.
    candidate = Regex::new(r"=\s*\{\s*([A-Za-z_][A-Za-z0-9_:]*\([^;{}]*\))\s*\};")
        .unwrap()
        .replace_all(&candidate, "= $1;")
        .to_string();

    if let Some(formatted) = format_with_rustfmt(&candidate) {
        candidate = formatted;
        summary.formatted_with_rustfmt = true;
        summary.applied_passes.push("rustfmt".to_string());
    }

    match check_rustc_and_collect_diagnostics(&candidate) {
        Ok(diagnostics) => {
            let (compiler_cleaned, extra_mut_removed, applied_diag_cleanup) =
                apply_rustc_warning_cleanups(&candidate, &diagnostics);
            if applied_diag_cleanup {
                let re_formatted = format_with_rustfmt(&compiler_cleaned).unwrap_or(compiler_cleaned);
                if check_rustc_and_collect_diagnostics(&re_formatted).is_ok() {
                    candidate = re_formatted;
                    summary.redundant_mut_removed += extra_mut_removed;
                    summary
                        .applied_passes
                        .push("rustc_warning_cleanup".to_string());
                } else {
                    summary
                        .rejected_passes
                        .push("rustc_warning_cleanup".to_string());
                }
            }

            if still_raw_engine_transcription(faithful_rust, &candidate) {
                summary.compiler_verified = false;
                summary.verification_error = Some(
                    "Upstream output is still a raw engine transcription (C ABI/runtime scaffolding); it is available in Generated Rust but was not accepted as Readable Rust."
                        .to_string(),
                );
                summary.rejected_passes.push(
                    "idiomatic_cleanup (upstream runtime/FFI scaffolding remains)".to_string(),
                );
                (faithful_rust.to_string(), summary)
            } else {
                summary.compiler_verified = true;
                (candidate, summary)
            }
        }
        Err(candidate_err) => {
            summary.rejected_passes.push(format!(
                "idiomatic_cleanup (failed rustc verification: {}; reverted to Faithful Rust)",
                candidate_err
            ));
            summary.formatted_with_rustfmt = false;
            if still_raw_engine_transcription(faithful_rust, faithful_rust) {
                summary.compiler_verified = false;
                summary.verification_error = Some(
                    "Upstream output is still a raw engine transcription (C ABI/runtime scaffolding); it is available in Generated Rust but was not accepted as Readable Rust."
                        .to_string(),
                );
                summary.rejected_passes.push(
                    "idiomatic_cleanup (upstream runtime/FFI scaffolding remains)".to_string(),
                );
                return (faithful_rust.to_string(), summary);
            }
            match check_rustc_and_collect_diagnostics(faithful_rust) {
                Ok(_) => {
                    summary.compiler_verified = true;
                    summary.verification_error = None;
                    (faithful_rust.to_string(), summary)
                }
                Err(faithful_err) => {
                    summary.compiler_verified = false;
                    summary.verification_error = Some(faithful_err);
                    (faithful_rust.to_string(), summary)
                }
            }
        }
    }
}

pub async fn verify_equivalence_against_source(
    configuration: &ApplicationConfiguration,
    source_code: &str,
    source_language: &SourceLanguage,
    faithful_rust: &str,
    idiomatic_rust: &str,
    stdin_input: &str,
) -> Result<(), String> {
    let exec_lang = match source_language {
        SourceLanguage::C => ExecutionLanguage::C,
        SourceLanguage::Cpp => ExecutionLanguage::Cpp,
    };

    let source_res = execute_program(
        configuration,
        &ExecutionRequest {
            code: source_code.to_string(),
            language: exec_lang,
            mode: ExecutionMode::Run,
            stdin_input: stdin_input.to_string(),
            provider: Some(ExecutionProvider::Local),
            compiler_standard: None,
        },
    )
    .await;

    let faithful_res = execute_program(
        configuration,
        &ExecutionRequest {
            code: faithful_rust.to_string(),
            language: ExecutionLanguage::Rust,
            mode: ExecutionMode::Run,
            stdin_input: stdin_input.to_string(),
            provider: Some(ExecutionProvider::Local),
            compiler_standard: None,
        },
    )
    .await;

    let idiomatic_res = execute_program(
        configuration,
        &ExecutionRequest {
            code: idiomatic_rust.to_string(),
            language: ExecutionLanguage::Rust,
            mode: ExecutionMode::Run,
            stdin_input: stdin_input.to_string(),
            provider: Some(ExecutionProvider::Local),
            compiler_standard: None,
        },
    )
    .await;

    if !idiomatic_res.success {
        return Err(format!("Idiomatic Rust failed execution: {}", idiomatic_res.error_output));
    }
    if source_res.success && idiomatic_res.output.trim() != source_res.output.trim() {
        return Err(format!(
            "Idiomatic Rust stdout diverged from C/C++ source:\nExpected:\n{}\nActual:\n{}",
            source_res.output, idiomatic_res.output
        ));
    }
    if faithful_res.success && idiomatic_res.output.trim() != faithful_res.output.trim() {
        return Err(format!(
            "Idiomatic Rust stdout diverged from Faithful Rust:\nExpected:\n{}\nActual:\n{}",
            faithful_res.output, idiomatic_res.output
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_multiline_and_raw_strings_custom_drops_and_drop_order() {
        let code_with_multiline_string = "fn main() {\n    let s = \"hello\nif true {\nworld\n}\";\n    println!(\"{}\", s);\n}";
        let (out1, changed1) = simplify_idiomatic_box_and_null_checks(code_with_multiline_string);
        assert!(!changed1);
        assert!(out1.contains("if true {\nworld\n}"));

        let code_with_raw_string = "fn main() {\n    let r = r#\"line\nif true {\n\"foo\"\n}\"#;\n    println!(\"{}\", r);\n}";
        let (out2, changed2) = simplify_idiomatic_box_and_null_checks(code_with_raw_string);
        assert!(!changed2);
        assert!(out2.contains("if true {\n\"foo\"\n}"));

        let code_custom_box = "fn main() {\n    let mut s: Box<Custom> = Box::new(Default::default());\n    *s = Custom;\n    let mut a: Box<i32> = Box::new(Default::default());\n    *a = 1;\n    let b: Box<i32> = Box::new(2);\n    println!(\"{}\", *b);\n    drop(a);\n}";
        let (out3, _) = simplify_idiomatic_box_and_null_checks(code_custom_box);
        assert!(out3.contains("let mut s: Box<Custom> = Box::new(Default::default());"));
        assert!(out3.contains("*s = Custom;"));
        assert!(out3.contains("let mut a: Box<i32> = Box::new(1);"));
        assert!(out3.contains("drop(a);"));
    }

    #[test]
    fn simplifies_cpp2rust_runtime_entrypoint_and_stdout() {
        let faithful = r#"extern crate libcc2rs;
use libcc2rs::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::io::{Read, Seek, Write};
use std::os::fd::AsFd;
use std::rc::{Rc, Weak};
pub fn main() {
    __cpp2rust_init_globals();
    std::process::exit(main_0());
}
fn main_0() -> i32 {
    let min: i32 = 100;
    let max: i32 = 1048575;
    if min > max {
        println!("Error in Code Runtime");
        return -1_i32;
    }
    {
        let mut __bytes = Ptr::<i8>::from_string_literal(b"Workers going on holoo").to_c_u8_bytes();
        __bytes.push(b'\n');
        match libcc2rs::c_stdout().with_mut(|__f| __f.write(&__bytes)) == __bytes.len() {
            true => 0,
            false => -1,
        }
    };
    return 0;
}
pub fn __cpp2rust_init_globals() {}"#;

        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert!(summary.compiler_verified, "{:?}", summary.verification_error);
        assert!(summary
            .applied_passes
            .iter()
            .any(|pass| pass == "simplify_cpp2rust_runtime_scaffolding"));
        assert!(!idiomatic.contains("libcc2rs"), "{idiomatic}");
        assert!(!idiomatic.contains("Ptr::<i8>"));
        assert!(!idiomatic.contains("main_0"));
        assert!(idiomatic.contains("fn main()"));
        assert!(idiomatic.contains("println!(\"Workers going on holoo\");"));
        assert!(idiomatic.contains("std::process::exit(-1);"));
    }

    #[test]
    fn rejects_cpp2rust_runtime_scaffolding_as_idiomatic() {
        let faithful = r#"extern crate libcc2rs;
use libcc2rs::*;
fn main_0() -> i32 {
    let mut __bytes = Ptr::<i8>::from_string_literal(b"hi").to_c_u8_bytes();
    __bytes.push(b'\n');
    match libcc2rs::c_stdout().with_mut(|__f| __f.write(&__bytes)) == __bytes.len() {
        true => 0,
        false => -1,
    }
}
pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}"#;

        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert_eq!(idiomatic, faithful);
        assert!(!summary.compiler_verified);
        assert!(summary
            .verification_error
            .as_deref()
            .unwrap_or_default()
            .contains("raw engine transcription"));
    }

    #[test]
    fn lowers_cpp2rust_class_translation_to_owned_struct_and_impl() {
        let faithful = r#"extern crate libcc2rs;
use libcc2rs::*;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
#[derive(Clone, Record, ByteRepr, VaArg, FnPtrArg, CppFrom, Default)]
#[cpp_from(i32, new, value)]
#[byte_size(8)]
pub struct Person {
    #[offset(0)]
    id: i32,
    #[offset(4)]
    score: f32,
}
impl Person {
    pub fn new(mut person_id: i32) -> Self {
        Self { id: person_id, score: 0_f32 }
    }
}
pub fn main() {
    __cpp2rust_init_globals();
    std::process::exit(main_0());
}
fn main_0() -> i32 {
    let person1: Value<Person> = Rc::new(RefCell::new(Person::new({ 1 })));
    ({ PersonImpl::set_score(&person1.as_pointer(), 12_f32) });
    write!(
        libcc2rs::cout(),
        "score {:}\n",
        ({ PersonImpl::get_score(&person1.as_pointer(),) }),
    );
    return 0;
}
pub trait PersonImpl {
    fn set_score(&self, score: f32);
    fn get_score(&self) -> f32;
}
impl PersonImpl for Ptr<Person> {
    fn set_score(&self, mut score: f32) {
        if ((*self).with(|__s| __s.id) > 0) {
            field!((*self), score).write(score);
        }
    }
    fn get_score(&self) -> f32 {
        if ((*self).with(|__s| __s.id) > 0) {
            return (*self).with(|__s| __s.score);
        }
        return -1_f32;
    }
}
pub fn __cpp2rust_init_globals() {}"#;

        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert!(summary.compiler_verified, "{:?}", summary.verification_error);
        assert!(summary
            .applied_passes
            .iter()
            .any(|pass| pass == "normalize_cpp2rust_class_translation"));
        assert!(!idiomatic.contains("libcc2rs"));
        assert!(!idiomatic.contains("unsafe"));
        assert!(!idiomatic.contains("extern \"C\""));
        assert!(!idiomatic.contains("Value<"));
        assert!(!idiomatic.contains("Ptr<"));
        assert!(!idiomatic.contains("field!("));
        assert!(idiomatic.contains("pub struct Person"));
        assert!(idiomatic.contains("impl Person {"));
        assert!(idiomatic.contains("fn set_score(&mut self"));
        assert!(idiomatic.contains("fn get_score(&self"));
        assert!(idiomatic.contains("person1.set_score(12_f32);"));
        assert!(idiomatic.contains("println!(\"score"));
        assert!(idiomatic.contains("#[derive(Clone, Default)]") || idiomatic.contains("#[derive(Clone)]"));
    }

    #[test]
    fn lowers_puts_literal_and_fixed_char_buffer_io_to_safe_rust() {
        let faithful = r#"extern "C" {
    fn puts(_: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn gets(_: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
}
unsafe fn main_0() -> ::core::ffi::c_int {
    let mut data: [::core::ffi::c_char; 10] = [0; 10];
    puts(b"Enter Data:\n\0" as *const u8 as *const ::core::ffi::c_char);
    gets(&raw mut data as *mut ::core::ffi::c_char);
    puts(&raw mut data as *mut ::core::ffi::c_char);
    return 0 as ::core::ffi::c_int;
}
pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}"#;

        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert!(summary.compiler_verified, "{:?}", summary.verification_error);
        assert!(!idiomatic.contains("unsafe"));
        assert!(!idiomatic.contains("extern \"C\""));
        assert!(!idiomatic.contains("gets"));
        assert!(!idiomatic.contains("puts"));
        assert!(idiomatic.contains("let mut data: [u8; 10] = [0u8; 10];"));
        assert!(idiomatic.contains(".take(data.len() as u64)"));
        assert!(idiomatic.contains("String::from_utf8_lossy(&data[..__read_data])"));
        assert!(idiomatic.contains("println!(\"Enter Data:\")"));
        assert!(idiomatic.contains("fn main_0() -> i32"));
    }

    #[test]
    fn lowers_malloc_free_array_into_owned_vec() {
        let faithful = r#"extern "C" {
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(_: *mut ::core::ffi::c_void);
}
pub type size_t = usize;
unsafe fn main_0() -> ::core::ffi::c_int {
    let mut arr: *mut ::core::ffi::c_int = malloc(
        (::core::mem::size_of::<::core::ffi::c_int>() as size_t).wrapping_mul(4 as size_t),
    ) as *mut ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        *arr.offset(i as isize) = i + 10 as ::core::ffi::c_int;
        i += 1;
    }
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j < 4 as ::core::ffi::c_int {
        printf(b"%d\0" as *const u8 as *const ::core::ffi::c_char, *arr.offset(j as isize));
        j += 1;
    }
    free(arr as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}"#;

        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert!(summary.compiler_verified, "{:?}", summary.verification_error);
        assert!(!idiomatic.contains("unsafe"));
        assert!(!idiomatic.contains("extern \"C\""));
        assert!(!idiomatic.contains("malloc"));
        assert!(!idiomatic.contains("free("));
        assert!(idiomatic.contains("let mut arr: Vec<i32> = vec![Default::default(); 4];"));
        assert!(idiomatic.contains("arr[i as usize] = i + 10i32;"));
    }

    #[test]
    fn refuses_idiomatic_output_that_still_needs_unsafe_or_c_interop() {
        // `qsort` needs an extern "C" comparator, so no safe Rust projection exists.
        let faithful = r#"extern "C" {
    fn qsort(__base: *mut ::core::ffi::c_void, __nmemb: usize, __size: usize, __compar: unsafe extern "C" fn(_: *const ::core::ffi::c_void, _: *const ::core::ffi::c_void) -> ::core::ffi::c_int);
}
unsafe extern "C" fn cmp(a: *const ::core::ffi::c_void, b: *const ::core::ffi::c_void) -> ::core::ffi::c_int {
    (*(a as *const ::core::ffi::c_int)).cmp(&*(b as *const ::core::ffi::c_int)) as ::core::ffi::c_int
}
unsafe fn main_0() -> ::core::ffi::c_int {
    let mut values: [::core::ffi::c_int; 3] = [3, 1, 2];
    qsort(&raw mut values as *mut ::core::ffi::c_void, 3, 4, cmp);
    return 0;
}
pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}"#;

        let (_, summary) = postprocess_rust_code(faithful);
        assert!(!summary.compiler_verified);
        assert!(summary
            .verification_error
            .as_deref()
            .unwrap_or_default()
            .contains("raw engine transcription"));
    }

    #[test]
    fn normalizes_c_abi_types_entrypoint_and_printf_for_any_c_translation() {
        let faithful = r#"extern "C" {
    fn printf(_: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
}
unsafe fn main_0() -> ::core::ffi::c_int {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while idx < 3 as ::core::ffi::c_int {
        printf(
            b"value=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            idx,
        );
        idx += 1;
    }
    return 0 as ::core::ffi::c_int;
}
pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}"#;

        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert!(summary.compiler_verified, "{:?}", summary.verification_error);
        assert!(summary
            .applied_passes
            .iter()
            .any(|pass| pass == "normalize_c_abi_translation"));
        assert!(!idiomatic.contains("::core::ffi"));
        assert!(!idiomatic.contains("printf"));
        assert!(idiomatic.contains("println!(\"value={}\""));
        assert!(idiomatic.contains("fn main_0() -> i32"));
        assert!(idiomatic.contains("fn main()"));
        assert!(idiomatic.contains("let exit_code = main_0();"));
    }

    #[test]
    fn keeps_unlowerable_printf_in_faithful_and_refuses_it_as_idiomatic() {
        let faithful = r#"extern "C" {
    fn printf(_: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
}
unsafe fn main_0() -> ::core::ffi::c_int {
    printf(b"%s\0" as *const u8 as *const ::core::ffi::c_char, b"hi\0" as *const u8);
    return 0;
}
pub fn main() {
    unsafe { ::std::process::exit(main_0() as i32) }
}"#;

        // `%s` needs a C string operand, so no safe Rust projection exists and the
        // raw transcription stays in Faithful only.
        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert!(!summary.compiler_verified);
        assert!(summary
            .verification_error
            .as_deref()
            .unwrap_or_default()
            .contains("raw engine transcription"));
        assert!(idiomatic.contains("printf("));
        assert!(!idiomatic.contains("println!(\"%s\""));
    }

    #[test]
    fn prunes_unused_c_ffi_declarations_only() {
        let code = r#"extern "C" {
    fn printf(_: *const i8, ...) -> i32;
    fn puts(_: *const i8) -> i32;
}
fn main() {
    printf(b"%d\0" as *const u8 as *const i8, 1);
}"#;

        let (pruned, changed) = prune_unused_c_ffi_blocks(code);
        assert!(changed);
        assert!(!pruned.contains("fn puts"));
        assert!(pruned.contains("fn printf"));
    }

    #[test]
    fn lowers_supported_printf_format_specifiers() {
        let (format, println) = c_printf_format_to_rust(r"count=%d ratio=%.2f\n\0").unwrap();
        assert_eq!(format, "count={} ratio={:.2}");
        assert!(println);

        let (format, println) = c_printf_format_to_rust(r"%u items\0").unwrap();
        assert_eq!(format, "{} items");
        assert!(!println);

        assert!(c_printf_format_to_rust(r"%s\0").is_none());
        assert!(c_printf_format_to_rust(r"%p\0").is_none());
        assert!(c_printf_format_to_rust(r"%5d\0").is_none());
    }
}




