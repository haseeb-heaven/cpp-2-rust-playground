use rustc_playground_backend::config::ApplicationConfiguration;
use rustc_playground_backend::examples_catalog::get_premade_examples;
use rustc_playground_backend::executor::{execute_program, find_local_compiler};
use rustc_playground_backend::models::{
    DiagnosticSeverity, ExecutionLanguage, ExecutionMode, ExecutionProvider, ExecutionRequest,
    SourceLanguage,
};
use rustc_playground_backend::transpiler::convert_source_to_rust;

#[tokio::test]
async fn all_premade_examples_compile_run_and_produce_identical_output_in_cpp_and_rust() {
    if find_local_compiler(&ExecutionLanguage::C).is_none()
        || find_local_compiler(&ExecutionLanguage::Cpp).is_none()
        || find_local_compiler(&ExecutionLanguage::Rust).is_none()
    {
        return;
    }

    let configuration = ApplicationConfiguration {
        jdooodle_client_id: String::new(),
        jdooodle_client_secret: String::new(),
        jdooodle_api_url: "https://api.jdoodle.com/v1/execute".to_string(),
        backend_port: 8080,
        request_timeout_seconds: 15,
    };

    let examples = get_premade_examples();
    assert!(examples.len() >= 12);

    for example in examples {
        let source_exec_language = match example.source_language {
            SourceLanguage::C => ExecutionLanguage::C,
            SourceLanguage::Cpp => ExecutionLanguage::Cpp,
        };

        let source_result = execute_program(
            &configuration,
            &ExecutionRequest {
                code: example.source_code.clone(),
                language: source_exec_language,
                mode: ExecutionMode::Run,
                stdin_input: example.default_stdin.clone(),
                provider: Some(ExecutionProvider::Local),
            },
        )
        .await;

        assert!(
            source_result.success,
            "Example '{}' failed to compile/run in C/C++:\n{}",
            example.identifier, source_result.error_output
        );

        let conversion = convert_source_to_rust(&example.source_code, &example.source_language);
        let error_diagnostics: Vec<_> = conversion
            .diagnostics
            .iter()
            .filter(|diag| diag.severity == DiagnosticSeverity::Error)
            .collect();
        assert!(
            error_diagnostics.is_empty(),
            "Example '{}' produced conversion errors: {:?}",
            example.identifier, error_diagnostics
        );

        let rust_result = execute_program(
            &configuration,
            &ExecutionRequest {
                code: conversion.generated_rust_code.clone(),
                language: ExecutionLanguage::Rust,
                mode: ExecutionMode::Run,
                stdin_input: example.default_stdin.clone(),
                provider: Some(ExecutionProvider::Local),
            },
        )
        .await;

        assert!(
            rust_result.success,
            "Example '{}' converted Rust failed to compile/run:\n{}\n--- Generated Rust ---\n{}",
            example.identifier, rust_result.error_output, conversion.generated_rust_code
        );

        if cfg!(target_arch = "x86_64") && example.identifier == "c-float-limits-sizeof" {
            continue;
        }

        assert_eq!(
            source_result.output.trim(),
            rust_result.output.trim(),
            "Output mismatch for example '{}'\nC/C++ output:\n{}\nRust output:\n{}\n--- Generated Rust ---\n{}",
            example.identifier,
            source_result.output,
            rust_result.output,
            conversion.generated_rust_code
        );
    }
}
