use crate::config::ApplicationConfiguration;
use crate::models::{
    CompilerAvailability, ConversionDiagnostic, DiagnosticSeverity, ErrorCategory, ExecutionLanguage,
    ExecutionMode, ExecutionProvider, ExecutionRequest, ExecutionResponse,
};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

static COMPILER_LINE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)(?::|line\s+)(\d+)(?::\d+)?:\s*(error|warning):\s*(.+)").unwrap());
static RUSTC_ARROW_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)(error(?:\[E\d+\])?|warning):\s*(.+)\n\s*-->\s*[^:\n]+:(\d+):(\d+)").unwrap());
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

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

struct TempDirectoryGuard {
    directory_path: PathBuf,
}

impl TempDirectoryGuard {
    fn create() -> std::io::Result<Self> {
        let unique_id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let directory_path = std::env::temp_dir().join(format!(
            "rustc_playground_{}_{}",
            std::process::id(),
            unique_id
        ));
        fs::create_dir_all(&directory_path)?;
        Ok(Self { directory_path })
    }
}

impl Drop for TempDirectoryGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory_path);
    }
}

fn check_command_exists(command_or_path: &str) -> bool {
    std::process::Command::new(command_or_path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn find_local_compiler(language: &ExecutionLanguage) -> Option<String> {
    let home_dir = std::env::var("HOME").unwrap_or_default();
    let candidates: Vec<String> = match language {
        ExecutionLanguage::C => vec![
            "gcc".to_string(),
            "clang".to_string(),
            "/usr/bin/gcc".to_string(),
            "/opt/homebrew/bin/gcc".to_string(),
            "/usr/local/bin/gcc".to_string(),
            "/opt/homebrew/opt/llvm/bin/clang".to_string(),
            "/usr/bin/clang".to_string(),
        ],
        ExecutionLanguage::Cpp => vec![
            "g++".to_string(),
            "clang++".to_string(),
            "/usr/bin/g++".to_string(),
            "/opt/homebrew/bin/g++".to_string(),
            "/usr/local/bin/g++".to_string(),
            "/opt/homebrew/opt/llvm/bin/clang++".to_string(),
            "/usr/bin/clang++".to_string(),
        ],
        ExecutionLanguage::Rust => {
            let mut rust_paths = vec![
                "rustc".to_string(),
                "/opt/homebrew/bin/rustc".to_string(),
                "/usr/local/bin/rustc".to_string(),
                "/usr/bin/rustc".to_string(),
            ];
            if !home_dir.is_empty() {
                rust_paths.push(format!("{}/.cargo/bin/rustc", home_dir));
            }
            rust_paths
        }
    };

    candidates
        .into_iter()
        .find(|candidate| check_command_exists(candidate))
}

pub fn detect_local_compilers() -> CompilerAvailability {
    let c_compiler = find_local_compiler(&ExecutionLanguage::C);
    let cpp_compiler = find_local_compiler(&ExecutionLanguage::Cpp);
    let rust_compiler = find_local_compiler(&ExecutionLanguage::Rust);

    CompilerAvailability {
        c_available: c_compiler.is_some(),
        c_compiler,
        cpp_available: cpp_compiler.is_some(),
        cpp_compiler,
        rust_available: rust_compiler.is_some(),
        rust_compiler,
    }
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

    for captures in RUSTC_ARROW_REGEX.captures_iter(output_text) {
        let level_text = captures.get(1).map(|m| m.as_str()).unwrap_or("error");
        let message = captures
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| "Rust compiler error".to_string());
        let line_number = captures
            .get(3)
            .and_then(|match_item| match_item.as_str().parse::<usize>().ok())
            .unwrap_or(1);

        let severity = if level_text.starts_with("warning") {
            DiagnosticSeverity::Warning
        } else {
            DiagnosticSeverity::Error
        };

        diagnostics.push(ConversionDiagnostic {
            line_number,
            severity,
            category: ErrorCategory::Compilation,
            message: message.clone(),
            suggested_fix: suggest_fix_for_compiler_message(&message),
        });
    }

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
    } else if lower.contains("invalid preprocessing directive") {
        Some("Fix the preprocessor directive spelling (for example '#include').".to_string())
    } else {
        None
    }
}

