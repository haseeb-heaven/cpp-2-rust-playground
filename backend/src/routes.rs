use crate::config::ApplicationConfiguration;
use crate::examples_catalog::get_premade_examples;
use crate::executor::execute_with_jdoodle;
use crate::models::{CodeExample, ConversionRequest, ConversionResponse, ExecutionRequest, ExecutionResponse};
use crate::transpiler::convert_source_to_rust;
use axum::extract::State;
use axum::http::Method;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct ApplicationState {
    pub configuration: ApplicationConfiguration,
}

pub fn create_application_router(configuration: ApplicationConfiguration) -> Router {
    let shared_state = Arc::new(ApplicationState { configuration });
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_check_handler))
        .route("/api/examples", get(examples_list_handler))
        .route("/api/convert", post(convert_code_handler))
        .route("/api/execute", post(execute_code_handler))
        .with_state(shared_state)
        .layer(cors_layer)
}

async fn health_check_handler() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "RustC-Playground Backend"
    }))
}

async fn examples_list_handler() -> Json<Vec<CodeExample>> {
    Json(get_premade_examples())
}

async fn convert_code_handler(
    Json(payload): Json<ConversionRequest>,
) -> Json<ConversionResponse> {
    let response = convert_source_to_rust(&payload.source_code, &payload.source_language);
    Json(response)
}

async fn execute_code_handler(
    State(state): State<Arc<ApplicationState>>,
    Json(payload): Json<ExecutionRequest>,
) -> Json<ExecutionResponse> {
    let response = execute_with_jdoodle(&state.configuration, &payload).await;
    Json(response)
}
