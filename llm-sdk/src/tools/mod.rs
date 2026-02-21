// Tools module - Tool execution system inspired by Codex
//
// Core components:
// - handler.rs: ToolHandler trait for implementing tools
// - context.rs: ToolInvocation, ToolPayload, ToolOutput types
// - registry.rs: Tool registration and lookup
// - orchestrator.rs: Approval and execution orchestration
// - error.rs: Tool-related errors

pub mod context;
pub mod error;
pub mod handler;
pub mod handlers;
pub mod orchestrator;
pub mod registry;

pub use context::{
    McpContent, McpToolResult, OutputBody, SessionInfo, ToolInvocation, ToolOutput, ToolPayload,
    TurnContext, TurnDiffTracker,
};
pub use error::{ToolError, ToolExecutionError};
pub use handler::{ToolHandler, ToolKind};
pub use orchestrator::{ApprovalGate, ApprovalRequirement, ApprovalResult, ToolOrchestrator};
pub use registry::{ToolRegistry, ToolRegistryBuilder, ToolSpec};
