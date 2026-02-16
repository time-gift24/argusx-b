use futures::Stream;
use reqwest::Client;
use std::pin::Pin;

use crate::config::Config;
use crate::error::{BigModelError, Result};
use crate::models::{ChatRequest, ChatResponse, ChatResponseChunk};

pub struct BigModelClient {
    config: Config,
    http: Client,
}

impl BigModelClient {
    pub fn new(config: Config) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");

        Self { config, http }
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/v4/chat/completions", self.config.base_url);

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let chat_response: ChatResponse = response.json().await?;
            Ok(chat_response)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(Self::handle_error(status.as_u16(), error_text))
        }
    }

    pub fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatResponseChunk>> + Send>> {
        // TODO: Implement streaming
        Box::pin(futures::stream::empty())
    }

    fn handle_error(status: u16, body: String) -> BigModelError {
        match status {
            400 => BigModelError::InvalidRequest(body),
            401 | 403 => BigModelError::AuthenticationError(body),
            429 => BigModelError::RateLimitError(body),
            500..=599 => BigModelError::ServerError(body),
            _ => BigModelError::ServerError(format!("Unknown error: {} - {}", status, body)),
        }
    }
}
