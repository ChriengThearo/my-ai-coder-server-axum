mod client;
mod config;
mod error;
mod handlers;
mod models;

use crate::{
    client::LlmClient,
    config::Config,
    handlers::{chat, get_balance, health, root, validate_api_key, AppState},
};
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "my_ai_coder_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;

    info!("==============================================");
    info!("       My AI Coder API (Rust + Axum)");
    info!("==============================================");
    info!("Model: {}", config.llm_model);
    info!("LLM Base URL: {}", config.llm_base_url);
    info!("Request timeout: {}s", config.request_timeout);
    info!("Max retries: {}", config.max_retries);
    info!("Server: http://{}:{}", config.host, config.port);
    info!("==============================================");
    info!("");

    // Create LLM client
    let llm_client = LlmClient::new(config.clone())?;

    // Create application state
    let state = AppState {
        llm_client,
        default_model: config.llm_model.clone(),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/chat", post(chat))
        .route("/api/auth/validate", post(validate_api_key))
        .route("/api/auth/balance", post(get_balance))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    // Start server
    // Parse host string to SocketAddr compatible format
    let host_octets: [u8; 4] = if config.host == "0.0.0.0" {
        [0, 0, 0, 0]
    } else {
        [127, 0, 0, 1]
    };
    
    let addr = SocketAddr::from((host_octets, config.port));
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
