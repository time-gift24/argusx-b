//! Core traits for the LLM SDK.
//!
//! This module contains the essential traits for the agent runtime:
//! - LanguageModelTrait: For LLM provider implementations
//! - RunSessionTrait: For agent session management
//! - RunStateTrait: For agent state management
//!
//! Additional traits (like memory, plans, approval) are available in the
//! tools module for the Codex-inspired tool system.

use crate::domain::*;
use crate::error::*;
use async_trait::async_trait;
use std::collections::HashSet;

// ============================================================================
// Tools module - re-exports
// ============================================================================

pub use crate::tools::{
    ToolHandler, ToolKind, ToolInvocation, ToolPayload, ToolOutput,
    ToolRegistry, ToolRegistryBuilder, ToolOrchestrator,
    ApprovalGate, ApprovalResult, ApprovalRequirement,
    OutputBody, McpToolResult, McpContent,
};

// ============================================================================
// Trait 1: LanguageModelTrait
// ============================================================================

/// Trait for language model implementations.
#[async_trait]
pub trait LanguageModelTrait: Send + Sync {
    /// Returns the provider name.
    fn provider(&self) -> &'static str;

    /// Returns the model ID.
    fn model_id(&self) -> String;

    /// Returns model metadata.
    fn metadata(&self) -> Option<LanguageModelMetadata>;

    /// Generate a complete response from the model.
    async fn generate(
        &self,
        input: LanguageModelInput,
    ) -> std::result::Result<ModelResponse, ModelError>;

    /// Stream responses from the model, returning a receiver for model events.
    async fn stream_events(
        &self,
        input: LanguageModelInput,
    ) -> std::result::Result<tokio::sync::mpsc::Receiver<ModelStreamEvent>, ModelError>;
}

// ============================================================================
// Supporting Types for RunSessionTrait
// ============================================================================

/// Parameters for initializing an agent session.
#[derive(Default)]
pub struct AgentParams {
    pub system_prompt: Option<String>,
    /// Tools registered in the session
    pub tools: Vec<std::sync::Arc<dyn ToolHandler>>,
    pub max_turns: Option<u32>,
}

// ============================================================================
// Trait 2: RunSessionTrait
// ============================================================================

/// Trait for agent run session management.
#[async_trait]
pub trait RunSessionTrait: Send + Sync {
    /// Run the agent with input items.
    async fn run(&self, input: Vec<AgentItem>) -> std::result::Result<AgentResponse, SessionError>;

    /// Run the agent in streaming mode, returning a receiver for run events.
    async fn run_events(
        &self,
        input: Vec<AgentItem>,
    ) -> std::result::Result<tokio::sync::mpsc::Receiver<RunStreamEvent>, SessionError>;

    /// Close the session and clean up resources.
    async fn close(self: Box<Self>) -> std::result::Result<(), SessionError>;
}

// ============================================================================
// Trait 3: RunStateView
// ============================================================================

/// Read-only view of agent run state.
pub trait RunStateView: Send + Sync {
    /// Get all items in the run state.
    fn items(&self) -> Vec<AgentItem>;

    /// Get the set of processed tool call IDs.
    fn processed_tool_call_ids(&self) -> HashSet<String>;
}

// ============================================================================
// Trait 4: RunStateTrait
// ============================================================================

/// Mutable trait for agent run state management.
pub trait RunStateTrait: RunStateView {
    /// Append an item to the run state.
    fn append_item(&mut self, item: AgentItem) -> usize;

    /// Append a model response to the run state.
    fn append_model_response(&mut self, model: ModelResponse) -> usize;

    /// Begin a new turn in the run state.
    fn begin_next_turn(&mut self) -> std::result::Result<(), SessionError>;

    /// Build messages for the current turn.
    fn build_turn_messages(&self) -> Vec<Message>;

    /// Finalize the run and produce the agent response.
    fn finalize(&self, final_content: Vec<Part>) -> AgentResponse;
}
