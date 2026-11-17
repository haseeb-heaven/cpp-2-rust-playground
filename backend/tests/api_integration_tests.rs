use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use rustc_playground_backend::config::ApplicationConfiguration;
use rustc_playground_backend::routes::create_application_router;
use serde_json::{json, Value};
use tower::ServiceExt;

fn test_configuration() -> ApplicationConfiguration {
    ApplicationConfiguration {
        jdooodle_client_id: String::new(),
        jdooodle_client_secret: String::new(),
        jdooodle_api_url: "https://api.jdoodle.com/v1/execute".to_string(),
        backend_port: 8080,
        request_timeout_seconds: 5,
    }
}

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let app = create_application_router(test_configuration());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(parsed["status"], "ok");
    assert_eq!(parsed["service"], "C/C++ to Rust Playground Backend");
}

#[tokio::test]
async fn examples_endpoint_returns_premade_examples() {
    let app = create_application_router(test_configuration());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/examples")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    let examples = parsed.as_array().unwrap();
    assert!(examples.len() >= 5);
}

#[tokio::test]
async fn convert_endpoint_translates_cpp_to_rust() {
    let app = create_application_router(test_configuration());
    let payload = json!({
        "sourceCode": "#include <iostream>\nint main() {\n    std::cout << \"Hi\" << std::endl;\n    return 0;\n}",
        "sourceLanguage": "cpp"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    let rust_code = parsed["generatedRustCode"].as_str().unwrap();
    assert!(!rust_code.trim().is_empty());
    if parsed["usedFallback"] == Value::Bool(true) {
        assert!(rust_code.contains("fn main() {"));
        assert!(rust_code.contains("println!(\"Hi\");"));
        assert!(parsed["transpilerEngine"]
            .as_str()
            .unwrap()
            .contains("Built-in Regex C++ Fallback"));
    } else {
        assert!(parsed["transpilerEngine"]
            .as_str()
            .unwrap()
            .contains("cpp2rust CLI"));
        assert_eq!(parsed["postProcessing"]["compilerVerified"], Value::Bool(true));
    }
}

#[tokio::test]
async fn compilers_endpoint_returns_availability() {
    let app = create_application_router(test_configuration());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/compilers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(parsed.get("cAvailable").is_some());
    assert!(parsed.get("cppAvailable").is_some());
    assert!(parsed.get("rustAvailable").is_some());
}

#[tokio::test]
async fn execute_endpoint_reports_missing_credentials_cleanly() {
    let app = create_application_router(test_configuration());
    let payload = json!({
        "code": "fn main() { println!(\"ok\"); }",
        "language": "rust",
        "mode": "run",
        "stdinInput": "",
        "provider": "online"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/execute")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(parsed["success"], false);
    assert!(parsed["errorOutput"]
        .as_str()
        .unwrap()
        .contains("credentials"));
    assert!(parsed["usedProvider"]
        .as_str()
        .unwrap()
        .starts_with("online"));
}

#[tokio::test]
async fn execute_endpoint_uses_local_compiler_when_local_provider_selected() {
    if rustc_playground_backend::executor::find_local_compiler(
        &rustc_playground_backend::models::ExecutionLanguage::Rust,
    )
    .is_none()
    {
        return;
    }

    let app = create_application_router(test_configuration());
    let payload = json!({
        "code": "fn main() { println!(\"strict-local\"); }",
        "language": "rust",
        "mode": "run",
        "stdinInput": "",
        "provider": "local"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/execute")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["output"], "strict-local\n");
    assert!(parsed["usedProvider"]
        .as_str()
        .unwrap()
        .starts_with("local ("));
}

#[tokio::test]
async fn convert_endpoint_respects_faithful_and_idiomatic_output_style_modes() {
    let source = "#include <stdlib.h>\n#include <stdio.h>\n\nint main() {\n    int *p = malloc(sizeof(int));\n    if (p != NULL) {\n        *p = 100;\n        printf(\"VALUE of p is %d\\n\", *p);\n        free(p);\n    }\n    return 0;\n}";

    let app_faithful = create_application_router(test_configuration());
    let faithful_res = app_faithful
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "sourceCode": source,
                        "sourceLanguage": "c",
                        "outputStyle": "faithful"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(faithful_res.status(), StatusCode::OK);
    let faithful_body = faithful_res.into_body().collect().await.unwrap().to_bytes();
    let faithful_json: Value = serde_json::from_slice(&faithful_body).unwrap();
    assert_eq!(faithful_json["outputStyle"], "faithful");
    let faithful_code = faithful_json["generatedRustCode"].as_str().unwrap();
    assert_eq!(faithful_code, faithful_json["faithfulRustCode"].as_str().unwrap());
    assert!(!faithful_code.trim().is_empty());

    let app_idiomatic = create_application_router(test_configuration());
    let idiomatic_res = app_idiomatic
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/convert")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "sourceCode": source,
                        "sourceLanguage": "c",
                        "outputStyle": "idiomatic"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(idiomatic_res.status(), StatusCode::OK);
    let idiomatic_body = idiomatic_res
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let idiomatic_json: Value = serde_json::from_slice(&idiomatic_body).unwrap();
    assert_eq!(idiomatic_json["outputStyle"], "idiomatic");
    let idiomatic_code = idiomatic_json["generatedRustCode"].as_str().unwrap();
    assert_eq!(idiomatic_code, idiomatic_json["idiomaticRustCode"].as_str().unwrap());
    assert!(!idiomatic_code.trim().is_empty());
    assert_eq!(
        faithful_json["faithfulRustCode"],
        idiomatic_json["faithfulRustCode"],
        "both mode requests must retain the same faithful engine output"
    );
    assert_eq!(
        faithful_json["idiomaticRustCode"],
        idiomatic_json["idiomaticRustCode"],
        "both mode requests must retain the same idiomatic view"
    );
}
