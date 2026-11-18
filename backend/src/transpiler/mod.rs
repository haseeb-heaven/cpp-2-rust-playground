pub mod clippy;
pub mod diagnostics;
pub mod explanations;
pub mod line_translator;
pub mod postprocessor;

use crate::models::{
    ConversionDiagnostic, ConversionResponse, DiagnosticSeverity, ErrorCategory, LineMapping,
    RustOutputStyle, SourceLanguage,
};
use diagnostics::{
    analyze_base_source_diagnostics, analyze_source_diagnostics, detect_unsupported_constructs,
};
pub use explanations::{
    format_explanations_as_rust_comments, generate_conversion_explanations,
};
use line_translator::{translate_single_line, TranslatorState};
use once_cell::sync::Lazy;
use postprocessor::postprocess_rust_code;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

static ENGINE_TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) struct TempEngineDirectory {
    pub(crate) path: PathBuf,
}

impl TempEngineDirectory {
    pub(crate) fn create() -> std::io::Result<Self> {
        let id = ENGINE_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let raw_path = std::env::temp_dir().join(format!(
            "cpp2rust_playground_engine_{}_{}",
            std::process::id(),
            id
        ));
        fs::create_dir_all(&raw_path)?;
        let path = fs::canonicalize(&raw_path).unwrap_or(raw_path);
        Ok(Self { path })
    }
}

impl Drop for TempEngineDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn check_transpiler_binary(candidate: &str, check_flag: &str) -> bool {
    if candidate.contains('/') {
        return Path::new(candidate).is_file();
    }
    Command::new(candidate)
        .arg(check_flag)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn workspace_root_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

pub fn detect_external_transpiler_binary(source_language: &SourceLanguage) -> Option<String> {
    let root = workspace_root_dir();
    let (env_key, check_flag, candidates) = match source_language {
        SourceLanguage::C => {
            let mut list = vec![
                root.join(".tools/bin/c2rust-transpile")
                    .to_string_lossy()
                    .to_string(),
                root.join(".tools/c2rust/0.22.1_1/bin/c2rust-transpile")
                    .to_string_lossy()
                    .to_string(),
                root.join(".tools/bin/c2rust").to_string_lossy().to_string(),
                ".tools/bin/c2rust-transpile".to_string(),
                ".tools/bin/c2rust".to_string(),
                "/opt/homebrew/bin/c2rust-transpile".to_string(),
                "/opt/homebrew/bin/c2rust".to_string(),
                "/usr/local/bin/c2rust-transpile".to_string(),
                "/usr/local/bin/c2rust".to_string(),
                "c2rust-transpile".to_string(),
                "c2rust".to_string(),
            ];
            if let Ok(home) = std::env::var("HOME") {
                if !home.is_empty() {
                    list.insert(0, format!("{}/.cargo/bin/c2rust-transpile", home));
                    list.insert(1, format!("{}/.cargo/bin/c2rust", home));
                }
            }
            ("C2RUST_BIN", "--version", list)
        }
        SourceLanguage::Cpp => (
            "CPP2RUST_BIN",
            "--help",
            vec![
                root.join(".tools/cpp2rust/build/cpp2rust/cpp2rust")
                    .to_string_lossy()
                    .to_string(),
                root.join(".tools/bin/cpp2rust")
                    .to_string_lossy()
                    .to_string(),
                ".tools/cpp2rust/build/cpp2rust/cpp2rust".to_string(),
                ".tools/bin/cpp2rust".to_string(),
                "/opt/homebrew/bin/cpp2rust".to_string(),
                "/usr/local/bin/cpp2rust".to_string(),
                "cpp2rust".to_string(),
            ],
        ),
    };

    if let Ok(explicit_path) = std::env::var(env_key) {
        let trimmed = explicit_path.trim().to_string();
        if !trimmed.is_empty() && check_transpiler_binary(&trimmed, check_flag) {
            return Some(trimmed);
        }
    }

    candidates
        .into_iter()
        .find(|candidate| check_transpiler_binary(candidate, check_flag))
}

fn macos_sdk_flag() -> String {
    if cfg!(target_os = "macos") {
        let sdk = "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk";
        if Path::new(sdk).exists() {
            return format!(" -isysroot {}", sdk);
        }
    }
    String::new()
}

fn clang_builtin_include_flag() -> String {
    let candidates = [
        "/opt/homebrew/Cellar/llvm/23.1.3_1/lib/clang/23/include",
        "/opt/homebrew/opt/llvm/lib/clang/23/include",
        "/opt/homebrew/lib/clang/23/include",
        "/usr/local/opt/llvm/lib/clang/23/include",
        "/Library/Developer/CommandLineTools/usr/lib/clang/16/include",
        "/Library/Developer/CommandLineTools/usr/lib/clang/15.0.0/include",
    ];
    for dir in &candidates {
        if Path::new(dir).is_dir() {
            return format!(" -isystem {}", dir);
        }
    }
    String::new()
}

enum CCodeSpan<'a> {
    Code(&'a str),
    NonCode(&'a str),
}

fn split_c_code_and_literals(source: &str) -> Vec<CCodeSpan<'_>> {
    let mut spans = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut code_start = 0;

    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            if i > code_start {
                spans.push(CCodeSpan::Code(&source[code_start..i]));
            }
            let start = i;
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            spans.push(CCodeSpan::NonCode(&source[start..i]));
            code_start = i;
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            if i > code_start {
                spans.push(CCodeSpan::Code(&source[code_start..i]));
            }
            let start = i;
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < bytes.len() {
                i += 2;
            } else {
                i = bytes.len();
            }
            spans.push(CCodeSpan::NonCode(&source[start..i]));
            code_start = i;
        } else if bytes[i] == b'"' {
            if i > code_start {
                spans.push(CCodeSpan::Code(&source[code_start..i]));
            }
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < bytes.len() {
                i += 1;
            }
            spans.push(CCodeSpan::NonCode(&source[start..i]));
            code_start = i;
        } else if bytes[i] == b'\'' {
            if i > code_start {
                spans.push(CCodeSpan::Code(&source[code_start..i]));
            }
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i] != b'\'' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < bytes.len() {
                i += 1;
            }
            spans.push(CCodeSpan::NonCode(&source[start..i]));
            code_start = i;
        } else {
            i += 1;
        }
    }

    if code_start < bytes.len() {
        spans.push(CCodeSpan::Code(&source[code_start..]));
    }

    spans
}

fn sanitize_c_code_chunk(chunk: &str, all_code: &str) -> String {
    static STRUCT_DEF_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?s)\b(typedef\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{([^}]*)\}\s*([A-Za-z_][A-Za-z0-9_]*)?",
        )
        .unwrap()
    });
    static FIELD_TYPE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(\bstruct\s+)?\b([A-Za-z_][A-Za-z0-9_]*)\s*\*\s*([A-Za-z_][A-Za-z0-9_]*)").unwrap()
    });

    STRUCT_DEF_REGEX
        .replace_all(chunk, |caps: &regex::Captures| {
            let is_typedef = caps.get(1).is_some();
            let typedef_kw = caps.get(1).map_or("", |m| m.as_str());
            let tag = caps.get(2).map_or("", |m| m.as_str());
            let body = caps.get(3).map_or("", |m| m.as_str());
            let trailing_id = caps.get(4).map_or("", |m| m.as_str());
            let alias = if is_typedef { trailing_id } else { "" };

            let mut sanitized_body = body.to_string();
            if !tag.is_empty() {
                sanitized_body = FIELD_TYPE_REGEX
                    .replace_all(&sanitized_body, |fcaps: &regex::Captures| {
                        let has_struct_kw = fcaps.get(1).is_some();
                        let type_name = fcaps.get(2).map_or("", |m| m.as_str());
                        let field_name = fcaps.get(3).map_or("", |m| m.as_str());
                        let is_self_ref = if has_struct_kw {
                            type_name == tag
                        } else {
                            type_name == tag || (!alias.is_empty() && type_name == alias)
                        };

                        if is_self_ref {
                            // If the field is accessed with chained member access (`->field->`, `->field.`, etc.),
                            // pointer arithmetic (`->field +`, `->field -`, `->field[`, etc.),
                            // sizeof expressions (`sizeof(*node->field)`, `sizeof(node->field)`, etc.),
                            // or _Generic selection (`_Generic(n->field, ...)`),
                            // converting it to `void *` would cause compile errors or alter sizeof/arithmetic/_Generic semantics.
                            // In that case, preserve the exact recursive struct pointer type!
                            let field_esc = regex::escape(field_name);
                            if let Ok(preservation_re) = Regex::new(&format!(
                                r"(?:->|\.)\s*{}(?:\s*(?:->|\.|\+|\[|\-|\+\+|\-\-))|(?:(?:\+\+|\-\-|\+)\s*(?:[A-Za-z_][A-Za-z0-9_]*(?:->|\.))\s*{})|\bsizeof\s*\([^)]*{}\b|\b_Generic\s*\([^)]*{}\b",
                                field_esc, field_esc, field_esc, field_esc
                            )) {
                                if preservation_re.is_match(all_code) {
                                    return fcaps.get(0).unwrap().as_str().to_string();
                                }
                            }
                            format!("void *{}", field_name)
                        } else {
                            fcaps.get(0).unwrap().as_str().to_string()
                        }
                    })
                    .to_string();
            }

            let trailing_suffix = if trailing_id.is_empty() {
                String::new()
            } else {
                format!(" {}", trailing_id)
            };
            format!("{typedef_kw}struct {tag} {{{sanitized_body}}}{trailing_suffix}")
        })
        .to_string()
}

