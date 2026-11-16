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
    assert_eq!(parsed["service"], "RustC-Playground Backend");
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
    assert!(rust_code.contains("fn main() {"));
    assert!(rust_code.contains("println!(\"Hi\");"));
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
}
