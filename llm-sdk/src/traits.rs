use crate::domain::*;
use crate::error::*;
use async_trait::async_trait;
use std::collections::HashSet;

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

    /// Stream responses from the model.
    async fn stream(
        &self,
        input: LanguageModelInput,
    ) -> std::result::Result<
        Box<
            dyn futures::Stream<Item = std::result::Result<PartialModelResponse, ModelError>>
                + Send
                + Unpin
                + '_,
        >,
        ModelError,
    >;
}

// ============================================================================
// Supporting Types for RunSessionTrait
// ============================================================================

/// Parameters for initializing an agent session.
pub struct AgentParams<Ctx> {
    pub system_prompt: Option<String>,
    pub tools: Vec<Box<dyn AgentToolTrait<Ctx>>>,
    pub max_turns: Option<u32>,
    pub context: Ctx,
}

/// Minimal stream trait for streaming model responses.
pub trait Stream: Send + Unpin {
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>>
    where
        Self: Unpin;

    type Item;
}

/// Trait for streaming agent events.
pub trait AgentEventStream: Send + Unpin {
    fn poll_next(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<std::result::Result<AgentStreamEvent, SessionError>>>;
}

// ============================================================================
// Trait 2: RunSessionTrait
// ============================================================================

/// Trait for agent run session management.
#[async_trait]
pub trait RunSessionTrait<Ctx>: Send + Sync {
    /// Initialize a new session with parameters.
    async fn init(
        params: AgentParams<Ctx>,
        context: Ctx,
    ) -> std::result::Result<Box<dyn RunSessionTrait<Ctx> + Send + Sync>, SessionError>
    where
        Self: Sized;

    /// Run the agent with input items.
    async fn run(&self, input: Vec<AgentItem>) -> std::result::Result<AgentResponse, SessionError>;

    /// Run the agent in streaming mode.
    fn run_stream(
        &self,
        input: Vec<AgentItem>,
    ) -> std::result::Result<Box<dyn AgentEventStream + Send + Unpin>, SessionError>;

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

// ============================================================================
// Trait 5: AgentToolTrait
// ============================================================================

/// Trait for agent tools that can be executed during runs.
#[async_trait]
pub trait AgentToolTrait<Ctx>: Send + Sync {
    /// Returns the tool name.
    fn name(&self) -> String;

    /// Returns the tool description.
    fn description(&self) -> String;

    /// Returns the JSON schema for tool parameters.
    fn parameters_schema(&self) -> serde_json::Value;

    /// Execute the tool with given arguments.
    async fn execute(
        &self,
        args: serde_json::Value,
        context: &Ctx,
        state: &dyn RunStateView,
    ) -> std::result::Result<Vec<Part>, ToolExecutionError>;
}

// ============================================================================
// Trait 6: DelegationToolTrait
// ============================================================================

/// Trait for tools that delegate to other agents.
#[async_trait]
pub trait DelegationToolTrait<Ctx>: Send + Sync {
    /// Returns the target agent name.
    fn target_agent_name(&self) -> &str;

    /// Rewrite the task for the target agent.
    fn rewrite_task(
        &self,
        task: String,
        context: &Ctx,
    ) -> std::result::Result<Message, ToolExecutionError>;

    /// Execute delegation to the target agent.
    async fn delegate(
        &self,
        task: String,
        context: &Ctx,
    ) -> std::result::Result<Vec<Part>, ToolExecutionError>;
}

// ============================================================================
// Trait 7: CoreMemoryStoreTrait
// ============================================================================

/// Trait for core (working) memory storage.
pub trait CoreMemoryStoreTrait: Send + Sync {
    /// List all memory blocks in core memory.
    fn list_core(&self) -> std::result::Result<Vec<MemoryBlock>, MemoryError>;

    /// Upsert (insert or update) a memory block.
    fn upsert_core(&self, block: MemoryBlock)
        -> std::result::Result<Vec<MemoryBlock>, MemoryError>;

    /// Delete a memory block by ID.
    fn delete_core(&self, id: &str) -> std::result::Result<Vec<MemoryBlock>, MemoryError>;

    /// Render core memory for inclusion in system prompt.
    fn render_for_system_prompt(
        &self,
        token_budget: usize,
    ) -> std::result::Result<String, MemoryError>;
}

// ============================================================================
// Trait 8: ArchivalMemoryStoreTrait
// ============================================================================

/// Trait for archival (long-term) memory storage.
pub trait ArchivalMemoryStoreTrait: Send + Sync {
    /// Insert or update an archival memory block.
    fn upsert_archival(&self, block: MemoryBlock) -> std::result::Result<(), MemoryError>;

    /// Delete an archival memory block by ID.
    fn delete_archival(&self, id: &str) -> std::result::Result<(), MemoryError>;

    /// Search archival memory for relevant blocks.
    fn search_archival(
        &self,
        query: &str,
        top_k: usize,
    ) -> std::result::Result<Vec<MemorySearchHit>, MemoryError>;
}

// ============================================================================
// Trait 9: PlanStoreTrait
// ============================================================================

/// Trait for plan storage and management.
pub trait PlanStoreTrait: Send + Sync {
    /// Get a plan snapshot by run ID.
    fn get_plan(&self, run_id: &str) -> std::result::Result<Option<PlanSnapshot>, PlanError>;

    /// Replace the plan for a run.
    fn replace_plan(&self, run_id: &str, next: PlanSnapshot) -> std::result::Result<(), PlanError>;

    /// Validate a plan snapshot.
    fn validate_plan(&self, plan: &PlanSnapshot) -> std::result::Result<(), PlanValidationError>;

    /// Check if a plan is complete for a run.
    fn is_complete(&self, run_id: &str) -> std::result::Result<bool, PlanError>;
}

// ============================================================================
// Trait 10: ApprovalGateTrait
// ============================================================================

/// Trait for approval gates (human-in-the-loop).
#[async_trait]
pub trait ApprovalGateTrait<Ctx>: Send + Sync {
    /// Generate approval key for an action.
    fn approval_key(&self, action: &ApprovalAction) -> String;

    /// Check if an action requires approval.
    fn requires_approval(&self, action: &ApprovalAction, context: &Ctx) -> bool;

    /// Get the current approval decision for a key.
    async fn current_decision(
        &self,
        key: &str,
        context: &Ctx,
    ) -> std::result::Result<Option<ApprovalDecision>, ApprovalError>;

    /// Create an error for when approval is required but not granted.
    fn approval_required_error(&self, action: &ApprovalAction) -> ApprovalError;
}

// ============================================================================
// Trait 11: InterruptionResumeTrait
// ============================================================================

/// Trait for handling interruptions and resumability.
pub trait InterruptionResumeTrait<Ctx>: Send + Sync {
    /// Capture a checkpoint from current state.
    fn capture_checkpoint(
        &self,
        items: &[AgentItem],
        context: &Ctx,
    ) -> std::result::Result<RunCheckpoint, ResumeError>;

    /// Restore input items from a checkpoint.
    fn restore_input(
        &self,
        checkpoint: &RunCheckpoint,
    ) -> std::result::Result<Vec<AgentItem>, ResumeError>;

    /// Restore context from a checkpoint.
    fn restore_context(
        &self,
        checkpoint: &RunCheckpoint,
        updated: Ctx,
    ) -> std::result::Result<Ctx, ResumeError>;

    /// Check if a tool call should be skipped on resume.
    fn should_skip_tool_call(&self, tool_call_id: &str, items: &[AgentItem]) -> bool;
}
