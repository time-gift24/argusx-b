//! Tool handler trait and related types
//!
//! This module defines the core interface for implementing tools,
//! inspired by Codex's ToolHandler design.

use async_trait::async_trait;

use super::context::{ToolInvocation, ToolOutput, ToolPayload};
use super::error::ToolExecutionError;

/// Tool kind - distinguishes different tool types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToolKind {
    /// Native function tool (read_file, apply_patch, etc.)
    Function,
    /// MCP server tool
    Mcp,
}

impl std::fmt::Display for ToolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolKind::Function => write!(f, "function"),
            ToolKind::Mcp => write!(f, "mcp"),
        }
    }
}

/// Core trait for implementing tools
///
/// Implement this trait to create custom tools. Each tool must:
/// 1. Define its kind (Function or Mcp)
/// 2. Handle the appropriate payload type
/// 3. Return appropriate output
///
/// # Example
///
/// ```ignore
/// use async_trait::async_trait;
/// use llm_sdk::tools::{ToolHandler, ToolKind, ToolOutput, ToolInvocation, ToolPayload, ToolExecutionError, OutputBody};
///
/// struct ReadFileHandler;
///
/// #[async_trait]
/// impl ToolHandler for ReadFileHandler {
///     fn kind(&self) -> ToolKind {
///         ToolKind::Function
///     }
///
///     fn matches_kind(&self, payload: &ToolPayload) -> bool {
///         matches!(payload, ToolPayload::Function { .. })
///     }
///
///     async fn is_mutating(&self, _invocation: &ToolInvocation) -> bool {
///         false // read operation
///     }
///
///     async fn handle(&self, invocation: ToolInvocation) -> Result<ToolOutput, ToolExecutionError> {
///         let args: ReadFileArgs = parse_arguments(&arguments)?;
///         // ... implementation
///         Ok(ToolOutput::Function {
///             body: OutputBody::Text(content),
///             success: Some(true),
///         })
///     }
/// }
/// ```
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Returns the tool kind (Function or Mcp)
    fn kind(&self) -> ToolKind;

    /// Returns the tool name
    fn name(&self) -> String;

    /// Returns the tool description
    fn description(&self) -> String;

    /// Returns the JSON schema for tool parameters
    fn parameters_schema(&self) -> serde_json::Value;

    /// Check if this handler can handle the given payload type
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(
            (self.kind(), payload),
            (ToolKind::Function, ToolPayload::Function { .. })
                | (ToolKind::Function, ToolPayload::Custom { .. })
                | (ToolKind::Mcp, ToolPayload::Mcp { .. })
        )
    }

    /// Check if this tool produces mutations (file changes, shell execution, etc.)
    ///
    /// This is used by the orchestrator to determine if approval is needed
    /// before execution. Override this for tools that modify state.
    async fn is_mutating(&self, _invocation: &ToolInvocation) -> bool {
        false
    }

    /// Execute the tool with the given invocation context
    ///
    /// # Arguments
    /// * `invocation` - Complete context including session, turn, and payload
    ///
    /// # Returns
    /// * `Ok(ToolOutput)` - Successful execution result
    /// * `Err(ToolExecutionError)` - Execution failed
    async fn handle(&self, invocation: ToolInvocation) -> Result<ToolOutput, ToolExecutionError>;

    /// Called after successful execution (optional hook)
    ///
    /// Override this to perform post-execution actions like logging
    async fn after_execute(
        &self,
        _invocation: &ToolInvocation,
        _output: &ToolOutput,
    ) -> Result<(), ToolExecutionError> {
        Ok(())
    }
}

/// Helper to parse JSON arguments
#[macro_export]
macro_rules! parse_tool_args {
    ($args:expr, $ty:ty) => {{
        serde_json::from_str::<$ty>($args).map_err(|e| {
            $crate::tools::ToolExecutionError::InvalidArguments(format!(
                "failed to parse arguments: {}",
                e
            ))
        })
    }};
}