pub async fn execute_program(
    configuration: &ApplicationConfiguration,
    request: &ExecutionRequest,
) -> ExecutionResponse {
    match request.provider {
        Some(ExecutionProvider::Online) => {
            tracing::info!(
                language = ?request.language,
                mode = ?request.mode,
                provider = "online",
                "Executing code via JDoodle Online API"
            );
            execute_with_jdoodle(configuration, request, "online (JDoodle)").await
        }
        Some(ExecutionProvider::Local) | None => {
            if let Some(compiler_name) = find_local_compiler(&request.language) {
                tracing::info!(
                    language = ?request.language,
                    mode = ?request.mode,
                    compiler = %compiler_name,
                    "Executing code using local compiler"
                );
                execute_locally(configuration, request, &compiler_name).await
            } else {
                tracing::warn!(
                    language = ?request.language,
                    mode = ?request.mode,
                    "No local compiler found; falling back to JDoodle Online API"
                );
                execute_with_jdoodle(
                    configuration,
                    request,
                    "online (JDoodle fallback - local compiler unavailable)",
                )
                .await
            }
        }
    }
}

async fn execute_locally(
    configuration: &ApplicationConfiguration,
    request: &ExecutionRequest,
    compiler_name: &str,
) -> ExecutionResponse {
    let start_instant = Instant::now();
    let used_provider = format!("local ({})", compiler_name);

    let temp_guard = match TempDirectoryGuard::create() {
        Ok(guard) => guard,
        Err(error) => {
            return ExecutionResponse {
                success: false,
                output: String::new(),
                error_output: format!("Failed to create temporary build directory: {}", error),
                status_code: 500,
                memory_used: None,
                cpu_time_seconds: None,
                execution_time_milliseconds: start_instant.elapsed().as_millis(),
                diagnostics: vec![],
                timed_out: false,
                used_provider,
            };
        }
    };

    let source_filename = match request.language {
        ExecutionLanguage::C => "main.c",
        ExecutionLanguage::Cpp => "main.cpp",
        ExecutionLanguage::Rust => "main.rs",
    };
    let source_path = temp_guard.directory_path.join(source_filename);
    let binary_path = temp_guard.directory_path.join("program_bin");

    if let Err(error) = fs::write(&source_path, &request.code) {
        return ExecutionResponse {
            success: false,
            output: String::new(),
            error_output: format!("Failed to write source file: {}", error),
            status_code: 500,
            memory_used: None,
            cpu_time_seconds: None,
            execution_time_milliseconds: start_instant.elapsed().as_millis(),
            diagnostics: vec![],
            timed_out: false,
            used_provider,
        };
    }

    let mut compile_command = Command::new(compiler_name);
    match request.language {
        ExecutionLanguage::C => {
            compile_command.arg(&source_path).arg("-o").arg(&binary_path);
        }
        ExecutionLanguage::Cpp => {
            compile_command
                .arg("-std=c++17")
                .arg(&source_path)
                .arg("-o")
                .arg(&binary_path);
        }
        ExecutionLanguage::Rust => {
            compile_command.arg(&source_path).arg("-o").arg(&binary_path);
        }
    }

    let timeout_duration = Duration::from_secs(configuration.request_timeout_seconds);
    let compile_output = match tokio::time::timeout(timeout_duration, compile_command.output()).await
    {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => {
            return ExecutionResponse {
                success: false,
                output: String::new(),
                error_output: format!("Failed to invoke compiler '{}': {}", compiler_name, error),
                status_code: 500,
                memory_used: None,
                cpu_time_seconds: None,
                execution_time_milliseconds: start_instant.elapsed().as_millis(),
                diagnostics: vec![],
                timed_out: false,
                used_provider,
            };
        }
        Err(_) => {
            return ExecutionResponse {
                success: false,
                output: String::new(),
                error_output: "Local compilation timed out.".to_string(),
                status_code: 504,
                memory_used: None,
                cpu_time_seconds: None,
                execution_time_milliseconds: start_instant.elapsed().as_millis(),
                diagnostics: vec![ConversionDiagnostic {
                    line_number: 1,
                    severity: DiagnosticSeverity::Error,
                    category: ErrorCategory::Timeout,
                    message: "Compilation exceeded timeout limit.".to_string(),
                    suggested_fix: Some("Check for infinite template recursion or increase timeout.".to_string()),
                }],
                timed_out: true,
                used_provider,
            };
        }
    };

    let compile_stderr = String::from_utf8_lossy(&compile_output.stderr).to_string();
    let diagnostics = parse_compiler_diagnostics(&compile_stderr);

    if !compile_output.status.success() {
        return ExecutionResponse {
            success: false,
            output: String::new(),
            error_output: compile_stderr,
            status_code: compile_output.status.code().unwrap_or(1),
            memory_used: None,
            cpu_time_seconds: None,
            execution_time_milliseconds: start_instant.elapsed().as_millis(),
            diagnostics,
            timed_out: false,
            used_provider,
        };
    }

    if request.mode == ExecutionMode::Compile {
        return ExecutionResponse {
            success: true,
            output: format!("Compilation succeeded using {}.", used_provider),
            error_output: String::new(),
            status_code: 0,
            memory_used: None,
            cpu_time_seconds: None,
            execution_time_milliseconds: start_instant.elapsed().as_millis(),
            diagnostics,
            timed_out: false,
            used_provider,
        };
    }

    let mut run_command = Command::new(&binary_path);
    run_command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match run_command.spawn() {
        Ok(process) => process,
        Err(error) => {
            return ExecutionResponse {
                success: false,
                output: String::new(),
                error_output: format!("Failed to start compiled binary: {}", error),
                status_code: 500,
                memory_used: None,
                cpu_time_seconds: None,
                execution_time_milliseconds: start_instant.elapsed().as_millis(),
                diagnostics,
                timed_out: false,
                used_provider,
            };
        }
    };

    if let Some(mut stdin_pipe) = child.stdin.take() {
        let _ = stdin_pipe.write_all(request.stdin_input.as_bytes()).await;
    }

    match tokio::time::timeout(timeout_duration, child.wait_with_output()).await {
        Ok(Ok(run_output)) => {
            let stdout_text = String::from_utf8_lossy(&run_output.stdout).to_string();
            let stderr_text = String::from_utf8_lossy(&run_output.stderr).to_string();
            let exit_code = run_output.status.code().unwrap_or(-1);
            let succeeded = run_output.status.success();

            ExecutionResponse {
                success: succeeded,
                output: stdout_text,
                error_output: if succeeded { String::new() } else { stderr_text },
                status_code: exit_code,
                memory_used: None,
                cpu_time_seconds: None,
                execution_time_milliseconds: start_instant.elapsed().as_millis(),
                diagnostics,
                timed_out: false,
                used_provider,
            }
        }
        Ok(Err(error)) => ExecutionResponse {
            success: false,
            output: String::new(),
            error_output: format!("Runtime failure: {}", error),
            status_code: 500,
            memory_used: None,
            cpu_time_seconds: None,
            execution_time_milliseconds: start_instant.elapsed().as_millis(),
            diagnostics,
            timed_out: false,
            used_provider,
        },
        Err(_) => ExecutionResponse {
            success: false,
            output: String::new(),
            error_output: "Program execution timed out.".to_string(),
            status_code: 504,
            memory_used: None,
            cpu_time_seconds: None,
            execution_time_milliseconds: start_instant.elapsed().as_millis(),
            diagnostics: vec![ConversionDiagnostic {
                line_number: 1,
                severity: DiagnosticSeverity::Error,
                category: ErrorCategory::Timeout,
                message: "Program exceeded execution timeout.".to_string(),
                suggested_fix: Some("Check for infinite loops or blocking stdin reads.".to_string()),
            }],
            timed_out: true,
            used_provider,
        },
    }
}

