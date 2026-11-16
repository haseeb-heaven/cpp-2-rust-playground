pub mod diagnostics;
pub mod line_translator;

use crate::models::{ConversionResponse, LineMapping, SourceLanguage};
use diagnostics::analyze_source_diagnostics;
use line_translator::{translate_single_line, TranslatorState};
use std::time::Instant;

pub fn convert_source_to_rust(
    source_code: &str,
    _source_language: &SourceLanguage,
) -> ConversionResponse {
    let start_instant = Instant::now();
    let source_lines: Vec<&str> = source_code.lines().collect();
    let diagnostics = analyze_source_diagnostics(&source_lines);

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
        assert!(result.generated_rust_code.contains("impl BoxValue {"));
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
}
