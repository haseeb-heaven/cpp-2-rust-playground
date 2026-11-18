//! Runs the Rust linter (`clippy-driver`) over generated Rust so the studio can
//! show real lint diagnostics next to the translation.

use crate::models::ClippyLint;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

static CLIPPY_BINARY: Lazy<Option<PathBuf>> = Lazy::new(resolve_clippy_binary);

/// Whether a usable `clippy-driver` was found on this machine.
pub fn clippy_available() -> bool {
    CLIPPY_BINARY.is_some()
}

/// Path to the resolved `clippy-driver`, if any.
pub fn clippy_binary() -> Option<&'static Path> {
    CLIPPY_BINARY.as_deref()
}

fn resolve_clippy_binary() -> Option<PathBuf> {
    if let Some(configured) = std::env::var_os("CLIPPY_DRIVER") {
        let candidate = PathBuf::from(configured);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    for candidate in ["clippy-driver", "/opt/homebrew/bin/clippy-driver"] {
        let path = PathBuf::from(candidate);
        if path.is_absolute() {
            if path.is_file() {
                return Some(path);
            }
            continue;
        }
        if let Some(found) = which(candidate) {
            return Some(found);
        }
    }
    None
}

fn which(program: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|directory| directory.join(program))
        .find(|candidate| candidate.is_file())
}

/// Runs clippy over `code` and returns its lint diagnostics, newest first.
pub fn run_clippy(code: &str) -> Result<Vec<ClippyLint>, String> {
    let Some(binary) = CLIPPY_BINARY.as_deref() else {
        return Err("clippy-driver is not installed (rustup component add clippy)".to_string());
    };
    if code.trim().is_empty() {
        return Ok(Vec::new());
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let directory = std::env::temp_dir().join(format!(
        "playground-clippy-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).map_err(|error| format!("cannot create temp dir: {error}"))?;
    let source_path = directory.join("generated.rs");
    let metadata_path = directory.join("generated.rmeta");
    fs::write(&source_path, code).map_err(|error| format!("cannot write source: {error}"))?;

    let output = Command::new(binary)
        .arg("--edition")
        .arg("2021")
        .arg("--crate-type")
        .arg("bin")
        .arg("--emit=metadata")
        .arg("-o")
        .arg(&metadata_path)
        .arg("--error-format=json")
        .arg("--crate-name")
        .arg("playground_generated")
        .arg("-W")
        .arg("clippy::all")
        .arg(&source_path)
        .output();

    let _ = fs::remove_dir_all(&directory);

    let output = output.map_err(|error| format!("cannot run clippy-driver: {error}"))?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(parse_clippy_diagnostics(&stderr))
}

fn parse_clippy_diagnostics(stderr: &str) -> Vec<ClippyLint> {
    let mut lints: Vec<ClippyLint> = Vec::new();
    let mut seen: Vec<(String, usize, usize)> = Vec::new();

    for line in stderr.lines() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        let Ok(diagnostic) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if diagnostic.get("$message_type").and_then(Value::as_str) != Some("diagnostic") {
            continue;
        }
        let level = diagnostic
            .get("level")
            .and_then(Value::as_str)
            .unwrap_or("warning");
        if level != "warning" && level != "error" {
            continue;
        }
        let message = diagnostic
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        // Summary lines such as "3 warnings emitted" carry no lint of their own.
        if message.is_empty()
            || message.ends_with("emitted")
            || message.starts_with("aborting due to")
        {
            continue;
        }
        let lint = diagnostic
            .get("code")
            .and_then(|code| code.get("code"))
            .and_then(Value::as_str)
            .unwrap_or("rustc")
            .to_string();

        let span = diagnostic
            .get("spans")
            .and_then(Value::as_array)
            .and_then(|spans| spans.first());
        let line_number = span
            .and_then(|span| span.get("line_start"))
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;
        let column = span
            .and_then(|span| span.get("column_start"))
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;

        if seen.contains(&(lint.clone(), line_number, column)) {
            continue;
        }
        seen.push((lint.clone(), line_number, column));
        lints.push(ClippyLint {
            line_number,
            column,
            level: level.to_string(),
            lint,
            message,
        });
    }

    lints.sort_by_key(|lint| (lint.line_number, lint.column));
    lints.truncate(40);
    lints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clippy_json_diagnostics() {
        let stderr = concat!(
            "{\"$message_type\":\"diagnostic\",\"message\":\"calls to `push` immediately after creation\",",
            "\"code\":{\"code\":\"clippy::vec_init_then_push\"},\"level\":\"warning\",",
            "\"spans\":[{\"line_start\":2,\"column_start\":5}]}\n",
            "{\"$message_type\":\"diagnostic\",\"message\":\"1 warning emitted\",\"code\":null,",
            "\"level\":\"warning\",\"spans\":[]}\n"
        );
        let lints = parse_clippy_diagnostics(stderr);
        assert_eq!(lints.len(), 1);
        assert_eq!(lints[0].lint, "clippy::vec_init_then_push");
        assert_eq!(lints[0].line_number, 2);
        assert_eq!(lints[0].column, 5);
    }

    #[test]
    fn clippy_reports_a_real_lint_for_push_after_new() {
        if !clippy_available() {
            return;
        }
        let code = "fn main() {\n    let mut values: Vec<i32> = Vec::new();\n    values.push(1);\n    println!(\"{}\", values.len());\n}\n";
        let lints = run_clippy(code).expect("clippy must run");
        assert!(
            lints.iter().any(|lint| lint.lint == "clippy::vec_init_then_push"),
            "expected vec_init_then_push, got {lints:?}"
        );
    }
}