pub fn sanitize_c_source_for_c2rust(source: &str) -> String {
    let spans = split_c_code_and_literals(source);
    let mut all_code_spans = String::new();
    for span in &spans {
        if let CCodeSpan::Code(code) = span {
            all_code_spans.push_str(code);
            all_code_spans.push(' ');
        }
    }
    let mut result = String::with_capacity(source.len());
    for span in spans {
        match span {
            CCodeSpan::Code(code) => result.push_str(&sanitize_c_code_chunk(code, &all_code_spans)),
            CCodeSpan::NonCode(literal_or_comment) => result.push_str(literal_or_comment),
        }
    }
    result
}

fn clean_c2rust_generated_code(code: &str) -> String {
    let mut lines = Vec::new();
    for line in code.lines() {
        let trimmed = line.trim();
        // `raw_ref_op` (&raw const / &raw mut) was stabilized in Rust 1.82.
        // Stripping this obsolete feature attribute allows generated code to compile cleanly on stable rustc,
        // while preserving genuinely required feature attributes like `c_variadic`.
        if trimmed == "#![feature(raw_ref_op)]" {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n")
}

fn run_command_with_timeout(
    mut cmd: Command,
    timeout: Duration,
    engine_name: &str,
    temp_dir: &Path,
) -> Result<String, String> {
    let stdout_path = temp_dir.join(format!("{}_stdout.log", engine_name));
    let stderr_path = temp_dir.join(format!("{}_stderr.log", engine_name));

    let stdout_file = fs::File::create(&stdout_path)
        .map_err(|e| format!("failed to create stdout log: {}", e))?;
    let stderr_file = fs::File::create(&stderr_path)
        .map_err(|e| format!("failed to create stderr log: {}", e))?;

    cmd.stdout(Stdio::from(stdout_file));
    cmd.stderr(Stdio::from(stderr_file));

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn {}: {}", engine_name, e))?;

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("external {} CLI timed out", engine_name));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("error waiting on {}: {}", engine_name, e));
            }
        }
    };

    if !status.success() {
        let stderr = fs::read_to_string(&stderr_path)
            .unwrap_or_default()
            .trim()
            .to_string();
        return Err(if stderr.is_empty() {
            format!("exited with status {}", status)
        } else {
            stderr
        });
    }

    Ok(String::new())
}

fn try_invoke_c2rust_transpile(source: &str, binary_path: &str) -> Result<String, String> {
    let temp_dir =
        TempEngineDirectory::create().map_err(|e| format!("failed to create temp dir: {}", e))?;
    let input_file = temp_dir.path.join("input.c");
    let compile_commands_file = temp_dir.path.join("compile_commands.json");
    let output_file_src = temp_dir.path.join("src").join("input.rs");
    let output_file_flat = temp_dir.path.join("input.rs");

    fs::write(&input_file, source).map_err(|e| format!("failed to write input.c: {}", e))?;
    let sdk_arg = macos_sdk_flag();
    let clang_builtin_arg = clang_builtin_include_flag();
    let compile_commands = serde_json::json!([{
        "directory": temp_dir.path.to_string_lossy(),
        "command": format!("clang -std=c11{}{}{} -c {}", sdk_arg, clang_builtin_arg, "", input_file.to_string_lossy()),
        "file": input_file.to_string_lossy(),
    }]);
    let serialized = serde_json::to_string_pretty(&compile_commands)
        .map_err(|e| format!("failed to encode compile_commands.json: {}", e))?;
    fs::write(&compile_commands_file, serialized)
        .map_err(|e| format!("failed to write compile_commands.json: {}", e))?;

    let mut cmd = Command::new(binary_path);
    if binary_path.ends_with("c2rust-transpile") {
        cmd.arg(&compile_commands_file)
            .arg("-o")
            .arg(&temp_dir.path);
    } else {
        cmd.arg("transpile")
            .arg(&compile_commands_file)
            .arg("--output-dir")
            .arg(&temp_dir.path);
    }

    run_command_with_timeout(cmd, Duration::from_millis(10000), "c2rust", &temp_dir.path)?;

    let generated = fs::read_to_string(&output_file_src)
        .or_else(|_| fs::read_to_string(&output_file_flat))
        .map_err(|e| format!("missing generated input.rs: {}", e))?;
    let cleaned = clean_c2rust_generated_code(&generated);
    if cleaned.trim().is_empty() {
        Err("external c2rust CLI produced empty output".to_string())
    } else {
        Ok(cleaned)
    }
}

pub fn try_invoke_external_backbone_engine(
    source_code: &str,
    source_language: &SourceLanguage,
    binary_path: &str,
) -> Result<String, String> {
    match source_language {
        SourceLanguage::C => {
            // 1. Attempt transpilation with original source preserving all recursive pointer types.
            match try_invoke_c2rust_transpile(source_code, binary_path) {
                Ok(rust_code) => Ok(rust_code),
                Err(orig_err) => {
                    // 2. If original source failed or timed out (e.g. c2rust DFExpr infinite recursion on self-referential structs),
                    // attempt sanitized source with cycle-breaking if sanitization produces a different source.
                    let sanitized = sanitize_c_source_for_c2rust(source_code);
                    if sanitized != source_code {
                        try_invoke_c2rust_transpile(&sanitized, binary_path).or(Err(orig_err))
                    } else {
                        Err(orig_err)
                    }
                }
            }
        }
        SourceLanguage::Cpp => {
            let temp_dir = TempEngineDirectory::create()
                .map_err(|e| format!("failed to create temp dir: {}", e))?;
            let input_file = temp_dir.path.join("input.cpp");
            let output_file = temp_dir.path.join("output.rs");

            fs::write(&input_file, source_code)
                .map_err(|e| format!("failed to write input.cpp: {}", e))?;

            let mut cmd = Command::new(binary_path);
            cmd.arg(format!("--file={}", input_file.display()))
                .arg(format!("-o={}", output_file.display()));

            let rules_dir = workspace_root_dir().join(".tools/cpp2rust/build/rules");
            if rules_dir.is_dir() {
                cmd.arg(format!("--rules={}", rules_dir.display()));
            }

            let current_path = std::env::var("PATH").unwrap_or_default();
            cmd.env(
                "PATH",
                format!("/opt/homebrew/bin:/usr/local/bin:{}", current_path),
            );

            run_command_with_timeout(cmd, Duration::from_millis(3000), "cpp2rust", &temp_dir.path)?;

            let generated = fs::read_to_string(&output_file)
                .map_err(|e| format!("missing generated output.rs: {}", e))?;
            if generated.trim().is_empty() {
                Err("external cpp2rust CLI produced empty output".to_string())
            } else {
                Ok(generated)
            }
        }
    }
}

pub fn select_transpiler_engine(source_language: &SourceLanguage) -> String {
    match source_language {
        SourceLanguage::C => {
            if let Some(binary) = detect_external_transpiler_binary(source_language) {
                format!("c2rust CLI ({})", binary)
            } else {
                "Built-in Regex C Fallback (c2rust CLI not installed)".to_string()
            }
        }
        SourceLanguage::Cpp => {
            if let Some(binary) = detect_external_transpiler_binary(source_language) {
                format!("cpp2rust CLI ({})", binary)
            } else {
                "Built-in Regex C++ Fallback (cpp2rust CLI not installed)".to_string()
            }
        }
    }
}

pub fn convert_source_to_rust(
    source_code: &str,
    source_language: &SourceLanguage,
) -> ConversionResponse {
    convert_source_to_rust_with_style(source_code, source_language, RustOutputStyle::Idiomatic)
}

