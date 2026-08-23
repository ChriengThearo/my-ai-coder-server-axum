use crate::{
    client::LlmClient,
    error::AppError,
    models::{ChatRequest, ChatResponse, HealthResponse, LlmRequest, Message},
};
use axum::{extract::State, Json};
use std::time::Instant;
use tracing::info;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub llm_client: LlmClient,
    pub default_model: String,
}

/// Root endpoint - returns API status
pub async fn root(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "online".to_string(),
        message: Some("My AI Coder API is running".to_string()),
        model: state.default_model,
    })
}

/// Health check endpoint
pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: None,
        model: state.default_model,
    })
}

/// Chat endpoint - processes chat requests and forwards to LLM API
pub async fn chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let request_started = Instant::now();

    info!("========== CHAT REQUEST ==========");

    // Validate and extract messages
    let messages = if let Some(messages) = request.messages {
        if !messages.is_empty() {
            messages
        } else {
            return Err(AppError::BadRequest(
                "'messages' array cannot be empty".to_string(),
            ));
        }
    } else if let Some(message_text) = request.message {
        vec![Message {
            role: "user".to_string(),
            content: Some(message_text),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }]
    } else {
        return Err(AppError::BadRequest(
            "Request must contain 'message' or 'messages'".to_string(),
        ));
    };

    let model = request.model.unwrap_or_else(|| state.default_model.clone());

    info!("Model: {}", model);
    info!("Messages: {}", messages.len());
    info!("Tools: {}", request.tools.as_ref().map(|t| t.len()).unwrap_or(0));

    // Build LLM request
    let llm_request = LlmRequest {
        model,
        messages,
        tools: request.tools,
        tool_choice: request.tool_choice,
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        max_completion_tokens: request.max_completion_tokens,
        top_p: request.top_p,
        frequency_penalty: request.frequency_penalty,
        presence_penalty: request.presence_penalty,
        stop: request.stop,
    };

    info!("==================================");

    // Call LLM API
    let assistant_message = state.llm_client.chat_completion(llm_request).await?;

    let total_elapsed = request_started.elapsed();
    info!("Total request time: {:.2}s", total_elapsed.as_secs_f64());
    info!("");

    Ok(Json(ChatResponse {
        message: assistant_message,
    }))
}
