use std::env;

#[derive(Debug, Clone)]
pub struct ApplicationConfiguration {
    pub jdooodle_client_id: String,
    pub jdooodle_client_secret: String,
    pub jdooodle_api_url: String,
    pub backend_port: u16,
    pub request_timeout_seconds: u64,
}

impl ApplicationConfiguration {
    pub fn load_from_environment() -> Self {
        let _ = dotenvy::dotenv();
        let _ = dotenvy::from_path("../.env");

        let jdooodle_client_id = env::var("JDOODLE_CLIENT_ID").unwrap_or_default();
        let jdooodle_client_secret = env::var("JDOODLE_CLIENT_SECRET").unwrap_or_default();
        let jdooodle_api_url = env::var("JDOODLE_API_URL")
            .unwrap_or_else(|_| "https://api.jdoodle.com/v1/execute".to_string());
        let backend_port = env::var("BACKEND_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(8080);
        let request_timeout_seconds = env::var("REQUEST_TIMEOUT_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(15);

        Self {
            jdooodle_client_id,
            jdooodle_client_secret,
            jdooodle_api_url,
            backend_port,
            request_timeout_seconds,
        }
    }
}
