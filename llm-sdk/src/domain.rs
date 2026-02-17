//! Domain models for the LLM SDK.
//!
//! This module contains all core domain types for the agent runtime,
//! including messages, parts, model inputs/outputs, and agent types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Core Domain Types
// ============================================================================

/// Message role in conversation history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    Tool,
}

/// Unified multimodal content unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Part {
    Text(TextPart),
    Image(ImagePart),
    Audio(AudioPart),
    Source(SourcePart),
    ToolCall(ToolCallPart),
    ToolResult(ToolResultPart),
    Reasoning(ReasoningPart),
}

/// Text content part with optional citations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPart {
    pub text: String,
    pub citations: Option<Vec<Citation>>,
}

/// Image content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePart {
    pub mime_type: String,
    pub data_base64: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub id: Option<String>,
}

/// Audio content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioPart {
    pub data_base64: String,
    pub format: AudioFormat,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub transcript: Option<String>,
    pub id: Option<String>,
}

/// Source/reference content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePart {
    pub source: String,
    pub title: String,
    pub content: Vec<Part>,
}

/// Tool call part (requesting a tool execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallPart {
    pub tool_call_id: String,
    pub tool_name: String,
    pub args: JsonValue,
    pub id: Option<String>,
}

/// Tool result part (response from tool execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultPart {
    pub tool_call_id: String,
    pub tool_name: String,
    pub content: Vec<Part>,
    pub is_error: bool,
}

/// Reasoning content part (model's thought process).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningPart {
    pub text: String,
    pub signature: Option<String>,
    pub id: Option<String>,
}

/// Citation reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub source: String,
    pub title: Option<String>,
    pub cited_text: Option<String>,
    pub start_index: u32,
    pub end_index: u32,
}

/// Alias for JSON value.
pub type JsonValue = serde_json::Value;

/// Alias for JSON schema.
pub type JsonSchema = serde_json::Value;

// ============================================================================
// Message Types
// ============================================================================

/// Message-level envelope for parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", content = "content", rename_all = "lowercase")]
pub enum Message {
    User { content: Vec<Part> },
    Assistant { content: Vec<Part> },
    Tool { content: Vec<Part> },
}

impl Message {
    /// Create a user message with text content.
    pub fn user_text(text: impl Into<String>) -> Self {
        Message::User {
            content: vec![Part::Text(TextPart {
                text: text.into(),
                citations: None,
            })],
        }
    }

    /// Create an assistant message with text content.
    pub fn assistant_text(text: impl Into<String>) -> Self {
        Message::Assistant {
            content: vec![Part::Text(TextPart {
                text: text.into(),
                citations: None,
            })],
        }
    }
}

// ============================================================================
// Model Response Types
// ============================================================================

/// Token usage statistics from the model.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Complete model response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelResponse {
    pub content: Vec<Part>,
    pub usage: Option<ModelUsage>,
    pub cost: Option<f64>,
}

/// Partial model response during streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialModelResponse {
    pub delta: Option<ContentDelta>,
    pub usage: Option<ModelUsage>,
    pub cost: Option<f64>,
}

/// Content delta for streaming updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDelta {
    pub index: usize,
    pub part: PartDelta,
}

/// Stream deltas emitted by model part: PartDelta streaming APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PartDelta {
    Text(TextPartDelta),
    ToolCall(ToolCallPartDelta),
    Image(ImagePartDelta),
    Audio(AudioPartDelta),
    Reasoning(ReasoningPartDelta),
}

/// Text part delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPartDelta {
    pub text: String,
}

/// Tool call part delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallPartDelta {
    pub tool_call_id: String,
    pub tool_name: Option<String>,
    pub args_delta: Option<JsonValue>,
}

/// Image part delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePartDelta {
    pub data_base64: Option<String>,
}

/// Audio part delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioPartDelta {
    pub data_base64: Option<String>,
    pub transcript: Option<String>,
}

/// Reasoning part delta.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningPartDelta {
    pub text: String,
}

// ============================================================================
// Language Model Input Types
// ============================================================================

/// Input to a language model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageModelInput {
    pub system_prompt: Option<String>,
    pub messages: Vec<Message>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
    pub response_format: Option<ResponseFormat>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub seed: Option<u64>,
    pub modalities: Option<Vec<Modality>>,
    pub audio: Option<AudioOptions>,
    pub reasoning: Option<ReasoningOptions>,
    pub metadata: Option<HashMap<String, String>>,
}

