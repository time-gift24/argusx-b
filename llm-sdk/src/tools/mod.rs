// Tools module - Tool execution system inspired by Codex
//
// Core components:
// - handler.rs: ToolHandler trait for implementing tools
// - context.rs: ToolInvocation, ToolPayload, ToolOutput types
// - registry.rs: Tool registration and lookup
// - orchestrator.rs: Approval and execution orchestration
// - error.rs: Tool-related errors

pub mod handler;
pub mod context;
pub mod registry;
pub mod orchestrator;
pub mod error;
pub mod handlers;

pub use handler::{ToolHandler, ToolKind};
pub use context::{
    ToolInvocation, ToolPayload, ToolOutput,
    SessionInfo, TurnContext, TurnDiffTracker,
    OutputBody, McpToolResult, McpContent,
};
pub use registry::{ToolRegistry, ToolRegistryBuilder, ToolSpec};
pub use orchestrator::{ToolOrchestrator, ApprovalGate, ApprovalResult, ApprovalRequirement};
pub use error::{ToolError, ToolExecutionError};
