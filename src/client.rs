use crate::{
    config::Config,
    error::AppError,
    models::{LlmRequest, LlmResponse, Message},
};
use reqwest::Client;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// LLM API client
#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    config: Config,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(config: Config) -> Result<Self, AppError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout))
            .build()
            .map_err(|e| AppError::InternalError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Send a chat completion request to the LLM API
    pub async fn chat_completion(&self, request: LlmRequest) -> Result<Message, AppError> {
        let url = format!("{}/chat/completions", self.config.llm_base_url);
        
        info!("Calling LLM API...");
        info!("Model: {}", request.model);
        info!("Messages: {}", request.messages.len());
        info!("Tools: {}", request.tools.as_ref().map(|t| t.len()).unwrap_or(0));
        
        let last_role = request.messages.last().map(|m| m.role.as_str()).unwrap_or("none");
        info!("Last message role: {}", last_role);

        let start = Instant::now();

        let mut retries = 0;
        let max_retries = self.config.max_retries;

        loop {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.config.llm_api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    
                    if status.is_success() {
                        let llm_response: LlmResponse = resp.json().await.map_err(|e| {
                            AppError::LlmApiError(format!("Failed to parse LLM response: {}", e))
                        })?;

                        let elapsed = start.elapsed();
                        info!("LLM API response received");
                        info!("Response time: {:.2}s", elapsed.as_secs_f64());

                        let message = llm_response
                            .choices
                            .into_iter()
                            .next()
                            .ok_or_else(|| AppError::LlmApiError("No choices in response".to_string()))?
                            .message;

                        info!("Response role: {}", message.role);
                        info!("Tool calls: {}", message.tool_calls.as_ref().map(|tc| tc.len()).unwrap_or(0));

                        return Ok(message);
                    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        return Err(AppError::RateLimitError(format!("LLM API rate limit: {}", error_text)));
                    } else {
                        let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        
                        if retries < max_retries {
                            retries += 1;
                            warn!("LLM API error (status {}), retrying ({}/{})...", status, retries, max_retries);
                            tokio::time::sleep(Duration::from_millis(500 * retries as u64)).await;
                            continue;
                        }
                        
                        return Err(AppError::LlmApiError(format!(
                            "LLM API error {}: {}",
                            status, error_text
                        )));
                    }
                }
                Err(e) => {
                    if retries < max_retries && !e.is_timeout() {
                        retries += 1;
                        warn!("Request error, retrying ({}/{})...", retries, max_retries);
                        tokio::time::sleep(Duration::from_millis(500 * retries as u64)).await;
                        continue;
                    }
                    return Err(e.into());
                }
            }
        }
    }
}
