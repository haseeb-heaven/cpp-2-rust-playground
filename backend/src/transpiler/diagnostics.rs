use crate::models::{ConversionDiagnostic, DiagnosticSeverity, ErrorCategory};

pub fn analyze_source_diagnostics(source_lines: &[&str]) -> Vec<ConversionDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut brace_stack: Vec<usize> = Vec::new();

    for (zero_based_index, raw_line) in source_lines.iter().enumerate() {
        let line_number = zero_based_index + 1;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        for character in trimmed.chars() {
            if character == '{' {
                brace_stack.push(line_number);
            } else if character == '}' {
                if brace_stack.pop().is_none() {
                    diagnostics.push(ConversionDiagnostic {
                        line_number,
                        severity: DiagnosticSeverity::Error,
                        category: ErrorCategory::Syntax,
                        message: "Unexpected closing brace '}' with no matching opening brace."
                            .to_string(),
                        suggested_fix: Some("Remove the extra '}' or add a matching '{'.".to_string()),
                    });
                }
            }
        }

        if trimmed.contains("malloc(") || trimmed.contains("calloc(") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Memory,
                message: "Manual heap allocation detected (malloc/calloc).".to_string(),
                suggested_fix: Some(
                    "Use Rust's Vec::with_capacity() or Box::new() for safe heap allocation."
                        .to_string(),
                ),
            });
        }

        if trimmed.contains("gets(") || trimmed.contains("strcpy(") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Memory,
                message: "Unsafe C string function detected (gets/strcpy).".to_string(),
                suggested_fix: Some(
                    "Use String::from() or std::io::stdin().read_line() in Rust.".to_string(),
                ),
            });
        }

        if trimmed.starts_with("goto ") {
            diagnostics.push(ConversionDiagnostic {
                line_number,
                severity: DiagnosticSeverity::Warning,
                category: ErrorCategory::Conversion,
                message: "'goto' statements are not supported in Rust.".to_string(),
                suggested_fix: Some(
                    "Refactor using a labeled loop ('label: loop { ... break 'label; })."
                        .to_string(),
                ),
            });
        }
    }

    for unclosed_line in brace_stack {
        diagnostics.push(ConversionDiagnostic {
            line_number: unclosed_line,
            severity: DiagnosticSeverity::Error,
            category: ErrorCategory::Syntax,
            message: "Unclosed opening brace '{'.".to_string(),
            suggested_fix: Some("Add a matching closing brace '}'.".to_string()),
        });
    }

    diagnostics
}
