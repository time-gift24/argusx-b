use anyhow::Result;
use async_trait::async_trait;
use llm_sdk::{LanguageModelInput, ModelResponse};
use std::pin::Pin;
use futures::stream::Stream;

pub type StreamResult = Pin<Box<dyn Stream<Item = Result<ModelResponse>> + Send>>;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate(&self, input: LanguageModelInput) -> Result<ModelResponse>;
    async fn stream(&self, input: LanguageModelInput) -> Result<StreamResult>;
}

pub mod bigmodel;
pub mod openai;
pub mod anthropic;
