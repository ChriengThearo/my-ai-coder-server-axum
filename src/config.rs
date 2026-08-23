use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub llm_api_key: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub host: String,
    pub port: u16,
    pub request_timeout: u64,
    pub max_retries: u32,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let llm_api_key = env::var("LLM_API_KEY")
            .map_err(|_| "LLM_API_KEY environment variable is required".to_string())?;

        let llm_base_url = env::var("LLM_BASE_URL")
            .unwrap_or_else(|_| "https://api.llmapi.ai/v1".to_string());

        let llm_model = env::var("LLM_MODEL")
            .unwrap_or_else(|_| "gpt-4o".to_string());

        let host = env::var("HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse()
            .map_err(|_| "PORT must be a valid number".to_string())?;

        let request_timeout = env::var("REQUEST_TIMEOUT")
            .unwrap_or_else(|_| "1800".to_string())
            .parse()
            .map_err(|_| "REQUEST_TIMEOUT must be a valid number".to_string())?;

        let max_retries = env::var("MAX_RETRIES")
            .unwrap_or_else(|_| "2".to_string())
            .parse()
            .map_err(|_| "MAX_RETRIES must be a valid number".to_string())?;

        Ok(Self {
            llm_api_key,
            llm_base_url,
            llm_model,
            host,
            port,
            request_timeout,
            max_retries,
        })
    }
}
