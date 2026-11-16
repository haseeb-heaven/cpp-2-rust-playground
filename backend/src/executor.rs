use crate::config::ApplicationConfiguration;
use crate::models::{
    ConversionDiagnostic, DiagnosticSeverity, ErrorCategory, ExecutionLanguage, ExecutionMode,
    ExecutionRequest, ExecutionResponse,
};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

static COMPILER_LINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)(?::|line\s+)(\d+)(?::\d+)?:\s*(error|warning):\s*(.+)").unwrap());

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JdoodleApiRequest {
    client_id: String,
    client_secret: String,
    script: String,
    language: String,
    version_index: String,
    stdin: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JdoodleApiResponse {
    output: Option<String>,
    status_code: Option<i32>,
    memory: Option<String>,
    cpu_time: Option<String>,
    error: Option<String>,
}

pub fn map_language_to_jdoodle(language: &ExecutionLanguage) -> (&'static str, &'static str) {
    match language {
        ExecutionLanguage::C => ("c", "5"),
        ExecutionLanguage::Cpp => ("cpp17", "1"),
        ExecutionLanguage::Rust => ("rust", "4"),
    }
}

pub fn parse_compiler_diagnostics(output_text: &str) -> Vec<ConversionDiagnostic> {
    let mut diagnostics = Vec::new();

    for captures in COMPILER_LINE_REGEX.captures_iter(output_text) {
        let line_number = captures
            .get(1)
            .and_then(|match_item| match_item.as_str().parse::<usize>().ok())
            .unwrap_or(1);
        let level_text = captures.get(2).map(|m| m.as_str()).unwrap_or("error");
        let message = captures
            .get(3)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "Compiler error".to_string());

        let severity = if level_text == "warning" {
            DiagnosticSeverity::Warning
        } else {
            DiagnosticSeverity::Error
        };

        let category = if message.to_lowercase().contains("expected") {
            ErrorCategory::Syntax
        } else if message.to_lowercase().contains("memory")
            || message.to_lowercase().contains("segmentation")
        {
            ErrorCategory::Memory
        } else {
            ErrorCategory::Compilation
        };

        diagnostics.push(ConversionDiagnostic {
            line_number,
            severity,
            category,
            message: message.clone(),
            suggested_fix: suggest_fix_for_compiler_message(&message),
        });
    }

    if diagnostics.is_empty()
        && (output_text.contains("error[E") || output_text.contains("error:"))
    {
        diagnostics.push(ConversionDiagnostic {
            line_number: 1,
            severity: DiagnosticSeverity::Error,
            category: ErrorCategory::Compilation,
            message: output_text
                .lines()
                .find(|line| line.contains("error"))
                .unwrap_or("Compilation failed")
                .trim()
                .to_string(),
            suggested_fix: Some("Check syntax, types, and missing imports.".to_string()),
        });
    }

    diagnostics
}

fn suggest_fix_for_compiler_message(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    if lower.contains("expected `;`") || lower.contains("expected ';'") {
        Some("Add a missing semicolon ';' at the end of the statement.".to_string())
    } else if lower.contains("cannot find") || lower.contains("undeclared") {
        Some("Declare the variable or add the required import/header.".to_string())
    } else if lower.contains("mismatched types") {
        Some("Ensure both sides of the expression have matching types.".to_string())
    } else if lower.contains("borrow") || lower.contains("moved") {
        Some("Consider borrowing with '&' or cloning with '.clone()'.".to_string())
    } else {
        None
    }
}

