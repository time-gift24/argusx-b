//! Tool registry - manages tool registration and lookup

use std::collections::HashMap;
use std::sync::Arc;

use super::context::ToolPayload;
use super::error::ToolExecutionError;
use super::handler::{ToolHandler, ToolKind};

/// Tool specification - metadata about a registered tool
#[derive(Clone, Debug)]
pub struct ToolSpec {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for parameters
    pub parameters: serde_json::Value,
    /// Tool kind
    pub kind: ToolKind,
}

/// Registry for managing tools
pub struct ToolRegistry {
    handlers: HashMap<String, Arc<dyn ToolHandler>>,
    specs: HashMap<String, ToolSpec>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            specs: HashMap::new(),
        }
    }

    /// Get a handler by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.handlers.get(name).cloned()
    }

    /// Get a tool specification by name
    pub fn get_spec(&self, name: &str) -> Option<&ToolSpec> {
        self.specs.get(name)
    }

    /// Check if a tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    /// List all tool names
    pub fn list_names(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Get all tool specifications
    pub fn all_specs(&self) -> Vec<&ToolSpec> {
        self.specs.values().collect()
    }

    /// Dispatch a tool call to the appropriate handler
    pub async fn dispatch(
        &self,
        tool_name: &str,
        payload: ToolPayload,
    ) -> Result<super::context::ToolOutput, ToolExecutionError> {
        let handler = self.get(tool_name).ok_or_else(|| {
            ToolExecutionError::ToolNotFound(tool_name.to_string())
        })?;

        if !handler.matches_kind(&payload) {
            return Err(ToolExecutionError::PayloadMismatch {
                expected: format!("{:?}", handler.kind()),
                actual: format!("{:?}", payload),
            });
        }

        handler.handle(super::context::ToolInvocation {
            session: super::context::SessionInfo::new(
                "unknown".to_string(),
                std::env::current_dir().unwrap_or_default(),
            ),
            turn: super::context::TurnContext {
                cwd: std::env::current_dir().unwrap_or_default(),
                turn_number: 0,
                messages: vec![],
            },
            tracker: super::context::TurnDiffTracker::new(),
            call_id: "unknown".to_string(),
            tool_name: tool_name.to_string(),
            payload,
        }).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing a ToolRegistry
pub struct ToolRegistryBuilder {
    registry: ToolRegistry,
}

impl ToolRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }

    /// Register a tool handler
    pub fn register<H: ToolHandler + 'static>(mut self, handler: H) -> Self {
        let name = handler.name();
        let spec = ToolSpec {
            name: name.clone(),
            description: handler.description(),
            parameters: handler.parameters_schema(),
            kind: handler.kind(),
        };

        self.registry.handlers.insert(name.clone(), Arc::new(handler));
        self.registry.specs.insert(name, spec);
        self
    }

    /// Build the final registry
    pub fn build(self) -> ToolRegistry {
        self.registry
    }
}

impl Default for ToolRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
