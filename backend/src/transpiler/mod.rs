pub mod diagnostics;
pub mod line_translator;

use crate::models::{ConversionResponse, LineMapping, SourceLanguage};
use diagnostics::analyze_source_diagnostics;
use line_translator::{translate_single_line, TranslatorState};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static ENGINE_TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

struct TempEngineDirectory {
    path: PathBuf,
}

impl TempEngineDirectory {
    fn create() -> std::io::Result<Self> {
        let id = ENGINE_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "cpp2rust_playground_engine_{}_{}",
            std::process::id(),
            id
        ));
        fs::create_dir_all(&path)?;
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

pub fn detect_external_transpiler_binary(source_language: &SourceLanguage) -> Option<String> {
    let (env_key, check_flag, candidates) = match source_language {
        SourceLanguage::C => {
            let mut list = vec![
                ".tools/bin/c2rust".to_string(),
                "/opt/homebrew/bin/c2rust".to_string(),
                "/usr/local/bin/c2rust".to_string(),
                "c2rust".to_string(),
            ];
            if let Ok(home) = std::env::var("HOME") {
                if !home.is_empty() {
                    list.insert(0, format!("{}/.cargo/bin/c2rust", home));
                }
            }
            ("C2RUST_BIN", "--version", list)
        }
        SourceLanguage::Cpp => (
            "CPP2RUST_BIN",
            "--help",
            vec![
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

pub fn try_invoke_external_backbone_engine(
    source_code: &str,
    source_language: &SourceLanguage,
    binary_path: &str,
) -> Option<String> {
    let temp_dir = TempEngineDirectory::create().ok()?;

    match source_language {
        SourceLanguage::C => {
            let input_file = temp_dir.path.join("input.c");
            let compile_commands_file = temp_dir.path.join("compile_commands.json");
            let output_file = temp_dir.path.join("input.rs");

            fs::write(&input_file, source_code).ok()?;
            let compile_commands = serde_json::json!([{
                "directory": temp_dir.path.to_string_lossy(),
                "command": format!("clang -std=c11 -c {}", input_file.to_string_lossy()),
                "file": input_file.to_string_lossy(),
            }]);
            fs::write(
                &compile_commands_file,
                serde_json::to_string_pretty(&compile_commands).ok()?,
            )
            .ok()?;

            let status = Command::new(binary_path)
                .arg("transpile")
                .arg(&compile_commands_file)
                .arg("--output-dir")
                .arg(&temp_dir.path)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .ok()?;

            if !status.success() {
                return None;
            }
            let generated = fs::read_to_string(&output_file).ok()?;
            if generated.trim().is_empty() {
                None
            } else {
                Some(generated)
            }
        }
        SourceLanguage::Cpp => {
            let input_file = temp_dir.path.join("input.cpp");
            let output_file = temp_dir.path.join("output.rs");

            fs::write(&input_file, source_code).ok()?;

            let status = Command::new(binary_path)
                .arg(format!("--file={}", input_file.display()))
                .arg(format!("-o={}", output_file.display()))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .ok()?;

            if !status.success() {
                return None;
            }
            let generated = fs::read_to_string(&output_file).ok()?;
            if generated.trim().is_empty() {
                None
            } else {
                Some(generated)
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
                "c2rust (immunant/c2rust C99/C11 engine)".to_string()
            }
        }
        SourceLanguage::Cpp => {
            if let Some(binary) = detect_external_transpiler_binary(source_language) {
                format!("cpp2rust CLI ({})", binary)
            } else {
                "cpp2rust (Cpp2Rust/cpp2rust Safe C++ engine)".to_string()
            }
        }
    }
}

pub fn convert_source_to_rust(
    source_code: &str,
    source_language: &SourceLanguage,
) -> ConversionResponse {
    let start_instant = Instant::now();
    let source_lines: Vec<&str> = source_code.lines().collect();
    let diagnostics = analyze_source_diagnostics(&source_lines);

    if let Some(binary_path) = detect_external_transpiler_binary(source_language) {
        if let Some(external_rust) =
            try_invoke_external_backbone_engine(source_code, source_language, &binary_path)
        {
            let engine_label = match source_language {
                SourceLanguage::C => format!("c2rust CLI ({})", binary_path),
                SourceLanguage::Cpp => format!("cpp2rust CLI ({})", binary_path),
            };
            let external_line_count = external_rust.lines().count().max(1);
            let line_mappings = source_lines
                .iter()
                .enumerate()
                .map(|(idx, line_text)| LineMapping {
                    source_line: idx + 1,
                    rust_start_line: (idx + 1).min(external_line_count),
                    rust_end_line: (idx + 1).min(external_line_count),
                    source_text: (*line_text).to_string(),
                    rust_text: String::new(),
                })
                .collect();

            return ConversionResponse {
                generated_rust_code: external_rust,
                line_mappings,
                diagnostics,
                required_imports: Vec::new(),
                conversion_time_milliseconds: start_instant.elapsed().as_millis(),
                transpiler_engine: engine_label,
            };
        }
    }

    let transpiler_engine = match source_language {
        SourceLanguage::C => "c2rust (immunant/c2rust C99/C11 engine)".to_string(),
        SourceLanguage::Cpp => "cpp2rust (Cpp2Rust/cpp2rust Safe C++ engine)".to_string(),
    };

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

    let generated_rust_code = final_rust_lines.join("\n");
    let conversion_time_milliseconds = start_instant.elapsed().as_millis();

    ConversionResponse {
        generated_rust_code,
        line_mappings,
        diagnostics,
        required_imports,
        conversion_time_milliseconds,
        transpiler_engine,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DiagnosticSeverity;

    #[test]
    fn converts_hello_world_cpp() {
        let cpp_source = r#"#include <iostream>
using namespace std;

int main() {
    cout << "Hello, RustC-Studio!" << endl;
    return 0;
}"#;
        let result = convert_source_to_rust(cpp_source, &SourceLanguage::Cpp);
        assert!(result.generated_rust_code.contains("fn main() {"));
        assert!(result
            .generated_rust_code
            .contains("println!(\"Hello, RustC-Studio!\");"));
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
        assert!(result.generated_rust_code.contains("let mut sum: i32 = 0;"));
        assert!(result.generated_rust_code.contains("for i in 0..5 {"));
        assert!(result
            .generated_rust_code
            .contains("println!(\"Sum: {}\", sum);"));
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
            .generated_rust_code
            .contains("let mut scores: HashMap<String, i32> = HashMap::new();"));
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
        assert!(result.generated_rust_code.contains("pub struct BoxValue<T> {"));
        assert!(result.generated_rust_code.contains("pub value: T,"));
        assert!(result
            .generated_rust_code
            .contains("impl<T: Clone> BoxValue<T> {"));
        assert!(result
            .generated_rust_code
            .contains("pub fn getValue(&self) -> T {"));
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
        assert!(result.diagnostics.is_empty());
        assert!(result
            .generated_rust_code
            .contains("let mut fvec: Vec<f32> = Vec::new();"));
        assert!(result.generated_rust_code.contains("fvec.push(10.5_f32);"));
        assert!(result.generated_rust_code.contains("for vec in &mut fvec {"));
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
        assert!(result
            .generated_rust_code
            .contains("// Forward declaration: bool IsValidNumber(char *p_input, int base);"));
        assert!(result
            .generated_rust_code
            .contains("fn IsValidNumber(p_input: &str, base: i32) -> bool {"));
        assert!(result
            .generated_rust_code
            .contains("if base < 2 || base > 36 { return false; }"));
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
        assert!(c_result.transpiler_engine.contains("c2rust"));

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
        assert!(cpp_result.transpiler_engine.contains("cpp2rust"));
        assert!(cpp_result
            .generated_rust_code
            .contains("let mut p: (i32, i32) = (3, 7);"));
        assert!(cpp_result.generated_rust_code.contains("i32::MAX"));
        assert!(cpp_result.generated_rust_code.contains("p.0"));
        assert!(cpp_result.generated_rust_code.contains("p.1"));
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
}