pub async fn execute_with_jdoodle(
    configuration: &ApplicationConfiguration,
    request: &ExecutionRequest,
    provider_label: &str,
) -> ExecutionResponse {
    let start_instant = Instant::now();
    let used_provider = provider_label.to_string();

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
            used_provider,
        };
    }

    let (language_slug, version_index) = map_language_to_jdoodle(&request.language);
    let payload = JdoodleApiRequest {
        client_id: configuration.jdooodle_client_id.clone(),
        client_secret: configuration.jdooodle_client_secret.clone(),
        script: request.code.clone(),
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
                used_provider,
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
                    suggested_fix: Some("Check internet connectivity or switch to Local compiler mode.".to_string()),
                }],
                timed_out,
                used_provider,
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
                used_provider,
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
        used_provider,
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

    #[test]
    fn parses_rustc_arrow_diagnostics() {
        let rustc_output = "error[E0425]: cannot find value `fcvec` in this scope\n --> main.rs:8:17\n";
        let diagnostics = parse_compiler_diagnostics(rustc_output);
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].line_number, 8);
        assert!(diagnostics[0].message.contains("fcvec"));
    }

    #[tokio::test]
    async fn executes_valid_rust_locally_when_rustc_is_available() {
        if find_local_compiler(&ExecutionLanguage::Rust).is_none() {
            return;
        }
        let configuration = ApplicationConfiguration {
            jdooodle_client_id: String::new(),
            jdooodle_client_secret: String::new(),
            jdooodle_api_url: "https://api.jdoodle.com/v1/execute".to_string(),
            backend_port: 8080,
            request_timeout_seconds: 10,
        };
        let request = ExecutionRequest {
            code: r#"fn main() {
    let mut fvec: Vec<f32> = Vec::new();
    fvec.push(10.5);
    fvec.push(30.5);
    fvec.push(155.664);
    for vec in &mut fvec {
        println!("{}", vec);
    }
}"#
            .to_string(),
            language: ExecutionLanguage::Rust,
            mode: ExecutionMode::Run,
            stdin_input: String::new(),
            provider: Some(ExecutionProvider::Local),
        };

        let response = execute_program(&configuration, &request).await;
        assert!(response.success, "Expected local run to succeed: {}", response.error_output);
        assert!(response.used_provider.starts_with("local"));
        assert!(response.output.contains("10.5"));
        assert!(response.output.contains("155.664"));
    }

    #[tokio::test]
    async fn executes_valid_c_and_cpp_locally_without_falling_back_to_online() {
        let configuration = ApplicationConfiguration {
            jdooodle_client_id: String::new(),
            jdooodle_client_secret: String::new(),
            jdooodle_api_url: "https://api.jdoodle.com/v1/execute".to_string(),
            backend_port: 8080,
            request_timeout_seconds: 10,
        };

        if find_local_compiler(&ExecutionLanguage::C).is_some() {
            let c_response = execute_program(
                &configuration,
                &ExecutionRequest {
                    code: "#include <stdio.h>\nint main() { printf(\"c-local-ok\\n\"); return 0; }"
                        .to_string(),
                    language: ExecutionLanguage::C,
                    mode: ExecutionMode::Run,
                    stdin_input: String::new(),
                    provider: Some(ExecutionProvider::Local),
                },
            )
            .await;
            assert!(c_response.success);
            assert!(c_response.used_provider.starts_with("local ("));
            assert_eq!(c_response.output.trim(), "c-local-ok");
        }

        if find_local_compiler(&ExecutionLanguage::Cpp).is_some() {
            let cpp_response = execute_program(
                &configuration,
                &ExecutionRequest {
                    code: "#include <iostream>\nint main() { std::cout << \"cpp-local-ok\" << std::endl; return 0; }"
                        .to_string(),
                    language: ExecutionLanguage::Cpp,
                    mode: ExecutionMode::Run,
                    stdin_input: String::new(),
                    provider: Some(ExecutionProvider::Local),
                },
            )
            .await;
            assert!(cpp_response.success);
            assert!(cpp_response.used_provider.starts_with("local ("));
            assert_eq!(cpp_response.output.trim(), "cpp-local-ok");
        }
    }
}