pub async fn execute_with_jdoodle(
    configuration: &ApplicationConfiguration,
    request: &ExecutionRequest,
) -> ExecutionResponse {
    let start_instant = Instant::now();

    if configuration.jdooodle_client_id.trim().is_empty()
        || configuration.jdooodle_client_secret.trim().is_empty()
    {
        return ExecutionResponse {
            success: false,
            output: String::new(),
            error_output: "JDoodle API credentials are not configured in .env.".to_string(),
            status_code: 500,
            memory_used: None,
            cpu_time_seconds: None,
            execution_time_milliseconds: start_instant.elapsed().as_millis(),
            diagnostics: vec![ConversionDiagnostic {
                line_number: 1,
                severity: DiagnosticSeverity::Error,
                category: ErrorCategory::Network,
                message: "Missing JDOODLE_CLIENT_ID or JDOODLE_CLIENT_SECRET in .env.".to_string(),
                suggested_fix: Some("Add valid JDoodle credentials to .env and restart the backend.".to_string()),
            }],
            timed_out: false,
        };
    }

    let (language_slug, version_index) = map_language_to_jdoodle(&request.language);
    let script_to_run = if request.mode == ExecutionMode::Compile {
        match request.language {
            ExecutionLanguage::Rust => format!(
                "{}\n// Compile check mode",
                request.code
            ),
            _ => request.code.clone(),
        }
    } else {
        request.code.clone()
    };

    let payload = JdoodleApiRequest {
        client_id: configuration.jdooodle_client_id.clone(),
        client_secret: configuration.jdooodle_client_secret.clone(),
        script: script_to_run,
        language: language_slug.to_string(),
        version_index: version_index.to_string(),
        stdin: if request.mode == ExecutionMode::Compile {
            String::new()
        } else {
            request.stdin_input.clone()
        },
    };

    let http_client = match Client::builder()
        .timeout(Duration::from_secs(configuration.request_timeout_seconds))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return ExecutionResponse {
                success: false,
                output: String::new(),
                error_output: format!("Failed to initialize HTTP client: {}", error),
                status_code: 500,
                memory_used: None,
                cpu_time_seconds: None,
                execution_time_milliseconds: start_instant.elapsed().as_millis(),
                diagnostics: vec![ConversionDiagnostic {
                    line_number: 1,
                    severity: DiagnosticSeverity::Error,
                    category: ErrorCategory::Network,
                    message: error.to_string(),
                    suggested_fix: None,
                }],
                timed_out: false,
            };
        }
    };

    let response_result = http_client
        .post(&configuration.jdooodle_api_url)
        .json(&payload)
        .send()
        .await;

    let response = match response_result {
        Ok(resp) => resp,
        Err(error) => {
            let timed_out = error.is_timeout();
            let category = if timed_out {
                ErrorCategory::Timeout
            } else {
                ErrorCategory::Network
            };
            return ExecutionResponse {
                success: false,
                output: String::new(),
                error_output: if timed_out {
                    "Execution request timed out.".to_string()
                } else {
                    format!("Network error calling JDoodle API: {}", error)
                },
                status_code: 504,
                memory_used: None,
                cpu_time_seconds: None,
                execution_time_milliseconds: start_instant.elapsed().as_millis(),
                diagnostics: vec![ConversionDiagnostic {
                    line_number: 1,
                    severity: DiagnosticSeverity::Error,
                    category,
                    message: error.to_string(),
                    suggested_fix: Some("Check internet connectivity or increase REQUEST_TIMEOUT_SECONDS.".to_string()),
                }],
                timed_out,
            };
        }
    };

    let http_status = response.status().as_u16() as i32;
    let parsed: JdoodleApiResponse = match response.json().await {
        Ok(data) => data,
        Err(error) => {
            return ExecutionResponse {
                success: false,
                output: String::new(),
                error_output: format!("Invalid JSON from JDoodle API: {}", error),
                status_code: http_status,
                memory_used: None,
                cpu_time_seconds: None,
                execution_time_milliseconds: start_instant.elapsed().as_millis(),
                diagnostics: vec![],
                timed_out: false,
            };
        }
    };

    let raw_output = parsed.output.unwrap_or_default();
    let api_error = parsed.error.unwrap_or_default();
    let combined_text = if api_error.is_empty() {
        raw_output.clone()
    } else {
        format!("{}\n{}", api_error, raw_output)
    };

    let diagnostics = parse_compiler_diagnostics(&combined_text);
    let has_errors = !api_error.is_empty()
        || diagnostics
            .iter()
            .any(|diag| diag.severity == DiagnosticSeverity::Error)
        || parsed.status_code.unwrap_or(200) >= 400;

    let display_output = if request.mode == ExecutionMode::Compile && !has_errors {
        "Compilation succeeded with 0 errors.".to_string()
    } else {
        raw_output
    };

    ExecutionResponse {
        success: !has_errors,
        output: if has_errors { String::new() } else { display_output },
        error_output: if has_errors { combined_text } else { String::new() },
        status_code: parsed.status_code.unwrap_or(http_status),
        memory_used: parsed.memory,
        cpu_time_seconds: parsed.cpu_time,
        execution_time_milliseconds: start_instant.elapsed().as_millis(),
        diagnostics,
        timed_out: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_languages_to_jdoodle_slugs() {
        assert_eq!(map_language_to_jdoodle(&ExecutionLanguage::C), ("c", "5"));
        assert_eq!(
            map_language_to_jdoodle(&ExecutionLanguage::Cpp),
            ("cpp17", "1")
        );
        assert_eq!(
            map_language_to_jdoodle(&ExecutionLanguage::Rust),
            ("rust", "4")
        );
    }

    #[test]
    fn parses_gcc_and_rustc_diagnostics() {
        let compiler_output = "main.cpp:4:12: error: expected ';' before 'return'\n";
        let diagnostics = parse_compiler_diagnostics(compiler_output);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line_number, 4);
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert!(diagnostics[0].suggested_fix.is_some());
    }
}
