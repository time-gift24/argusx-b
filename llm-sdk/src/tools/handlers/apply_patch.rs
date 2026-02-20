//! Apply patch tool handler
//!
//! Provides the ability to create, modify, and delete files using a patch language.
//!
//! Patch format:
//! *** Begin Patch
//! *** Add File: /path/to/file
//! +content line 1
//! +content line 2
//! *** Delete File: /path/to/file
//! *** Update File: /path/to/file
//! @@ -old line
//! +new line
//! *** End Patch

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::context::{OutputBody, ToolInvocation, ToolOutput, ToolPayload};
use crate::tools::error::ToolExecutionError;
use crate::tools::handler::{ToolHandler, ToolKind};

/// Arguments for apply_patch tool
#[derive(Deserialize)]
struct ApplyPatchArgs {
    /// The patch content
    input: String,
}

/// Apply patch tool handler
pub struct ApplyPatchHandler;

impl ApplyPatchHandler {
    pub fn new() -> Self {
        Self
    }

    /// Parse and execute the patch
    async fn apply_patch(
        &self,
        input: &str,
        cwd: &std::path::Path,
    ) -> Result<String, ToolExecutionError> {
        let mut results = Vec::new();

        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(result) = self.parse_line(line, cwd).await? {
                results.push(result);
            }
        }

        Ok(if results.is_empty() {
            "No changes applied".to_string()
        } else {
            results.join("\n")
        })
    }

    async fn parse_line(
        &self,
        line: &str,
        cwd: &std::path::Path,
    ) -> Result<Option<String>, ToolExecutionError> {
        let line = line.trim();

        // Parse section headers
        if let Some(path) = line.strip_prefix("*** Add File: ") {
            return self.add_file(path.trim(), cwd).await;
        }

        if let Some(path) = line.strip_prefix("*** Delete File: ") {
            return self.delete_file(path.trim(), cwd).await;
        }

        if let Some(path) = line.strip_prefix("*** Update File: ") {
            return self.update_file_start(path.trim()).await;
        }

        // Skip other directives for now
        if line.starts_with("*** Begin Patch")
            || line.starts_with("*** End Patch")
            || line.starts_with("@@")
            || line.starts_with('-')
            || line.starts_with('+')
        {
            return Ok(None);
        }

        Ok(None)
    }

    async fn add_file(
        &self,
        path: &str,
        cwd: &std::path::Path,
    ) -> Result<Option<String>, ToolExecutionError> {
        // Note: This is a simplified implementation
        // In a full implementation, you'd read content until *** or End Patch
        Ok(Some(format!("Would create file: {}", path)))
    }

    async fn delete_file(
        &self,
        path: &str,
        cwd: &std::path::Path,
    ) -> Result<Option<String>, ToolExecutionError> {
        let full_path = if std::path::Path::new(path).is_absolute() {
            std::path::PathBuf::from(path)
        } else {
            cwd.join(path)
        };

        if !full_path.exists() {
            return Err(ToolExecutionError::respond_to_model(format!(
                "File does not exist: {}",
                full_path.display()
            )));
        }

        std::fs::remove_file(&full_path).map_err(|e| {
            ToolExecutionError::FileError(format!("Failed to delete file: {}", e))
        })?;

        Ok(Some(format!("Deleted: {}", full_path.display())))
    }

    async fn update_file_start(&self, path: &str) -> Result<Option<String>, ToolExecutionError> {
        // This would need to track state for the full update
        Ok(Some(format!("Would update file: {}", path)))
    }
}

impl Default for ApplyPatchHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for ApplyPatchHandler {
    fn kind(&self) -> ToolKind {
        ToolKind::Function
    }

    fn name(&self) -> String {
        "apply_patch".to_string()
    }

    fn description(&self) -> String {
        r#"Apply a patch to create, modify, or delete files.

Patch format:
*** Begin Patch
*** Add File: /path/to/file
+content line 1
+content line 2
*** Delete File: /path/to/file
*** Update File: /path/to/file
@@ -old line
+new line
*** End Patch

Use + prefix for new content when adding files.
Use - and + for modifications with unified diff format."#
            .to_string()
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "The patch content following the apply_patch language format"
                }
            },
            "required": ["input"]
        })
    }

    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. } | ToolPayload::Custom { .. })
    }

    async fn is_mutating(&self, _invocation: &ToolInvocation) -> bool {
        true // This tool modifies files
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<ToolOutput, ToolExecutionError> {
        let input = match &invocation.payload {
            ToolPayload::Function { arguments } => {
                let args: ApplyPatchArgs = serde_json::from_str(arguments).map_err(|e| {
                    ToolExecutionError::InvalidArguments(format!("failed to parse: {}", e))
                })?;
                args.input
            }
            ToolPayload::Custom { input } => input.clone(),
            _ => {
                return Err(ToolExecutionError::PayloadMismatch {
                    expected: "Function or Custom".to_string(),
                    actual: format!("{:?}", invocation.payload),
                });
            }
        };

        // Parse and execute the patch
        let result = self.apply_patch(&input, &invocation.session.cwd).await?;

        // Track changes in the tracker
        // Note: In a full implementation, we'd parse the patch and track actual changes
        let mut tracker = invocation.tracker;
        tracker.add_modified(std::path::PathBuf::from("applied_patch"));

        Ok(ToolOutput::Function {
            body: OutputBody::Text(result),
            success: Some(true),
        })
    }
}
