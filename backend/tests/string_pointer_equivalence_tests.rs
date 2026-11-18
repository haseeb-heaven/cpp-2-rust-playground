use rustc_playground_backend::config::ApplicationConfiguration;
use rustc_playground_backend::executor::{execute_program, find_local_compiler};
use rustc_playground_backend::models::{
    ExecutionLanguage, ExecutionMode, ExecutionProvider, ExecutionRequest, RustOutputStyle,
    SourceLanguage,
};
use rustc_playground_backend::transpiler::convert_source_to_rust_with_style;

const STRING_POINTER_SOURCE: &str = r#"#include <stdio.h>
#include <string.h>

int main(void) {
    const char *str1 = "I am Heaven";
    const char *str2 = "Heaven";
    const char *ptr = str2;

    if (strstr(str1, str2) != 0) {
        str2 = "Denver";
        printf("Found %s\n", ptr);
    }
    return 0;
}"#;

fn configuration() -> ApplicationConfiguration {
    ApplicationConfiguration {
        jdooodle_client_id: String::new(),
        jdooodle_client_secret: String::new(),
        jdooodle_api_url: "https://api.jdoodle.com/v1/execute".to_string(),
        backend_port: 8080,
        request_timeout_seconds: 15,
    }
}

#[tokio::test]
async fn string_pointer_aliasing_preserves_c_output_in_readable_rust() {
    if find_local_compiler(&ExecutionLanguage::C).is_none()
        || find_local_compiler(&ExecutionLanguage::Rust).is_none()
    {
        return;
    }

    let conversion = convert_source_to_rust_with_style(
        STRING_POINTER_SOURCE,
        &SourceLanguage::C,
        RustOutputStyle::Idiomatic,
    );
    assert!(
        conversion.post_processing.compiler_verified,
        "{:?}\n{}",
        conversion.post_processing.verification_error,
        conversion.generated_rust_code
    );
    assert!(conversion.generated_rust_code.contains("let ptr: &str = str2;"));

    let c_result = execute_program(
        &configuration(),
        &ExecutionRequest {
            code: STRING_POINTER_SOURCE.to_string(),
            language: ExecutionLanguage::C,
            mode: ExecutionMode::Run,
            stdin_input: String::new(),
            provider: Some(ExecutionProvider::Local),
            compiler_standard: None,
        },
    )
    .await;
    assert!(c_result.success, "{}", c_result.error_output);

    let rust_result = execute_program(
        &configuration(),
        &ExecutionRequest {
            code: conversion.generated_rust_code,
            language: ExecutionLanguage::Rust,
            mode: ExecutionMode::Run,
            stdin_input: String::new(),
            provider: Some(ExecutionProvider::Local),
            compiler_standard: None,
        },
    )
    .await;
    assert!(rust_result.success, "{}", rust_result.error_output);
    assert_eq!(c_result.output, "Found Heaven\n");
    assert_eq!(rust_result.output, c_result.output);
}
