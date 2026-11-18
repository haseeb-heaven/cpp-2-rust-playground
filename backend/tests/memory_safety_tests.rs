use rustc_playground_backend::config::ApplicationConfiguration;
use rustc_playground_backend::executor::{execute_program, find_local_compiler};
use rustc_playground_backend::models::{
    ExecutionLanguage, ExecutionMode, ExecutionProvider, ExecutionRequest, RustOutputStyle,
    SourceLanguage,
};
use rustc_playground_backend::transpiler::convert_source_to_rust_with_style;

const OUT_OF_BOUNDS_C: &str = r#"#include <stdio.h>

int main(void) {
    int arr[5] = {10, 20, 30, 40, 50};
    arr[6] = 60;
    printf("%d\n", arr[0]);
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
async fn readable_rust_does_not_silently_accept_c_out_of_bounds_access() {
    if find_local_compiler(&ExecutionLanguage::C).is_none()
        || find_local_compiler(&ExecutionLanguage::Rust).is_none()
    {
        return;
    }

    let c_result = execute_program(
        &configuration(),
        &ExecutionRequest {
            code: OUT_OF_BOUNDS_C.to_string(),
            language: ExecutionLanguage::C,
            mode: ExecutionMode::Compile,
            stdin_input: String::new(),
            provider: Some(ExecutionProvider::Local),
            compiler_standard: None,
        },
    )
    .await;
    assert!(
        c_result.success,
        "C must compile this undefined-behavior example: {}",
        c_result.error_output
    );

    let conversion = convert_source_to_rust_with_style(
        OUT_OF_BOUNDS_C,
        &SourceLanguage::C,
        RustOutputStyle::Idiomatic,
    );
    assert!(
        conversion.generated_rust_code.contains("arr[6i32 as usize]"),
        "{}",
        conversion.generated_rust_code
    );

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
    assert!(
        !rust_result.success,
        "Rust must not run an unchecked out-of-bounds write successfully"
    );
}
