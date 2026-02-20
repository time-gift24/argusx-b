//! Tool execution context types
//!
//! Defines the core types for tool invocation:
//! - ToolInvocation: Complete context for tool execution
//! - ToolPayload: Input parameters to tools
//! - ToolOutput: Output result from tools

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session information passed to tools
#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub session_id: String,
    pub cwd: std::path::PathBuf,
    /// Environment variables
    pub env: HashMap<String, String>,
}

impl SessionInfo {
    pub fn new(session_id: String, cwd: std::path::PathBuf) -> Self {
        Self {
            session_id,
            cwd,
            env: std::env::vars().collect(),
        }
    }
}

/// Turn context - information about the current conversation turn
#[derive(Clone, Debug)]
pub struct TurnContext {
    /// Current working directory
    pub cwd: std::path::PathBuf,
    /// Turn number
    pub turn_number: u32,
    /// Messages in current turn (serialized as JSON value for simplicity)
    pub messages: Vec<serde_json::Value>,
}

/// Change tracker for monitoring file modifications
#[derive(Clone, Debug, Default)]
pub struct TurnDiffTracker {
    /// Files modified in current turn
    pub modified_files: Vec<std::path::PathBuf>,
    /// Files created in current turn
    pub created_files: Vec<std::path::PathBuf>,
    /// Files deleted in current turn
    pub deleted_files: Vec<std::path::PathBuf>,
}

impl TurnDiffTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_modified(&mut self, path: std::path::PathBuf) {
        self.modified_files.push(path);
    }

    pub fn add_created(&mut self, path: std::path::PathBuf) {
        self.created_files.push(path);
    }

    pub fn add_deleted(&mut self, path: std::path::PathBuf) {
        self.deleted_files.push(path);
    }
}

/// Tool invocation context - everything needed to execute a tool
#[derive(Clone, Debug)]
pub struct ToolInvocation {
    /// Session information
    pub session: SessionInfo,
    /// Current turn context
    pub turn: TurnContext,
    /// Change tracker
    pub tracker: TurnDiffTracker,
    /// Unique call ID for this invocation
    pub call_id: String,
    /// Name of the tool being invoked
    pub tool_name: String,
    /// Input payload
    pub payload: ToolPayload,
}

impl ToolInvocation {
    pub fn new(
        session: SessionInfo,
        turn: TurnContext,
        call_id: String,
        tool_name: String,
        payload: ToolPayload,
    ) -> Self {
        Self {
            session,
            turn,
            tracker: TurnDiffTracker::new(),
            call_id,
            tool_name,
            payload,
        }
    }
}

/// Input payload types for tools
#[derive(Clone, Debug)]
pub enum ToolPayload {
    /// Function-style tool with JSON arguments
    Function {
        /// JSON string of arguments
        arguments: String,
    },
    /// Custom/freeform tool input
    Custom {
        /// Raw input string
        input: String,
    },
    /// Local shell command
    LocalShell {
        /// Command and arguments
        command: Vec<String>,
    },
    /// MCP server tool
    Mcp {
        /// MCP server name
        server: String,
        /// Tool name on MCP server
        tool: String,
        /// Raw JSON arguments
        raw_arguments: String,
    },
}

impl ToolPayload {
    /// Extract the input as a string for logging/debugging
    pub fn as_str(&self) -> String {
        match self {
            ToolPayload::Function { arguments } => arguments.clone(),
            ToolPayload::Custom { input } => input.clone(),
            ToolPayload::LocalShell { command } => command.join(" "),
            ToolPayload::Mcp { raw_arguments, .. } => raw_arguments.clone(),
        }
    }
}

/// Output types from tool execution
#[derive(Clone, Debug)]
pub enum ToolOutput {
    /// Function-style tool output
    Function {
        /// Output body (text or structured)
        body: OutputBody,
        /// Success indicator
        success: Option<bool>,
    },
    /// MCP tool output
    Mcp {
        /// MCP tool result
        result: Result<McpToolResult, String>,
    },
}

/// Output body for function tools
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutputBody {
    /// Plain text output
    Text(String),
    /// Structured content (will be serialized as-is)
    Structured(serde_json::Value),
}

impl OutputBody {
    pub fn text<S: Into<String>>(s: S) -> Self {
        OutputBody::Text(s.into())
    }

    pub fn structured<V: Into<serde_json::Value>>(v: V) -> Self {
        OutputBody::Structured(v.into())
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            OutputBody::Text(s) => Some(s),
            OutputBody::Structured(v) => v.as_str(),
        }
    }
}

impl From<String> for OutputBody {
    fn from(s: String) -> Self {
        OutputBody::Text(s)
    }
}

impl From<&str> for OutputBody {
    fn from(s: &str) -> Self {
        OutputBody::Text(s.to_string())
    }
}

/// MCP tool call result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpToolResult {
    /// Content returned by the tool
    pub content: Vec<McpContent>,
    /// Whether the tool call was successful
    pub is_error: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpContent {
    /// Text content
    Text { text: String },
    /// Image content (base64)
    Image { data: String, mime_type: String },
    /// Resource content
    Resource { uri: String, mime_type: String, text: Option<String> },
}

// ============================================================================
// Arguments parsing helper
// ============================================================================

/// Parse JSON arguments into a typed struct
pub fn parse_arguments<T>(arguments: &str) -> Result<T, super::error::ToolExecutionError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(arguments).map_err(|e| {
        super::error::ToolExecutionError::InvalidArguments(format!(
            "failed to parse function arguments: {}",
            e
        ))
    })
}
