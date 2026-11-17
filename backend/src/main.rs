use rustc_playground_backend::config::ApplicationConfiguration;
use rustc_playground_backend::routes::create_application_router;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let configuration = ApplicationConfiguration::load_from_environment();
    let port = configuration.backend_port;
    let router = create_application_router(configuration);

    let address = SocketAddr::from(([127, 0, 0, 1], port));
    info!("C/C++ to Rust Playground backend listening on http://{}", address);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Failed to bind TCP listener");
    axum::serve(listener, router)
        .await
        .expect("Failed to start Axum server");
}
