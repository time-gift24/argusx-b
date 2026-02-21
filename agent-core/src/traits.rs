use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::error::AgentError;
use crate::events::{RunStreamEvent, UiThreadEvent};
use crate::model::{InputEnvelope, ModelOutputEvent, ModelRequest, TurnRequest};
use crate::transcript_item::TranscriptItem;

pub type ModelEventStream =
    Pin<Box<dyn Stream<Item = Result<ModelOutputEvent, AgentError>> + Send>>;
pub type RunEventStream = Pin<Box<dyn Stream<Item = RunStreamEvent> + Send>>;
pub type UiEventStream = Pin<Box<dyn Stream<Item = UiThreadEvent> + Send>>;

pub struct RuntimeStreams {
    pub run: RunEventStream,
    pub ui: UiEventStream,
}

#[async_trait]
pub trait LanguageModel: Send + Sync {
    fn model_name(&self) -> &str;

    async fn stream(&self, request: ModelRequest) -> Result<ModelEventStream, AgentError>;
}

#[async_trait]
pub trait Runtime: Send + Sync {
    async fn run_turn(&self, request: TurnRequest) -> Result<RuntimeStreams, AgentError>;

    async fn inject_input(&self, turn_id: &str, input: InputEnvelope) -> Result<(), AgentError>;

    async fn cancel_turn(&self, turn_id: &str, reason: Option<String>) -> Result<(), AgentError>;
}

#[async_trait]
pub trait CheckpointStore: Send + Sync {
    async fn append_items(&self, turn_id: &str, items: &[TranscriptItem])
        -> Result<(), AgentError>;

    async fn load_items(&self, turn_id: &str) -> Result<Vec<TranscriptItem>, AgentError>;

    async fn snapshot(&self, turn_id: &str, items: &[TranscriptItem]) -> Result<(), AgentError>;
}
