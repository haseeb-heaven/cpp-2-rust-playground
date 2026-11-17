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
pub enum ExecutionProvider {
    Local,
    Online,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RustOutputStyle {
    Faithful,
    #[default]
    Idiomatic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostProcessingSummary {
    pub applied_passes: Vec<String>,
    pub rejected_passes: Vec<String>,
    pub formatted_with_rustfmt: bool,
    pub compiler_verified: bool,
    pub verification_error: Option<String>,
    pub unsafe_blocks_removed: usize,
    pub redundant_mut_removed: usize,
    pub redundant_casts_removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionExplanation {
    pub construct: String,
    pub cpp_pattern: String,
    pub rust_equivalent: String,
    pub why_chosen: String,
    pub ownership_and_iteration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRequest {
    pub source_code: String,
    pub source_language: SourceLanguage,
    #[serde(default)]
    pub output_style: RustOutputStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResponse {
    pub generated_rust_code: String,
    pub faithful_rust_code: String,
    pub idiomatic_rust_code: String,
    pub output_style: RustOutputStyle,
    pub line_mappings: Vec<LineMapping>,
    pub diagnostics: Vec<ConversionDiagnostic>,
    pub required_imports: Vec<String>,
    pub conversion_time_milliseconds: u128,
    pub transpiler_engine: String,
    pub used_fallback: bool,
    pub fallback_reason: Option<String>,
    pub unsupported_constructs: Vec<String>,
    pub post_processing: PostProcessingSummary,
    pub explanations: Vec<ConversionExplanation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionRequest {
    pub code: String,
    pub language: ExecutionLanguage,
    pub mode: ExecutionMode,
    pub stdin_input: String,
    #[serde(default)]
    pub provider: Option<ExecutionProvider>,
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
    pub used_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerAvailability {
    pub c_available: bool,
    pub c_compiler: Option<String>,
    pub cpp_available: bool,
    pub cpp_compiler: Option<String>,
    pub rust_available: bool,
    pub rust_compiler: Option<String>,
    pub c2rust_available: bool,
    pub c2rust_binary: Option<String>,
    pub cpp2rust_available: bool,
    pub cpp2rust_binary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeExample {
    pub identifier: String,
    pub title: String,
    pub category: String,
    pub difficulty: String,
    pub source_language: SourceLanguage,
    pub description: String,
    pub source_code: String,
    pub default_stdin: String,
}