pub fn convert_source_to_rust_with_style(
    source_code: &str,
    source_language: &SourceLanguage,
    output_style: RustOutputStyle,
) -> ConversionResponse {
    let start_instant = Instant::now();
    let source_lines: Vec<&str> = source_code.lines().collect();

    let cli_name = match source_language {
        SourceLanguage::C => "c2rust",
        SourceLanguage::Cpp => "cpp2rust",
    };

    let mut fallback_reason = Some(format!(
        "External {} CLI binary is not installed; used built-in regex-based fallback translator.",
        cli_name
    ));

    let mut diagnostics = analyze_source_diagnostics(&source_lines);
    let (_, unsupported_constructs) = detect_unsupported_constructs(&source_lines);

    let mut translator_state = TranslatorState::new();
    let mut translated_blocks: Vec<(usize, String, String)> = Vec::with_capacity(source_lines.len());

    for (zero_based_index, raw_line) in source_lines.iter().enumerate() {
        let source_line_number = zero_based_index + 1;
        let translated_line = translate_single_line(raw_line, &mut translator_state);
        translated_blocks.push((
            source_line_number,
            (*raw_line).to_string(),
            translated_line,
        ));
    }

    if let Some(pending) = translator_state.pending_multiline_statement.take() {
        let flushed = translate_single_line(&format!("{};", pending), &mut translator_state);
        if !flushed.is_empty() {
            translated_blocks.push((source_lines.len().max(1), pending, flushed));
        }
    }

    let required_imports: Vec<String> = translator_state.required_imports.into_iter().collect();
    let mut final_rust_lines: Vec<String> = Vec::new();
    let mut line_mappings: Vec<LineMapping> = Vec::new();

    if !required_imports.is_empty() {
        for import_path in &required_imports {
            final_rust_lines.push(format!("use {};", import_path));
        }
        final_rust_lines.push(String::new());
    }

    for (source_line_number, source_text, translated_chunk) in translated_blocks {
        let rust_start_line = final_rust_lines.len() + 1;
        if translated_chunk.is_empty() {
            if source_text.trim().is_empty() {
                final_rust_lines.push(String::new());
                let rust_end_line = final_rust_lines.len();
                line_mappings.push(LineMapping {
                    source_line: source_line_number,
                    rust_start_line,
                    rust_end_line,
                    source_text,
                    rust_text: String::new(),
                });
            }
            continue;
        }

        let chunk_lines: Vec<&str> = translated_chunk.lines().collect();
        for chunk_line in &chunk_lines {
            final_rust_lines.push((*chunk_line).to_string());
        }
        let rust_end_line = final_rust_lines.len();
        line_mappings.push(LineMapping {
            source_line: source_line_number,
            rust_start_line,
            rust_end_line,
            source_text,
            rust_text: translated_chunk,
        });
    }

    let mut faithful_rust_code = final_rust_lines.join("\n");
    let (mut idiomatic_rust_code, mut post_processing) = postprocess_rust_code(&faithful_rust_code);
    let readable_fallback_rust_code = idiomatic_rust_code.clone();
    let readable_fallback_post_processing = post_processing.clone();

    let has_base_errors = diagnostics
        .iter()
        .any(|diag| diag.severity == DiagnosticSeverity::Error);

    let fallback_label = match source_language {
        SourceLanguage::C => "Built-in Regex C Fallback (c2rust CLI not executed)".to_string(),
        SourceLanguage::Cpp => {
            "Built-in Regex C++ Fallback (cpp2rust CLI not executed)".to_string()
        }
    };

    let (transpiler_engine, used_fallback) = if let Some(binary_path) = detect_external_transpiler_binary(source_language) {
        let engine_label = format!("{} CLI ({})", cli_name, binary_path);
        match try_invoke_external_backbone_engine(source_code, source_language, &binary_path) {
            Ok(backbone_rust_code) => {
                let (backbone_idiomatic, backbone_post) = postprocess_rust_code(&backbone_rust_code);
                faithful_rust_code = backbone_rust_code;
                if backbone_post.compiler_verified {
                    idiomatic_rust_code = backbone_idiomatic;
                    post_processing = backbone_post;
                    diagnostics = analyze_base_source_diagnostics(&source_lines);
                    fallback_reason = None;
                    (engine_label, false)
                } else if readable_fallback_post_processing.compiler_verified {
                    idiomatic_rust_code = readable_fallback_rust_code;
                    post_processing = readable_fallback_post_processing;
                    diagnostics = analyze_base_source_diagnostics(&source_lines);
                    fallback_reason = Some(format!(
                        "Raw {} CLI output is available in Generated Rust, but it still contains runtime/FFI scaffolding. Readable Rust uses a separately rustc-verified projection.",
                        cli_name
                    ));
                    (
                        format!("{} + Built-in Readable Projection", engine_label),
                        true,
                    )
                } else {
                    idiomatic_rust_code = format!(
                        "// Readable Rust is unavailable for this upstream translation.\n// Generated Rust shows the raw {} output.\n// Reason: {}",
                        cli_name,
                        backbone_post
                            .verification_error
                            .as_deref()
                            .unwrap_or("the output could not be safely humanized")
                    );
                    post_processing = backbone_post;
                    fallback_reason = Some(format!(
                        "External {} CLI produced Generated Rust, but no compiler-verified Readable projection is available.",
                        cli_name
                    ));
                    (engine_label, false)
                }
            }
            Err(err_msg) => {
                let clipped: String = err_msg.chars().take(160).collect();
                fallback_reason = Some(format!(
                    "External {} CLI failed ({}); used built-in regex-based fallback translator.",
                    cli_name, err_msg
                ));
                (
                    format!("Built-in Regex {} Fallback ({} CLI failed: {})",
                        match source_language { SourceLanguage::C => "C", SourceLanguage::Cpp => "C++" },
                        cli_name, clipped),
                    true,
                )
            }
        }
    } else {
        (fallback_label, true)
    };

    if !post_processing.compiler_verified && !source_code.trim().is_empty() {
        if !has_base_errors {
            diagnostics.push(ConversionDiagnostic {
                line_number: 1,
                severity: DiagnosticSeverity::Error,
                category: ErrorCategory::Conversion,
                message: format!(
                    "Translation did not pass rustc verification: {}. The translator cannot verify this C/C++ construct.",
                    post_processing
                        .verification_error
                        .as_deref()
                        .unwrap_or("rustc verification failed")
                ),
                suggested_fix: Some(
                    "Install c2rust/cpp2rust CLI or inspect diagnostics and edit the generated Rust manually."
                        .to_string(),
                ),
            });
        }
        idiomatic_rust_code = format!(
            "// Readable Rust is unavailable for this C/C++ input.\n// {}\n// Generated Rust shows the raw engine/fallback output; it must be reviewed manually before use.",
            fallback_reason
                .as_deref()
                .unwrap_or("The translator could not produce compiler-verified Rust")
        );
    }

    let explanations = generate_conversion_explanations(
        source_code,
        source_language,
        &faithful_rust_code,
        &idiomatic_rust_code,
        &output_style,
        &post_processing,
        used_fallback,
        &transpiler_engine,
    );
    let generated_rust_code = match output_style {
        RustOutputStyle::Faithful => faithful_rust_code.clone(),
        RustOutputStyle::Idiomatic => idiomatic_rust_code.clone(),
    };
    let max_line = generated_rust_code.lines().count().max(1);
    if output_style == RustOutputStyle::Idiomatic {
        for mapping in &mut line_mappings {
            mapping.rust_start_line = mapping.rust_start_line.min(max_line);
            mapping.rust_end_line = mapping.rust_end_line.min(max_line);
        }
    }
    // Lint the code the user actually reads: the verified Readable projection,
    // otherwise the raw Generated output.
    let lint_target = if post_processing.compiler_verified {
        idiomatic_rust_code.clone()
    } else {
        faithful_rust_code.clone()
    };
    let (clippy_lints, clippy_error) = match clippy::run_clippy(&lint_target) {
        Ok(lints) => (lints, None),
        Err(error) => (Vec::new(), Some(error)),
    };
    let conversion_time_milliseconds = start_instant.elapsed().as_millis();

    ConversionResponse {
        generated_rust_code,
        faithful_rust_code,
        idiomatic_rust_code,
        output_style,
        line_mappings,
        diagnostics,
        required_imports,
        conversion_time_milliseconds,
        transpiler_engine,
        used_fallback,
        fallback_reason,
        unsupported_constructs,
        post_processing,
        explanations,
        clippy_lints,
        clippy_available: clippy::clippy_available(),
        clippy_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DiagnosticSeverity;

    fn assert_honest_engine_contract(result: &ConversionResponse, language: &SourceLanguage) {
        if result.used_fallback {
            assert!(result.fallback_reason.is_some());
            assert!(
                result.transpiler_engine.contains("Built-in Regex")
                    || result
                        .transpiler_engine
                        .contains("Built-in Readable Projection")
            );
        } else {
            let expected_engine = match language {
                SourceLanguage::C => "c2rust CLI",
                SourceLanguage::Cpp => "cpp2rust CLI",
            };
            assert!(result.transpiler_engine.contains(expected_engine));
        }
    }

    fn uses_raw_regex_fallback(result: &ConversionResponse) -> bool {
        result
            .transpiler_engine
            .contains("Built-in Regex C")
    }

    #[test]
    fn converts_hello_world_cpp() {
        let cpp_source = r#"#include <iostream>
using namespace std;

int main() {
    cout << "Hello, RustC-Studio!" << endl;
    return 0;
}"#;
        let result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        assert_honest_engine_contract(&result, &SourceLanguage::Cpp);
        assert!(!result.generated_rust_code.trim().is_empty());
        if uses_raw_regex_fallback(&result) {
            assert!(result.generated_rust_code.contains("fn main() {"));
            assert!(result
                .generated_rust_code
                .contains("println!(\"Hello, RustC-Studio!\");"));
        }
        assert!(!result.line_mappings.is_empty());
    }

    #[test]
    fn converts_c_printf_and_for_loop() {
        let c_source = r#"#include <stdio.h>
int main() {
    int sum = 0;
    for (int i = 0; i < 5; i++) {
        sum += i;
    }
    printf("Sum: %d\n", sum);
    return 0;
}"#;
        let result = convert_source_to_rust(c_source, &SourceLanguage::C);
        assert_honest_engine_contract(&result, &SourceLanguage::C);
        if uses_raw_regex_fallback(&result) {
            assert!(result.faithful_rust_code.contains("let mut sum: i32 = 0;"));
            assert!(result.faithful_rust_code.contains("for i in 0..5 {"));
            assert!(result.idiomatic_rust_code.contains("for i in 0..5 {"));
            assert!(result
                .idiomatic_rust_code
                .contains("println!(\"Sum: {}\", sum);"));
        }
    }

    #[test]
    fn converts_vector_and_hashmap_with_imports() {
        let cpp_source = r#"#include <vector>
#include <unordered_map>
int main() {
    std::vector<int> numbers = {1, 2, 3};
    numbers.push_back(4);
    std::unordered_map<std::string, int> scores;
    return 0;
}"#;
        let result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        assert!(result
            .required_imports
            .contains(&"std::collections::HashMap".to_string()));
        assert!(result
            .generated_rust_code
            .contains("use std::collections::HashMap;"));
        assert!(result
            .generated_rust_code
            .contains("let mut numbers: Vec<i32> = vec![1, 2, 3];"));
        assert!(result.generated_rust_code.contains("numbers.push(4);"));
        assert!(result
            .faithful_rust_code
            .contains("let mut scores: HashMap<String, i32> = HashMap::new();"));
        assert!(result
            .idiomatic_rust_code
            .contains("let scores: HashMap<String, i32> = HashMap::new();"));
    }

    #[test]
    fn converts_class_with_method_and_template() {
        let cpp_source = r#"template <typename T>
class BoxValue {
public:
    T value;
    T getValue() {
        return value;
    }
};"#;
        let result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        assert_honest_engine_contract(&result, &SourceLanguage::Cpp);
        if uses_raw_regex_fallback(&result) {
            assert!(result.generated_rust_code.contains("pub struct BoxValue<T> {"));
            assert!(result.generated_rust_code.contains("pub value: T,"));
            assert!(result
                .generated_rust_code
                .contains("impl<T: Clone> BoxValue<T> {"));
            assert!(result
                .generated_rust_code
                .contains("pub fn getValue(&self) -> T {"));
        }
    }

    #[test]
    fn detects_unmatched_braces_and_unsafe_calls() {
        let c_source = r#"int main() {
    char* buf = (char*)malloc(16);
    gets(buf);
"#;
        let result = convert_source_to_rust(c_source, &SourceLanguage::C);
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.severity == DiagnosticSeverity::Error));
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("malloc")));
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("gets")));
    }

    #[test]
    fn catches_misspelled_include_and_undeclared_vector_identifier() {
        let cpp_source = r#"#inlcude <iostream>
#include <vector>

int main(){

    std::vector<float> fvec;
    fvec.push_back(10.5);
    fvec.push_back(30.5);
    fvec.push_back(155.664);

    for(auto& vec: fcvec){
        std::cout << vec << std::endl;
    }

    return 0;
}"#;
        let result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);

        assert!(
            !result.generated_rust_code.contains("\n#inlcude")
                && !result.generated_rust_code.starts_with("#inlcude"),
            "Misspelled #inlcude must not leak as raw Rust code"
        );
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("#inlcude")
                && diag
                    .suggested_fix
                    .as_deref()
                    .unwrap_or("")
                    .contains("#include")));
        assert!(result
            .diagnostics
            .iter()
            .any(|diag| diag.message.contains("fcvec")
                && diag
                    .suggested_fix
                    .as_deref()
                    .unwrap_or("")
                    .contains("fvec")));
    }

    #[test]
    fn converts_float_vector_and_float_suffix_literals() {
        let cpp_source = r#"#include <iostream>
#include <vector>

int main() {
    std::vector<float> fvec;
    fvec.push_back(10.5f);
    for (auto& vec : fvec) {
        std::cout << vec << std::endl;
    }
    return 0;
}"#;
        let result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        assert_honest_engine_contract(&result, &SourceLanguage::Cpp);
        if uses_raw_regex_fallback(&result) {
            assert!(result.diagnostics.is_empty());
            assert!(result
                .faithful_rust_code
                .contains("let mut fvec: Vec<f32> = Vec::new();"));
            assert!(result
                .idiomatic_rust_code
                .contains("let mut fvec = Vec::new();"));
            assert!(result.generated_rust_code.contains("fvec.push(10.5_f32);"));
            assert!(result.generated_rust_code.contains("for vec in &mut fvec {"));
        }
    }

    #[test]
    fn converts_forward_prototype_char_pointer_and_single_line_if() {
        let c_source = r#"#include <stdio.h>
#include <stdbool.h>
#include <string.h>
#include <ctype.h>

bool IsValidNumber(char *p_input, int base);

int main() {
    if (IsValidNumber("1A3F", 16)) {
        printf("valid\n");
    }
    return 0;
}

bool IsValidNumber(char *p_input, int base) {
    if (base < 2 || base > 36) return false;
    int len = strlen(p_input);
    if (len == 0) return false;
    for (int i = 0; i < len; i++) {
        char ch = toupper(p_input[i]);
        int digit = 0;
        if (isdigit(ch)) {
            digit = ch - '0';
        } else if (isalpha(ch)) {
            digit = ch - 'A' + 10;
        } else {
            return false;
        }
        if (digit >= base) return false;
    }
    return true;
}"#;
        let result = convert_source_to_rust(c_source, &SourceLanguage::C);
        assert_honest_engine_contract(&result, &SourceLanguage::C);
        if uses_raw_regex_fallback(&result) {
            assert!(result
                .faithful_rust_code
                .contains("// Forward declaration: bool IsValidNumber(char *p_input, int base);"));
            assert!(result
                .idiomatic_rust_code
                .contains("fn IsValidNumber(p_input: &str, base: i32) -> bool {"));
            assert!(result
                .idiomatic_rust_code
                .contains("if base < 2 || base > 36 {"));
            assert!(!result
                .idiomatic_rust_code
                .contains("// Forward declaration:"));
        }
    }

    #[test]
    fn converts_float_h_long_double_sizeof_and_emits_compatibility_warning() {
        let c_source = r#"#include <float.h>
#include <stdio.h>

int main(void) {
    printf("sizeof(long double): %zu\n", sizeof(long double));
    printf("LDBL_MANT_DIG: %d\n", LDBL_MANT_DIG);
    printf("LDBL_DIG: %d\n", LDBL_DIG);
    printf("LDBL_MAX: %Lg\n", LDBL_MAX);
    return 0;
}"#;
        let result = convert_source_to_rust(c_source, &SourceLanguage::C);
        assert!(result
            .generated_rust_code
            .contains("std::mem::size_of::<f64>()"));
        assert!(result
            .generated_rust_code
            .contains("f64::MANTISSA_DIGITS"));
        assert!(result.generated_rust_code.contains("f64::DIGITS"));
        assert!(result.generated_rust_code.contains("f64::MAX"));
        assert!(result.diagnostics.iter().any(|diag| {
            diag.severity == DiagnosticSeverity::Warning && diag.message.contains("long double")
        }));
    }

    #[test]
    fn dispatches_c_to_c2rust_and_cpp_to_cpp2rust_engines_with_rules() {
        let c_result = convert_source_to_rust("int main(void) { return 0; }", &SourceLanguage::C);
        if detect_external_transpiler_binary(&SourceLanguage::C).is_none() {
            assert!(c_result.used_fallback);
            assert!(c_result.fallback_reason.is_some());
            assert!(c_result.transpiler_engine.contains("Built-in Regex C Fallback"));
            assert!(!c_result.transpiler_engine.starts_with("c2rust ("));
        } else {
            assert!(!c_result.used_fallback);
            assert!(c_result.transpiler_engine.contains("c2rust CLI"));
        }

        let cpp_source = r#"#include <utility>
#include <limits>
#include <iostream>
int main() {
    std::pair<int, int> p = std::make_pair(3, 7);
    int hi = std::numeric_limits<int>::max();
    std::cout << p.first << " " << p.second << " " << hi << std::endl;
    return 0;
}"#;
        let cpp_result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        assert_honest_engine_contract(&cpp_result, &SourceLanguage::Cpp);
        if uses_raw_regex_fallback(&cpp_result) {
            assert!(cpp_result
                .faithful_rust_code
                .contains("let mut p: (i32, i32) = (3, 7);"));
            assert!(cpp_result.idiomatic_rust_code.contains("let p: (i32, i32) = (3, 7);"));
            assert!(cpp_result.idiomatic_rust_code.contains("i32::MAX"));
            assert!(cpp_result.idiomatic_rust_code.contains("p.0"));
            assert!(cpp_result.idiomatic_rust_code.contains("p.1"));
        }
    }

    #[test]
    fn flags_unsupported_cpp_syntax_and_marks_fallback_unverified() {
        let cpp_source = r#"#include <iostream>
namespace demo {
class Base {
public:
    virtual int compute() { return 1; }
};
}
int main() {
    try {
        throw 42;
    } catch (int e) {
        std::cout << e << std::endl;
    }
    return 0;
}"#;
        let result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        assert_honest_engine_contract(&result, &SourceLanguage::Cpp);
        if uses_raw_regex_fallback(&result) {
            assert!(!result.unsupported_constructs.is_empty());
            assert!(!result.post_processing.compiler_verified);
            assert!(result.diagnostics.iter().any(|d| {
                d.severity == DiagnosticSeverity::Error && d.category == ErrorCategory::Conversion
            }));
            assert!(result
                .generated_rust_code
                .contains("// [Unsupported C/C++ syntax in regex fallback]:"));
        }
    }

    #[cfg(unix)]
    #[test]
    fn invokes_external_c2rust_and_cpp2rust_cli_wrappers_when_configured() {
        use std::os::unix::fs::PermissionsExt;

        let script_dir = TempEngineDirectory::create().unwrap();

        let mock_c2rust = script_dir.path.join("mock_c2rust.sh");
        fs::write(
            &mock_c2rust,
            "#!/bin/sh\nif [ \"$1\" = \"transpile\" ]; then\n  out_dir=\"$4\"\n  echo 'pub unsafe fn main() { println!(\"from c2rust cli\"); }' > \"$out_dir/input.rs\"\n  exit 0\nfi\nexit 1\n",
        )
        .unwrap();
        let mut c_perms = fs::metadata(&mock_c2rust).unwrap().permissions();
        c_perms.set_mode(0o755);
        fs::set_permissions(&mock_c2rust, c_perms).unwrap();

        let c_out = try_invoke_external_backbone_engine(
            "int main(void) { return 0; }",
            &SourceLanguage::C,
            mock_c2rust.to_str().unwrap(),
        )
        .expect("mock c2rust should return generated Rust");
        assert!(c_out.contains("from c2rust cli"));

        let mock_cpp2rust = script_dir.path.join("mock_cpp2rust.sh");
        fs::write(
            &mock_cpp2rust,
            "#!/bin/sh\nfor arg in \"$@\"; do\n  case \"$arg\" in\n    -o=*) out_file=\"${arg#-o=}\" ;;\n  esac\ndone\necho 'fn main() { println!(\"from cpp2rust cli\"); }' > \"$out_file\"\n",
        )
        .unwrap();
        let mut cpp_perms = fs::metadata(&mock_cpp2rust).unwrap().permissions();
        cpp_perms.set_mode(0o755);
        fs::set_permissions(&mock_cpp2rust, cpp_perms).unwrap();

        let cpp_out = try_invoke_external_backbone_engine(
            "int main() { return 0; }",
            &SourceLanguage::Cpp,
            mock_cpp2rust.to_str().unwrap(),
        )
        .expect("mock cpp2rust should return generated Rust");
        assert!(cpp_out.contains("from cpp2rust cli"));
    }

    #[test]
    fn postprocesses_idiomatic_rust_formats_with_rustfmt_and_rejects_broken_transforms() {
        let faithful = r#"// #include <stdio.h>
// Forward declaration: int helper();
pub unsafe fn main() {
    let mut x: i32 = 42;
    let mut idx: usize = 0;
    let mut unused_count: libc::c_int = 7;
    unsafe {
        println!("{} {} {}", x, idx as usize, unused_count);
    }
    // return 0;
}"#;
        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert!(summary.compiler_verified);
        assert!(summary.formatted_with_rustfmt);
        assert!(summary.unsafe_blocks_removed >= 1);
        assert!(summary.redundant_mut_removed >= 2);
        assert!(!idiomatic.contains("pub unsafe fn main"));
        assert!(!idiomatic.contains("libc::c_int"));
        assert!(!idiomatic.contains("idx as usize"));
        assert!(!idiomatic.contains("// #include <stdio.h>"));

        let mut rejected = Vec::new();
        let (rolled_back, accepted) = postprocessor::verify_or_rollback_transformation(
            faithful,
            "fn main() { let broken = unknown_symbol_123(); }",
            "broken_pass",
            &mut rejected,
        );
        assert!(!accepted);
        assert_eq!(rolled_back, faithful);
        assert_eq!(rejected.len(), 1);
        assert!(rejected[0].contains("broken_pass"));
    }

    #[test]
    fn explains_std_vector_int_to_vec_i32_ownership_and_iteration() {
        let cpp_source = r#"#include <iostream>
#include <vector>

int main() {
    std::vector<int> nums = {1, 2, 3};
    nums.push_back(4);
    for (const auto& n : nums) {
        std::cout << n << std::endl;
    }
    return 0;
}"#;
        let result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        let vec_expl = result
            .explanations
            .iter()
            .find(|item| item.rust_equivalent == "Vec<i32>")
            .expect("Expected Vec<i32> conversion explanation for std::vector<int>");
        assert_eq!(vec_expl.cpp_pattern, "std::vector<int>");
        assert!(vec_expl.why_chosen.contains("Vec<i32>"));
        assert!(vec_expl.ownership_and_iteration.contains("&vec"));
        assert!(vec_expl.ownership_and_iteration.contains("&mut vec"));
        assert!(vec_expl.ownership_and_iteration.contains(".into_iter()"));
        assert!(!result
            .generated_rust_code
            .contains("// --- Rust Translation Notes"));
        if uses_raw_regex_fallback(&result) {
            assert!(result
                .generated_rust_code
                .contains("// std::vector -> Vec<i32>: owns its heap buffer"));
            assert!(result.generated_rust_code.contains("RAII Drop"));
            assert!(result.generated_rust_code.contains("&vec"));
            assert!(result.generated_rust_code.contains("&mut vec"));
        }
    }

    #[test]
    fn converts_c_malloc_int_pointer_and_null_check_and_executes() {
        let c_source = r#"#include <stdlib.h>
#include <stdio.h>

int main(){
int *p = malloc(sizeof(int));
if (p != NULL) {
    *p = 100;
    printf("VALUE of p is %d\n",*p);
    free(p);
}
  return 0;
}"#;
        let faithful = convert_source_to_rust_with_style(
            c_source,
            &SourceLanguage::C,
            RustOutputStyle::Faithful,
        );
        assert_eq!(faithful.output_style, RustOutputStyle::Faithful);
        assert_honest_engine_contract(&faithful, &SourceLanguage::C);
        if uses_raw_regex_fallback(&faithful) {
            assert!(faithful.generated_rust_code.contains("// #include <stdlib.h>"));
            assert!(faithful
                .generated_rust_code
                .contains("let mut p: Box<i32> = Box::new(Default::default());"));
            assert!(faithful.generated_rust_code.contains("if true {"));
            assert!(faithful.generated_rust_code.contains("*p = 100;"));
            assert!(faithful.generated_rust_code.contains("drop(p);"));
            assert!(faithful.generated_rust_code.contains("// return 0;"));
            assert!(!faithful.generated_rust_code.contains("Box<int>"));
            assert!(!faithful.generated_rust_code.contains("p != None"));
        }

        let idiomatic = convert_source_to_rust_with_style(
            c_source,
            &SourceLanguage::C,
            RustOutputStyle::Idiomatic,
        );
        assert_eq!(idiomatic.output_style, RustOutputStyle::Idiomatic);
        assert_honest_engine_contract(&idiomatic, &SourceLanguage::C);
        if uses_raw_regex_fallback(&idiomatic) {
            assert!(idiomatic
                .generated_rust_code
                .contains("let p: Box<i32> = Box::new(100);"));
            assert!(!idiomatic.generated_rust_code.contains("// #include"));
            assert!(!idiomatic.generated_rust_code.contains("if true"));
            assert!(!idiomatic.generated_rust_code.contains("Default::default()"));
            assert!(!idiomatic.generated_rust_code.contains("drop(p);"));
            assert!(!idiomatic.generated_rust_code.contains("Box<int>"));
            assert!(!idiomatic.generated_rust_code.contains("p != None"));
            assert!(!idiomatic
                .generated_rust_code
                .contains("// --- Rust Translation Notes"));
            assert!(idiomatic
                .generated_rust_code
                .contains("// Rust Box<i32>: single-owner heap allocation"));
        }
    }

    #[test]
    fn converts_allman_c_strchr_loop_and_float_vector_insert_begin() {
        let c_source = r#"#include <stdio.h>
#include <string.h>

int main(void)
{
    const char *str = "Try not. Do, or do not. There is no try.";
    char target = 'T';
    const char* result = str;

    while((result = strchr(result, target)) != NULL)
    {
        printf("Found '%c' starting at '%s'\n", target, result);
        ++result; // Increment result, otherwise we'll find target at the same location
    }
}"#;
        let c_res = convert_source_to_rust(c_source, &SourceLanguage::C);
        assert_honest_engine_contract(&c_res, &SourceLanguage::C);
        if uses_raw_regex_fallback(&c_res) {
            assert!(c_res.post_processing.compiler_verified);
            assert!(c_res.generated_rust_code.contains("fn main() {"));
            assert!(c_res
                .generated_rust_code
                .contains("while let Some(_pos) = result.find(target) {"));
            assert!(c_res.generated_rust_code.contains("result = &result[1..];"));
        }

        let cpp_source = r#"#include <iostream>
#include <vector>

int main() {
    std::vector<float> fvec;

    fvec.push_back(10.5);
    fvec.push_back(30.5);
    fvec.push_back(155.664);

    fvec.insert(fvec.begin(), 999);

    for (const auto& vec : fvec) {
        std::cout << vec << std::endl;
    }

    return 0;
}"#;
        let cpp_res = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        assert_honest_engine_contract(&cpp_res, &SourceLanguage::Cpp);
        if uses_raw_regex_fallback(&cpp_res) {
            assert!(cpp_res.post_processing.compiler_verified);
            assert!(cpp_res.generated_rust_code.contains("fvec.insert(0, 999.0);"));
        }
    }

    #[test]
    fn idiomatic_simplifier_preserves_scope_when_if_true_contains_let() {
        let c_source = r#"#include <stdlib.h>
#include <stdio.h>

int main() {
    int x = 1;
    int *p = malloc(sizeof(int));
    if (p != NULL) {
        int x = 2;
        *p = x;
        printf("%d ", *p);
        free(p);
    }
    printf("%d\n", x);
    return 0;
}"#;
        let idiomatic = convert_source_to_rust_with_style(
            c_source,
            &SourceLanguage::C,
            RustOutputStyle::Idiomatic,
        );
        assert_honest_engine_contract(&idiomatic, &SourceLanguage::C);
        if uses_raw_regex_fallback(&idiomatic) {
            assert!(idiomatic.post_processing.compiler_verified);
            assert!(!idiomatic.generated_rust_code.contains("if true"));
            assert!(idiomatic.generated_rust_code.contains("{\n        let x: i32 = 2;"));
        }
    }

    #[test]
    fn idiomatic_simplifier_ignores_braces_in_comments_and_strings() {
        let faithful = r#"fn main() {
    let flag = true;
    if true {
        if flag {
            println!("}}");
            // }
        }
        println!("after");
    }
}"#;
        let (idiomatic, summary) = postprocess_rust_code(faithful);
        assert!(summary.compiler_verified);
        assert!(!idiomatic.contains("if true"));
        assert!(idiomatic.contains("println!(\"after\");"));
        assert!(idiomatic.contains("println!(\"}}\");"));
    }

    #[test]
    fn converts_cpp_stl_vector_erase_remove_algorithms_strings_and_maps() {
        let erase_remove_src = r#"#include <iostream>
#include <vector>

int main() {
    std::vector<int> fvec = {3};

    fvec.push_back(10);
    fvec.push_back(30);
    fvec.push_back(155);
    fvec.emplace_back(888);

    fvec.erase(std::remove(fvec.begin(), fvec.end(), 155), fvec.end());

    for (const auto& vec : fvec) {
        std::cout << vec << std::endl;
    }

    return 0;
}"#;
        let res = convert_source_to_rust(erase_remove_src, &SourceLanguage::Cpp);
        assert_honest_engine_contract(&res, &SourceLanguage::Cpp);
        if uses_raw_regex_fallback(&res) {
            assert!(res.post_processing.compiler_verified);
            assert!(res
                .generated_rust_code
                .contains("fvec.retain(|&_x| _x != 155);"));
            assert!(res.generated_rust_code.contains("fvec.push(888);"));
        }

        let algorithms_src = r#"#include <iostream>
#include <vector>
#include <algorithm>
#include <numeric>
#include <unordered_map>
#include <string>

int main() {
    std::vector<int> nums = {5, 1, 5, 2, 3, 3};
    std::sort(nums.begin(), nums.end());
    nums.erase(std::unique(nums.begin(), nums.end()), nums.end());
    nums.insert(nums.begin(), 2, 9);
    nums.resize(10, 0);
    nums.reserve(20);
    nums.shrink_to_fit();

    if (std::find(nums.begin(), nums.end(), 5) != nums.end()) {
        std::cout << std::count(nums.begin(), nums.end(), 9) << std::endl;
    }
    int total = std::accumulate(nums.begin(), nums.end(), 0);
    std::string text = std::to_string(total);
    int parsed = std::stoi(text);
    if (text.find("1") != std::string::npos) {
        std::cout << parsed << std::endl;
    }

    std::unordered_map<std::string, int> scores;
    scores["alice"] = 10;
    scores["bob"] = 20;
    scores.erase("bob");
    if (scores.find("alice") != scores.end()) {
        for (const auto& [k, v] : scores) {
            std::cout << k << " " << v << std::endl;
        }
    }
    return 0;
}"#;
        let algo_res = convert_source_to_rust(algorithms_src, &SourceLanguage::Cpp);
        assert_honest_engine_contract(&algo_res, &SourceLanguage::Cpp);
        if uses_raw_regex_fallback(&algo_res) {
            assert!(algo_res.post_processing.compiler_verified);
            assert!(algo_res.generated_rust_code.contains("nums.dedup();"));
            assert!(algo_res.generated_rust_code.contains("nums.splice(0..0, std::iter::repeat(9).take(2));"));
            assert!(algo_res.generated_rust_code.contains("nums.resize(10, 0);"));
            assert!(algo_res.generated_rust_code.contains("nums.contains(&5)"));
            assert!(algo_res.generated_rust_code.contains("nums.iter().fold(0, |acc, &_x| acc + _x)"));
            assert!(algo_res.generated_rust_code.contains("format!(\"{}\", total)"));
            assert!(algo_res.generated_rust_code.contains(".parse::<i32>().unwrap_or(0)"));
            assert!(algo_res.generated_rust_code.contains("scores.remove(\"bob\");"));
            assert!(algo_res.generated_rust_code.contains("scores.contains_key(\"alice\")"));
            assert!(algo_res.generated_rust_code.contains("for (k, v) in &scores {"));
        }
    }

    #[test]
    fn sanitizes_c_self_referencing_struct_pointers_for_c2rust() {
        let input = r#"
typedef struct Edge {
    uint32_t to;
    uint32_t weight;
    struct Edge *next;
} Edge;
struct Node {
    int val;
    struct Node *left;
    struct Node *right;
};
// struct Node { struct Node *left; };
const char *literal = "struct Node { struct Node *left; };";
struct Element { struct Other *p; } Other;
"#;
        let sanitized = sanitize_c_source_for_c2rust(input);
        // Struct definition has its self-referential pointers sanitized
        assert!(sanitized.contains("void *next;"));
        assert!(sanitized.contains("void *left;"));
        assert!(sanitized.contains("void *right;"));

        // Preserves comments and string literals without rewriting them
        assert!(sanitized.contains("// struct Node { struct Node *left; };"));
        assert!(sanitized.contains(r#"const char *literal = "struct Node { struct Node *left; };";"#));

        // Preserves unrelated struct pointer types and does not confuse instance names with typedefs
        assert!(sanitized.contains("struct Other *p;"));
        assert!(!sanitized.contains("struct void *p;"));
    }

    #[test]
    fn transpiles_and_verifies_c_arena_dijkstra_program_via_c2rust() {
        let c_dijkstra = r#"#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <stdalign.h>
#include <limits.h>

#define ARENA_CAPACITY (64 * 1024)
#define HASH_BUCKETS   32
#define MAX_VERTICES   16

typedef struct {
    uint8_t *base;
    size_t capacity;
    size_t offset;
} Arena;

static void *arena_alloc(Arena *a, size_t size, size_t align) {
    uintptr_t curr = (uintptr_t)(a->base + a->offset);
    uintptr_t aligned = (curr + (align - 1)) & ~(uintptr_t)(align - 1);
    size_t next = (size_t)(aligned - (uintptr_t)a->base) + size;
    if (next > a->capacity) return NULL;
    a->offset = next;
    return memset((void *)aligned, 0, size);
}

typedef struct {
    const char *key;
    uint32_t vertex_id;
    bool occupied;
} HashSlot;

typedef struct {
    HashSlot slots[HASH_BUCKETS];
} SymbolTable;

static uint32_t hash_str(const char *s) {
    uint32_t h = 2166136261u;
    while (*s) {
        h ^= (uint8_t)(*s++);
        h *= 16777619u;
    }
    return h;
}

static bool sym_insert(SymbolTable *st, const char *key, uint32_t id) {
    uint32_t idx = hash_str(key) & (HASH_BUCKETS - 1);
    for (uint32_t i = 0; i < HASH_BUCKETS; i++) {
        uint32_t probe = (idx + i) & (HASH_BUCKETS - 1);
        if (!st->slots[probe].occupied) {
            st->slots[probe] = (HashSlot){ .key = key, .vertex_id = id, .occupied = true };
            return true;
        }
    }
    return false;
}

static int32_t sym_lookup(const SymbolTable *st, const char *key) {
    uint32_t idx = hash_str(key) & (HASH_BUCKETS - 1);
    for (uint32_t i = 0; i < HASH_BUCKETS; i++) {
        uint32_t probe = (idx + i) & (HASH_BUCKETS - 1);
        if (!st->slots[probe].occupied) return -1;
        if (strcmp(st->slots[probe].key, key) == 0) return (int32_t)st->slots[probe].vertex_id;
    }
    return -1;
}

typedef struct Edge {
    uint32_t to;
    uint32_t weight;
    struct Edge *next;
} Edge;

typedef struct {
    const char *names[MAX_VERTICES];
    Edge *adj[MAX_VERTICES];
    uint32_t vertex_count;
    SymbolTable symbols;
} Graph;

static uint32_t graph_add_vertex(Graph *g, const char *name) {
    int32_t existing = sym_lookup(&g->symbols, name);
    if (existing >= 0) return (uint32_t)existing;
    uint32_t id = g->vertex_count++;
    g->names[id] = name;
    sym_insert(&g->symbols, name, id);
    return id;
}

static void graph_add_edge(Graph *g, Arena *a, const char *src, const char *dst, uint32_t weight) {
    uint32_t u = graph_add_vertex(g, src);
    uint32_t v = graph_add_vertex(g, dst);
    Edge *e = arena_alloc(a, sizeof(Edge), alignof(Edge));
    e->to = v;
    e->weight = weight;
    e->next = g->adj[u];
    g->adj[u] = e;
}

typedef struct {
    uint32_t vertex;
    uint32_t dist;
} HeapNode;

typedef struct {
    HeapNode *data;
    size_t size;
    size_t cap;
} MinHeap;

static void heap_push(MinHeap *h, uint32_t v, uint32_t dist) {
    if (h->size == h->cap) {
        h->cap = h->cap ? h->cap * 2 : 8;
        h->data = realloc(h->data, h->cap * sizeof(HeapNode));
    }
    size_t i = h->size++;
    h->data[i] = (HeapNode){ v, dist };
    while (i > 0) {
        size_t p = (i - 1) / 2;
        if (h->data[p].dist <= h->data[i].dist) break;
        HeapNode tmp = h->data[p];
        h->data[p] = h->data[i];
        h->data[i] = tmp;
        i = p;
    }
}

static HeapNode heap_pop(MinHeap *h) {
    HeapNode top = h->data[0];
    h->data[0] = h->data[--h->size];
    size_t i = 0;
    while (true) {
        size_t l = 2 * i + 1, r = 2 * i + 2, smallest = i;
        if (l < h->size && h->data[l].dist < h->data[smallest].dist) smallest = l;
        if (r < h->size && h->data[r].dist < h->data[smallest].dist) smallest = r;
        if (smallest == i) break;
        HeapNode tmp = h->data[i];
        h->data[i] = h->data[smallest];
        h->data[smallest] = tmp;
        i = smallest;
    }
    return top;
}

typedef struct {
    uint32_t total_cost;
    size_t hop_count;
    uint32_t hops[];
} RouteResult;

static RouteResult *dijkstra_shortest_path(const Graph *g, Arena *a, const char *start_name, const char *end_name) {
    int32_t src = sym_lookup(&g->symbols, start_name);
    int32_t dst = sym_lookup(&g->symbols, end_name);
    if (src < 0 || dst < 0) return NULL;

    uint32_t dist[MAX_VERTICES];
    int32_t prev[MAX_VERTICES];
    for (uint32_t i = 0; i < g->vertex_count; i++) {
        dist[i] = UINT32_MAX;
        prev[i] = -1;
    }
    dist[src] = 0;

    MinHeap pq = { 0 };
    heap_push(&pq, (uint32_t)src, 0);

    while (pq.size > 0) {
        HeapNode curr = heap_pop(&pq);
        if (curr.dist > dist[curr.vertex]) continue;
        if (curr.vertex == (uint32_t)dst) break;

        for (Edge *e = g->adj[curr.vertex]; e != NULL; e = e->next) {
            uint32_t alt = dist[curr.vertex] + e->weight;
            if (alt < dist[e->to]) {
                dist[e->to] = alt;
                prev[e->to] = (int32_t)curr.vertex;
                heap_push(&pq, e->to, alt);
            }
        }
    }
    free(pq.data);

    if (dist[dst] == UINT32_MAX) return NULL;

    size_t count = 0;
    for (int32_t at = dst; at != -1; at = prev[at]) count++;

    RouteResult *res = arena_alloc(a, sizeof(RouteResult) + count * sizeof(uint32_t), alignof(RouteResult));
    res->total_cost = dist[dst];
    res->hop_count = count;

    size_t idx = count;
    for (int32_t at = dst; at != -1; at = prev[at]) {
        res->hops[--idx] = (uint32_t)at;
    }
    return res;
}

int main(void) {
    Arena arena = { .base = malloc(ARENA_CAPACITY), .capacity = ARENA_CAPACITY, .offset = 0 };
    if (!arena.base) return 1;

    Graph *net = arena_alloc(&arena, sizeof(Graph), alignof(Graph));
    graph_add_edge(net, &arena, "Srinagar", "Sonamarg", 80);
    graph_add_edge(net, &arena, "Sonamarg", "Kargil", 125);
    graph_add_edge(net, &arena, "Kargil", "Leh", 215);

    RouteResult *route = dijkstra_shortest_path(net, &arena, "Srinagar", "Leh");
    if (route) {
        printf("Optimal Route Found (%zu hops, Total Distance: %u km)\n", route->hop_count, route->total_cost);
    }

    free(arena.base);
    return 0;
}
"#;
        let res = convert_source_to_rust(c_dijkstra, &SourceLanguage::C);
        if detect_external_transpiler_binary(&SourceLanguage::C).is_some() {
            assert!(!res.used_fallback, "c2rust must be used without fallback when installed: {:?}", res.fallback_reason);
            assert!(res.transpiler_engine.contains("c2rust CLI"));
            assert!(res.faithful_rust_code.contains("pub struct Arena") || res.faithful_rust_code.contains("pub struct AnonStruct"));

            // Faithful keeps the real c2rust output, and that output must be valid Rust.
            assert!(
                postprocessor::check_rustc_and_collect_diagnostics(&res.faithful_rust_code).is_ok(),
                "c2rust Faithful output for the dijkstra program must compile"
            );

            // This program's translation still needs malloc/free and raw pointers, so
            // it is raw C-shaped Rust: available in Faithful, never shown as Idiomatic.
            assert!(!res.post_processing.compiler_verified);
            assert!(res
                .post_processing
                .verification_error
                .as_deref()
                .unwrap_or_default()
                .contains("raw engine transcription"));
            assert!(res.faithful_rust_code.contains("dijkstra_shortest_path"));
            assert!(!res.generated_rust_code.contains("unsafe"));
            assert!(res.generated_rust_code.contains("Readable Rust is unavailable"));
        }
    }

    #[test]
    fn test_user_cpp_and_c_snippets() {
        let cpp_snippet = r#"#include <iostream>
#include <vector>
#include <algorithm>

void printVec(const std::string& label, const std::vector<int>& v) {
    std::cout << label << ": { ";
    for (int x : v) std::cout << x << " ";
    std::cout << "}\n";
}

int main() {
    std::vector<int> scores = {70, 85, 70, 90};
    printVec("Initial", scores);

    // 1. Update single elements by index
    scores[1] = 88;
    scores.at(3) = 95;
    printVec("After index update", scores);

    // 2. Replace all occurrences of 70 with 75
    std::replace(scores.begin(), scores.end(), 70, 75);
    printVec("After replace", scores);

    // 3. Overwrite entire vector with three 100s
    scores.assign(3, 100);
    printVec("After assign", scores);

    return 0;
}
"#;
        let res_cpp = convert_source_to_rust_with_style(cpp_snippet, &SourceLanguage::Cpp, RustOutputStyle::Idiomatic);
        println!("=== CPP ENGINE: {} ===", res_cpp.transpiler_engine);
        println!("=== CPP USED FALLBACK: {} (reason: {:?}) ===", res_cpp.used_fallback, res_cpp.fallback_reason);
        println!("=== CPP VERIFIED: {} (err: {:?}) ===", res_cpp.post_processing.compiler_verified, res_cpp.post_processing.verification_error);
        println!("=== CPP GENERATED RUST ===\n{}", res_cpp.generated_rust_code);
        println!("=== CPP FAITHFUL RUST ===\n{}", res_cpp.faithful_rust_code);

        let c_lru = r#"#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

#define POOL_SIZE      8
#define CACHE_CAPACITY 4
#define HASH_SIZE      16
#define KEY_LEN        24
#define VAL_LEN        48

/* 1. Fixed-Size Record & Compile-Time Layout Check (Pure C11) */
typedef struct CacheNode {
    char key[KEY_LEN];
    char value[VAL_LEN];
    uint32_t crc32;
    bool in_use;
    struct CacheNode *prev;
    struct CacheNode *next;
} CacheNode;

_Static_assert(sizeof(CacheNode) >= 72, "CacheNode layout size verification failed");

/* 2. Fixed-Block Slab Memory Pool */
typedef struct {
    CacheNode slots[POOL_SIZE];
    CacheNode *free_head;
} SlabPool;

static void pool_init(SlabPool *pool) {
    memset(pool, 0, sizeof(*pool));
    for (size_t i = 0; i < POOL_SIZE - 1; i++) {
        pool->slots[i].next = &pool->slots[i + 1];
    }
    pool->slots[POOL_SIZE - 1].next = NULL;
    pool->free_head = &pool->slots[0];
}

static CacheNode *pool_alloc(SlabPool *pool) {
    if (!pool->free_head) return NULL;
    CacheNode *node = pool->free_head;
    pool->free_head = node->next;
    memset(node, 0, sizeof(*node));
    node->in_use = true;
    return node;
}

static void pool_free(SlabPool *pool, CacheNode *node) {
    node->in_use = false;
    node->next = pool->free_head;
    pool->free_head = node;
}

/* 3. Bitwise CRC32 Integrity Checksum */
static uint32_t compute_crc32(const void * restrict data, size_t len) {
    const uint8_t *bytes = (const uint8_t *)data;
    uint32_t crc = 0xFFFFFFFFu;
    for (size_t i = 0; i < len; i++) {
        crc ^= bytes[i];
        for (int b = 0; b < 8; b++) {
            crc = (crc >> 1) ^ (0xEDB88320u & -(crc & 1u));
        }
    }
    return ~crc;
}

/* 4. O(1) LRU Cache (Hash Table + Doubly Linked List) */
typedef struct {
    SlabPool pool;
    CacheNode *buckets[HASH_SIZE];
    CacheNode *head; // Most recently used
    CacheNode *tail; // Least recently used
    size_t count;
} LRUCache;

static uint32_t hash_key(const char *key) {
    uint32_t h = 5381u;
    while (*key) h = ((h << 5) + h) ^ (uint8_t)(*key++);
    return h & (HASH_SIZE - 1);
}

static void lru_detach(LRUCache *c, CacheNode *n) {
    if (n->prev) n->prev->next = n->next;
    else c->head = n->next;

    if (n->next) n->next->prev = n->prev;
    else c->tail = n->prev;

    n->prev = n->next = NULL;
}

static void lru_attach_front(LRUCache *c, CacheNode *n) {
    n->prev = NULL;
    n->next = c->head;
    if (c->head) c->head->prev = n;
    c->head = n;
    if (!c->tail) c->tail = n;
}

static CacheNode *lru_get(LRUCache *c, const char *key) {
    uint32_t idx = hash_key(key);
    CacheNode *curr = c->buckets[idx];
    if (curr && curr->in_use && strcmp(curr->key, key) == 0) {
        lru_detach(c, curr);
        lru_attach_front(c, curr);
        printf("[HIT]   Key '%s' -> '%s' (Moved to MRU)\n", curr->key, curr->value);
        return curr;
    }
    printf("[MISS]  Key '%s' not in cache\n", key);
    return NULL;
}

static void lru_put(LRUCache *c, const char *key, const char *val) {
    CacheNode *existing = lru_get(c, key);
    if (existing) {
        strncpy(existing->value, val, VAL_LEN - 1);
        existing->crc32 = compute_crc32(existing->value, strlen(existing->value));
        return;
    }

    if (c->count == CACHE_CAPACITY) {
        CacheNode *victim = c->tail;
        printf("[EVICT] Dropping LRU Key '%s' (CRC: 0x%08X)\n", victim->key, victim->crc32);
        c->buckets[hash_key(victim->key)] = NULL;
        lru_detach(c, victim);
        pool_free(&c->pool, victim);
        c->count--;
    }

    CacheNode *node = pool_alloc(&c->pool);
    if (!node) return;

    strncpy(node->key, key, KEY_LEN - 1);
    strncpy(node->value, val, VAL_LEN - 1);
    node->crc32 = compute_crc32(node->value, strlen(node->value));

    c->buckets[hash_key(key)] = node;
    lru_attach_front(c, node);
    c->count++;
    printf("[PUT]   Inserted '%s' = '%s' (CRC: 0x%08X)\n", node->key, node->value, node->crc32);
}

/* 5. Binary Write-Ahead Log (WAL) Persistence */
static void wal_flush(const LRUCache *c, const char *filename) {
    FILE *fp = fopen(filename, "wb");
    if (!fp) return;
    for (CacheNode *curr = c->head; curr != NULL; curr = curr->next) {
        fwrite(curr->key, 1, KEY_LEN, fp);
        fwrite(curr->value, 1, VAL_LEN, fp);
        fwrite(&curr->crc32, sizeof(uint32_t), 1, fp);
    }
    fclose(fp);
    printf("[WAL]   Flushed %zu active records to '%s'\n", c->count, filename);
}

static void wal_verify(const char *filename) {
    FILE *fp = fopen(filename, "rb");
    if (!fp) return;
    char key[KEY_LEN], val[VAL_LEN];
    uint32_t stored_crc = 0;

    puts("\n=== Verifying Binary WAL Checksums ===");
    while (fread(key, 1, KEY_LEN, fp) == KEY_LEN &&
           fread(val, 1, VAL_LEN, fp) == VAL_LEN &&
           fread(&stored_crc, sizeof(uint32_t), 1, fp) == 1) {
        uint32_t calc_crc = compute_crc32(val, strlen(val));
        printf("Record '%s' -> '%s' | Stored CRC: 0x%08X | Status: %s\n",
               key, val, stored_crc, (stored_crc == calc_crc) ? "VALID" : "CORRUPT");
    }
    fclose(fp);
}

int main(void) {
    LRUCache cache = { 0 };
    pool_init(&cache.pool);

    puts("=== 1. Populating Slab-Backed LRU Cache (Capacity: 4) ===");
    lru_put(&cache, "cfg_threads", "16");
    lru_put(&cache, "cfg_timeout", "2500ms");
    lru_put(&cache, "cfg_mode", "strict_iso_c11");
    lru_put(&cache, "cfg_buffer", "65536");

    puts("\n=== 2. Accessing Key to Promote & Triggering Eviction ===");
    lru_get(&cache, "cfg_threads"); // Promotes 'cfg_threads' so 'cfg_timeout' becomes oldest
    lru_put(&cache, "cfg_retries", "3"); // Evicts 'cfg_timeout'

    puts("\n=== 3. Persisting Cache State to Disk ===");
    wal_flush(&cache, "engine_wal.bin");
    wal_verify("engine_wal.bin");

    remove("engine_wal.bin"); // Clean up temporary file
    return 0;
}
"#;
        let res_c = convert_source_to_rust_with_style(c_lru, &SourceLanguage::C, RustOutputStyle::Idiomatic);
        println!("=== C ENGINE: {} ===", res_c.transpiler_engine);
        println!("=== C USED FALLBACK: {} (reason: {:?}) ===", res_c.used_fallback, res_c.fallback_reason);
        println!("=== C VERIFIED: {} (err: {:?}) ===", res_c.post_processing.compiler_verified, res_c.post_processing.verification_error);
        println!("=== C GENERATED RUST ===\n{}", res_c.generated_rust_code);

        assert!(res_cpp.post_processing.compiler_verified, "C++ snippet must pass rustc verification: {:?}", res_cpp.post_processing.verification_error);
        assert!(res_cpp.generated_rust_code.contains("printVec(\"Initial\", &scores);"));
        assert!(res_cpp.generated_rust_code.contains("scores = vec![100; 3];"));
        assert!(!res_cpp.generated_rust_code.contains("libcc2rs"));
        assert!(!res_cpp.generated_rust_code.contains("Ptr::<"));

        let res_cpp_faithful = convert_source_to_rust_with_style(cpp_snippet, &SourceLanguage::Cpp, RustOutputStyle::Faithful);
        assert!(!res_cpp_faithful.generated_rust_code.trim().is_empty());
        if res_cpp
            .transpiler_engine
            .contains("Built-in Readable Projection")
        {
            assert!(res_cpp_faithful.generated_rust_code.contains("libcc2rs"));
        }
        assert_ne!(res_cpp.generated_rust_code, res_cpp_faithful.generated_rust_code, "Faithful and Idiomatic outputs must be distinct");

        assert!(res_c.used_fallback, "C LRU snippet falls back gracefully when upstream c2rust cannot solve complex slab-pool pointer structures");
        assert!(!res_c.post_processing.compiler_verified, "Complex C slab pool is correctly marked unverified rather than falsely claimed verified");
    }

    #[test]
    fn test_cpp_vector_crud_operations() {
        let cpp_code = r#"#include <iostream>
#include <vector>
#include <algorithm>

int main() {
    // 1. Create
    std::vector<int> numbers = {10, 20, 30};
    numbers.push_back(40);
    numbers.insert(numbers.begin(), 5);

    // 2. Read
    int first = numbers[0];
    int second = numbers.at(1);
    std::cout << "First: " << first << ", Second: " << second << std::endl;

    // 3. Update
    numbers[1] = 25;
    std::replace(numbers.begin(), numbers.end(), 30, 35);
    numbers.assign(4, 50);

    // 4. Delete
    numbers.pop_back();
    numbers.clear();

    return 0;
}
"#;
        let idiomatic = convert_source_to_rust_with_style(cpp_code, &SourceLanguage::Cpp, RustOutputStyle::Idiomatic);
        assert!(idiomatic.post_processing.compiler_verified, "Vector CRUD Idiomatic failed rustc verification: {:?}", idiomatic.post_processing.verification_error);
        assert!(idiomatic.generated_rust_code.contains("let mut numbers: Vec<i32> = vec![10, 20, 30];"));
        assert!(idiomatic.generated_rust_code.contains("numbers.push(40);"));
        assert!(idiomatic.generated_rust_code.contains("numbers.insert(0, 5);"));
        assert!(idiomatic.generated_rust_code.contains("numbers.pop();"));
        assert!(idiomatic.generated_rust_code.contains("numbers.clear();"));
        assert!(!idiomatic.generated_rust_code.contains("// #include"));

        let faithful = convert_source_to_rust_with_style(cpp_code, &SourceLanguage::Cpp, RustOutputStyle::Faithful);
        assert!(!faithful.generated_rust_code.trim().is_empty());
        assert_ne!(idiomatic.generated_rust_code, faithful.generated_rust_code);
    }

    #[test]
    fn test_cpp_map_crud_operations() {
        let cpp_code = r#"#include <iostream>
#include <map>
#include <string>

int main() {
    // 1. Create & Insert
    std::map<std::string, int> ages;
    ages["Alice"] = 30;
    ages["Bob"] = 25;

    // 2. Read & Find
    int alice_age = ages["Alice"];
    bool has_bob = ages.find("Bob") != ages.end();
    std::cout << "Alice: " << alice_age << ", Has Bob: " << has_bob << std::endl;

    // 3. Update
    ages["Alice"] = 31;

    // 4. Delete
    ages.erase("Bob");
    ages.clear();

    return 0;
}
"#;
        let idiomatic = convert_source_to_rust_with_style(cpp_code, &SourceLanguage::Cpp, RustOutputStyle::Idiomatic);
        assert!(idiomatic.post_processing.compiler_verified, "Map CRUD Idiomatic failed rustc verification: {:?}", idiomatic.post_processing.verification_error);
        assert!(idiomatic.required_imports.iter().any(|imp| imp.contains("HashMap")));
        assert!(idiomatic.generated_rust_code.contains("ages.clear();"));
        assert!(!idiomatic.generated_rust_code.contains("// #include"));

        let faithful = convert_source_to_rust_with_style(cpp_code, &SourceLanguage::Cpp, RustOutputStyle::Faithful);
        assert!(!faithful.generated_rust_code.trim().is_empty());
        assert_ne!(idiomatic.generated_rust_code, faithful.generated_rust_code);
    }

    #[test]
    fn test_cpp_set_crud_operations() {
        let cpp_code = r#"#include <iostream>
#include <set>

int main() {
    // 1. Create & Insert
    std::set<int> unique_ids;
    unique_ids.insert(101);
    unique_ids.insert(202);
    unique_ids.insert(303);

    // 2. Read & Membership check
    bool exists = unique_ids.count(202) > 0;
    std::cout << "Exists: " << exists << ", Size: " << unique_ids.size() << std::endl;

    // 3. Delete
    unique_ids.erase(202);
    unique_ids.clear();

    return 0;
}
"#;
        let idiomatic = convert_source_to_rust_with_style(cpp_code, &SourceLanguage::Cpp, RustOutputStyle::Idiomatic);
        assert!(idiomatic.post_processing.compiler_verified, "Set CRUD Idiomatic failed rustc verification: {:?}", idiomatic.post_processing.verification_error);
        assert!(idiomatic.required_imports.iter().any(|imp| imp.contains("HashSet")));
        assert!(idiomatic.generated_rust_code.contains("unique_ids.insert(101);"));
        assert!(idiomatic.generated_rust_code.contains("unique_ids.clear();"));
        assert!(!idiomatic.generated_rust_code.contains("// #include"));

        let faithful = convert_source_to_rust_with_style(cpp_code, &SourceLanguage::Cpp, RustOutputStyle::Faithful);
        assert!(!faithful.generated_rust_code.trim().is_empty());
        assert_ne!(idiomatic.generated_rust_code, faithful.generated_rust_code);
    }

    #[test]
    fn converts_c_malloc_and_printf_program_to_verified_idiomatic_rust() {
        let source = r#"#include <stdio.h>
#include <stdlib.h>

int main() {
    int *arr = (int*)malloc(sizeof(int) * 4);

    for(int idx = 0; idx < 4; idx++){
      arr[idx] = idx + 10;
    }

    for(int idx = 0; idx < 4; idx++){
      printf("%d", arr[idx]);
    }

    return 0;
}"#;

        let result = convert_source_to_rust_with_style(
            source,
            &SourceLanguage::C,
            RustOutputStyle::Idiomatic,
        );
        if detect_external_transpiler_binary(&SourceLanguage::C).is_none() {
            return;
        }
        assert!(
            result.post_processing.compiler_verified,
            "{:?}",
            result.post_processing.verification_error
        );
        assert!(!result.generated_rust_code.contains("::core::ffi"));
        assert!(!result.generated_rust_code.contains("unsafe"));
        assert!(!result.generated_rust_code.contains("extern \"C\""));
        assert!(!result.generated_rust_code.contains("malloc"));
        assert!(!result.generated_rust_code.contains("#[no_mangle]"));
        assert!(result
            .generated_rust_code
            .contains("let mut arr: Vec<i32> = vec![Default::default(); 4];"));
        assert!(result.generated_rust_code.contains("print!(\"{}\","));
        assert!(result.generated_rust_code.contains("fn main()"));
    }

    #[test]
    fn converts_c_array_length_idiom_to_a_compiler_verified_rust_len_call() {
        let source = r#"#include <stdio.h>
int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    for (int idx = 0; idx < sizeof(arr) / sizeof(arr[0]); idx++) {
        printf("%d ", arr[idx]);
    }
    printf("\n");
    return 0;
}"#;

        let result = convert_source_to_rust_with_style(
            source,
            &SourceLanguage::C,
            RustOutputStyle::Idiomatic,
        );
        assert!(result.post_processing.compiler_verified, "{:?}", result.post_processing.verification_error);
        assert!(result.generated_rust_code.contains("arr.len()"));
        assert!(!result.generated_rust_code.contains("size_of::<arr[0]>"));
    }
}
