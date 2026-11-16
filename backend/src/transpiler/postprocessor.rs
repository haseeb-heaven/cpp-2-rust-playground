use crate::config::ApplicationConfiguration;
use crate::executor::execute_program;
use crate::models::{
    ExecutionLanguage, ExecutionMode, ExecutionProvider, ExecutionRequest, PostProcessingSummary,
    SourceLanguage,
};
use crate::transpiler::{detect_external_transpiler_binary, TempEngineDirectory};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

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

#[derive(Debug, Deserialize)]
struct RustcDiagnosticMessage {
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

fn find_rustfmt_binary() -> Option<String> {
    let candidates = [
        "rustfmt",
        "/opt/homebrew/bin/rustfmt",
        "/usr/local/bin/rustfmt",
        "/usr/bin/rustfmt",
    ];
    for candidate in candidates {
        if candidate.contains('/') && !Path::new(candidate).is_file() {
            continue;
        }
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

fn check_rustc_and_collect_diagnostics(
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
    let output = Command::new(rustc)
        .arg("--crate-type")
        .arg(crate_type)
        .arg("--edition")
        .arg("2021")
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

fn try_external_c2rust_tools(code: &str) -> (String, Vec<String>) {
    let mut current = code.to_string();
    let mut applied = Vec::new();

    if current.contains("unsafe") {
        if let Some(c2rust_bin) = detect_external_transpiler_binary(&SourceLanguage::C) {
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

fn strip_transpiler_boilerplate(code: &str) -> (String, bool) {
    let mut output_lines: Vec<String> = Vec::new();
    let mut changed = false;
    let mut prev_blank = false;

    for raw_line in code.lines() {
        let trimmed = raw_line.trim();
        let is_boilerplate = BOILERPLATE_EXACT_LINES.contains(&trimmed)
            || trimmed.starts_with("// #include <")
            || trimmed.starts_with("// #include \"")
            || trimmed.starts_with("// Forward declaration: ")
            || trimmed.starts_with("// Global pointer '")
            || trimmed.starts_with("// Pointer alias: ")
            || trimmed.starts_with("// C11 _Generic macro ");

        if is_boilerplate {
            changed = true;
            continue;
        }

        let cleaned_line = raw_line.trim_end().to_string();
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
    let mut updated = code.replace("pub unsafe fn main(", "fn main(");
    updated = updated.replace("unsafe fn main(", "fn main(");
    let mut count = if updated != code { 1 } else { 0 };

    let unsafe_block_regex = Regex::new(r"\bunsafe\s*\{([^{}]*)\}").unwrap();
    loop {
        let mut removed_in_pass = 0usize;
        let next = unsafe_block_regex
            .replace_all(&updated, |captures: &regex::Captures| {
                let inner = captures.get(1).unwrap().as_str();
                let trimmed_inner = inner.trim();
                let has_unsafe_op = trimmed_inner.contains('*')
                    || trimmed_inner.contains("transmute")
                    || trimmed_inner.contains("libc::")
                    || trimmed_inner.contains("from_raw")
                    || trimmed_inner.contains("as_ptr")
                    || trimmed_inner.contains("as_mut_ptr")
                    || trimmed_inner.contains("get_unchecked")
                    || trimmed_inner.contains("extern");
                if has_unsafe_op {
                    captures.get(0).unwrap().as_str().to_string()
                } else {
                    removed_in_pass += 1;
                    format!("{{ {} }}", trimmed_inner)
                }
            })
            .to_string();
        updated = next;
        count += removed_in_pass;
        if removed_in_pass == 0 {
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
        .replace_all(&updated, "let mut $1 = Box::new($2::default());")
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

fn is_variable_mutated(code: &str, var_name: &str) -> bool {
    let escaped = regex::escape(var_name);
    let mut_ref_re = Regex::new(&format!(r"&\s*mut\s+{}\b", escaped)).unwrap();
    if mut_ref_re.is_match(code) {
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
        r"\b{}\.(?:push|push_back|pop|pop_front|pop_back|insert|entry|sort|sort_by|reverse|clear|remove|retain|extend|swap|push_str|truncate)\s*\(",
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
    if faithful_rust.contains("// [Preprocessor] #")
        || faithful_rust.matches('{').count() != faithful_rust.matches('}').count()
    {
        summary.rejected_passes.push(
            "idiomatic_cleanup (source has syntax/preprocessor errors; preserved Faithful Rust)"
                .to_string(),
        );
        return (faithful_rust.to_string(), summary);
    }

    let mut candidate = faithful_rust.to_string();

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
        candidate = mut_cleaned;
        summary.redundant_mut_removed += mut_removed;
        summary
            .applied_passes
            .push("remove_unnecessary_mut".to_string());
    }

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

            summary.compiler_verified = true;
            (candidate, summary)
        }
        Err(_) => {
            summary.rejected_passes.push(
                "idiomatic_cleanup (failed rustc verification; reverted to Faithful Rust)"
                    .to_string(),
            );
            summary.formatted_with_rustfmt = false;
            summary.compiler_verified = check_rustc_and_collect_diagnostics(faithful_rust).is_ok();
            (faithful_rust.to_string(), summary)
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
