use anyhow::Result;
use async_trait::async_trait;
use futures::stream::Stream as FuturesStream;
use llm_sdk::{LanguageModelInput, ModelResponse, PartialModelResponse};
use std::pin::Pin;

pub type StreamResult<'a> =
    Pin<Box<dyn FuturesStream<Item = Result<PartialModelResponse>> + Send + 'a>>;

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    async fn generate(&self, input: LanguageModelInput) -> Result<ModelResponse>;
    async fn stream<'a>(&'a self, input: LanguageModelInput) -> Result<StreamResult<'a>>;
}

pub mod anthropic;
pub mod bigmodel;
pub mod openai;
