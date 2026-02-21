//! Tool orchestrator - handles approval and execution coordination
//!
//! The orchestrator manages the complete tool execution lifecycle:
//! 1. Check if approval is required (based on is_mutating)
//! 2. Wait for user approval if needed
//! 3. Execute the tool
//! 4. Return the result

use std::sync::Arc;

use async_trait::async_trait;

use super::context::{ToolInvocation, ToolOutput};
use super::error::ToolExecutionError;
use super::handler::ToolHandler;

/// Approval requirement level
#[derive(Clone, Debug)]
pub enum ApprovalRequirement {
    /// No approval needed
    None,
    /// User approval required
    NeedsApproval { reason: Option<String> },
    /// Automatically approved (e.g., read-only operations)
    AutoApprove,
}

/// Approval result
#[derive(Clone, Debug)]
pub struct ApprovalResult {
    /// Whether the action is approved
    pub approved: bool,
    /// Reason if denied
    pub reason: Option<String>,
    /// Whether sandbox should be bypassed
    pub bypass_sandbox: bool,
}

/// Approval gate trait - determines if tools need approval
#[async_trait]
pub trait ApprovalGate: Send + Sync {
    /// Check if an invocation requires approval
    async fn check_approval(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<ApprovalResult, ToolExecutionError>;

    /// Request approval from user (blocking)
    async fn request_approval(
        &self,
        invocation: &ToolInvocation,
    ) -> Result<ApprovalResult, ToolExecutionError>;
}

/// Default approval gate that approves all read-only operations
pub struct DefaultApprovalGate;

#[async_trait]
impl ApprovalGate for DefaultApprovalGate {
    async fn check_approval(
        &self,
        _invocation: &ToolInvocation,
    ) -> Result<ApprovalResult, ToolExecutionError> {
        // Default: deny all mutating operations, allow read operations
        Ok(ApprovalResult {
            approved: false, // Default to deny, let request_approval handle the UI
            reason: Some("Default approval gate: mutating operations require approval".to_string()),
            bypass_sandbox: false,
        })
    }

    async fn request_approval(
        &self,
        _invocation: &ToolInvocation,
    ) -> Result<ApprovalResult, ToolExecutionError> {
        // Default implementation - in production, this would show a UI prompt
        Ok(ApprovalResult {
            approved: false,
            reason: Some("Default approval gate: approval UI not implemented".to_string()),
            bypass_sandbox: false,
        })
    }
}

/// Tool orchestrator - coordinates execution with approval
pub struct ToolOrchestrator {
    approval_gate: Arc<dyn ApprovalGate>,
}

impl ToolOrchestrator {
    /// Create a new orchestrator with the given approval gate
    pub fn new(approval_gate: Arc<dyn ApprovalGate>) -> Self {
        Self { approval_gate }
    }

    /// Create with default approval gate
    pub fn with_default_gate() -> Self {
        Self {
            approval_gate: Arc::new(DefaultApprovalGate),
        }
    }

    /// Execute a tool with approval handling
    pub async fn execute(
        &self,
        handler: &dyn ToolHandler,
        invocation: ToolInvocation,
    ) -> Result<ToolOutput, ToolExecutionError> {
        // 1. Check if this tool is mutating
        let is_mutating = handler.is_mutating(&invocation).await;

        // 2. Handle approval for mutating operations
        if is_mutating {
            let approval = self.approval_gate.check_approval(&invocation).await?;

            if !approval.approved {
                // Request approval from user
                let approval = self.approval_gate.request_approval(&invocation).await?;

                if !approval.approved {
                    return Err(ToolExecutionError::ApprovalDenied(
                        approval
                            .reason
                            .unwrap_or_else(|| "Approval denied".to_string()),
                    ));
                }
            }
        }

        // 3. Execute the tool
        let output = handler.handle(invocation.clone()).await?;

        // 4. Post-execution hook
        let _ = handler.after_execute(&invocation, &output).await;

        Ok(output)
    }

    /// Execute without approval checks (for trusted internal use)
    pub async fn execute_trusted(
        &self,
        handler: &dyn ToolHandler,
        invocation: ToolInvocation,
    ) -> Result<ToolOutput, ToolExecutionError> {
        handler.handle(invocation).await
    }
}

impl Default for ToolOrchestrator {
    fn default() -> Self {
        Self::with_default_gate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::context::{OutputBody, SessionInfo, ToolPayload, TurnContext};
    use crate::tools::handler::ToolKind;

    struct TestHandler;

    #[async_trait]
    impl ToolHandler for TestHandler {
        fn kind(&self) -> ToolKind {
            ToolKind::Function
        }

        fn name(&self) -> String {
            "test".to_string()
        }

        fn description(&self) -> String {
            "test tool".to_string()
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({ "type": "object", "properties": {} })
        }

        async fn handle(
            &self,
            _invocation: ToolInvocation,
        ) -> Result<ToolOutput, ToolExecutionError> {
            Ok(ToolOutput::Function {
                body: OutputBody::Text("test result".to_string()),
                success: Some(true),
            })
        }
    }

    #[tokio::test]
    async fn execute_trusted_returns_handler_output() {
        let orchestrator = ToolOrchestrator::with_default_gate();
        let handler = TestHandler;
        let invocation = ToolInvocation::new(
            SessionInfo::new("test-session".to_string(), std::env::temp_dir()),
            TurnContext {
                cwd: std::env::temp_dir(),
                turn_number: 1,
                messages: vec![],
            },
            "call-1".to_string(),
            "test".to_string(),
            ToolPayload::Function {
                arguments: "{}".to_string(),
            },
        );

        let output = orchestrator
            .execute_trusted(&handler, invocation)
            .await
            .expect("trusted execution should succeed");

        match output {
            ToolOutput::Function { body, success } => {
                assert_eq!(body.as_text(), Some("test result"));
                assert_eq!(success, Some(true));
            }
            _ => panic!("expected function output"),
        }
    }
}
