use anyhow::Result;
use async_trait::async_trait;
use llm_sdk::{LanguageModelInput, ModelResponse, ModelStreamEvent};
use tokio::sync::mpsc;

pub type StreamResult = mpsc::Receiver<ModelStreamEvent>;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate(&self, input: LanguageModelInput) -> Result<ModelResponse>;
    async fn stream_events(&self, input: LanguageModelInput) -> Result<StreamResult>;
}

pub mod anthropic;
pub mod bigmodel;
pub mod openai;