impl LanguageModelInput {
    /// Create a new language model input with messages.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            system_prompt: None,
            messages,
            tools: None,
            tool_choice: None,
            response_format: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            presence_penalty: None,
            frequency_penalty: None,
            seed: None,
            modalities: None,
            audio: None,
            reasoning: None,
            metadata: None,
        }
    }

    /// Set the system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set tools available to the model.
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set the temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

/// Tool definition for function calling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: JsonSchema,
}

/// How model should select tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "tool_name", rename_all = "camelCase")]
pub enum ToolChoice {
    Auto,
    None,
    Required,
    Tool { tool_name: String },
}

/// Response shape constraints requested from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "schema", rename_all = "camelCase")]
pub enum ResponseFormat {
    Text,
    Json {
        name: String,
        description: Option<String>,
        schema: Option<JsonSchema>,
    },
}

/// Requested output channels for the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Text,
    Image,
    Audio,
}

/// Audio options for input/output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOptions {
    pub format: Option<AudioFormat>,
    pub voice: Option<String>,
    pub language: Option<String>,
}

/// Audio format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Wav,
    Mp3,
    Linear16,
    Flac,
    Mulaw,
    Alaw,
    Aac,
    Opus,
}

/// Reasoning options for models that support it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningOptions {
    pub enabled: bool,
    pub budget_tokens: Option<u32>,
}

// ============================================================================
// Model Metadata Types
// ============================================================================

/// Metadata about a language model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageModelMetadata {
    pub pricing: Option<LanguageModelPricing>,
    pub capabilities: Option<Vec<LanguageModelCapability>>,
}

/// Pricing information for a language model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageModelPricing {
    pub input_cost_per_text_token: Option<f64>,
    pub input_cost_per_cached_text_token: Option<f64>,
    pub output_cost_per_text_token: Option<f64>,
    pub input_cost_per_audio_token: Option<f64>,
    pub input_cost_per_cached_audio_token: Option<f64>,
    pub output_cost_per_audio_token: Option<f64>,
    pub input_cost_per_image_token: Option<f64>,
    pub input_cost_per_cached_image_token: Option<f64>,
    pub output_cost_per_image_token: Option<f64>,
}

/// Capabilities of a language model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LanguageModelCapability {
    TextInput,
    TextOutput,
    ImageInput,
    ImageOutput,
    AudioInput,
    AudioOutput,
    FunctionCalling,
    StructuredOutput,
    Citation,
    Reasoning,
}

// ============================================================================
// Agent Runtime Types
// ============================================================================

/// Agent runtime event log item.
/// This is the only persisted replay unit required for resumability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AgentItem {
    Message(Message),
    Model(ModelResponse),
    Tool(AgentItemTool),
}

/// Tool execution item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentItemTool {
    pub tool_call_id: String,
    pub tool_name: String,
    pub input: JsonValue,
    pub output: Vec<Part>,
    pub is_error: bool,
}

/// Complete agent response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResponse {
    pub output: Vec<AgentItem>,
    pub content: Vec<Part>,
}

/// Streaming event surface from run_stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AgentStreamEvent {
    Partial(PartialModelResponse),
    Item { index: usize, item: AgentItem },
    Response(AgentResponse),
}

// ============================================================================
// Memory Types
// ============================================================================

/// Durable memory unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryBlock {
    pub id: String,
    pub content: String,
}

/// Search hit for archival memory retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySearchHit {
    pub block: MemoryBlock,
    pub score: Option<f32>,
}

// ============================================================================
// Plan Types
// ============================================================================

/// One step in planner-executor plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStep {
    pub status: PlanStepStatus,
    pub step: String,
}

/// Status of a plan step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Complete,
}

/// Plan snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSnapshot {
    pub explanation: Option<String>,
    pub steps: Vec<PlanStep>,
}

// ============================================================================
// Approval Types
// ============================================================================

/// Approval action to be approved.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalAction {
    pub action_type: String,
    pub resource: String,
    pub metadata: Option<JsonValue>,
}

/// Human decision written back by HITL flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApprovalDecision {
    Approved,
    Denied,
    NeedsMoreInfo,
}

// ============================================================================
// Checkpoint Types
// ============================================================================

/// Serialized replay checkpoint after interruption.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunCheckpoint {
    pub items: Vec<AgentItem>,
    pub context_fingerprint: String,
    pub interrupted_at: u64,
}
