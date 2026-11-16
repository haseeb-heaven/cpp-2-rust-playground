use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceLanguage {
    C,
    Cpp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionLanguage {
    C,
    Cpp,
    Rust,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    Compile,
    Run,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ErrorCategory {
    Syntax,
    Compilation,
    Runtime,
    Conversion,
    Network,
    Input,
    Memory,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionDiagnostic {
    pub line_number: usize,
    pub severity: DiagnosticSeverity,
    pub category: ErrorCategory,
    pub message: String,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineMapping {
    pub source_line: usize,
    pub rust_start_line: usize,
    pub rust_end_line: usize,
    pub source_text: String,
    pub rust_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRequest {
    pub source_code: String,
    pub source_language: SourceLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResponse {
    pub generated_rust_code: String,
    pub line_mappings: Vec<LineMapping>,
    pub diagnostics: Vec<ConversionDiagnostic>,
    pub required_imports: Vec<String>,
    pub conversion_time_milliseconds: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRequest {
    pub code: String,
    pub language: ExecutionLanguage,
    pub mode: ExecutionMode,
    pub stdin_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResponse {
    pub success: bool,
    pub output: String,
    pub error_output: String,
    pub status_code: i32,
    pub memory_used: Option<String>,
    pub cpu_time_seconds: Option<String>,
    pub execution_time_milliseconds: u128,
    pub diagnostics: Vec<ConversionDiagnostic>,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeExample {
    pub identifier: String,
    pub title: String,
    pub category: String,
    pub source_language: SourceLanguage,
    pub description: String,
    pub source_code: String,
    pub default_stdin: String,
}
