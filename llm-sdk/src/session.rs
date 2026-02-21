//! Run session implementation for the LLM SDK.
//!
//! This module provides the RunSession implementation that orchestrates
//! the agent loop with tool execution.

use crate::domain::*;
use crate::error::SessionError;
use crate::traits::{AgentParams, LanguageModelTrait, RunSessionTrait};
use crate::tools::{SessionInfo, ToolExecutionError, ToolHandler, ToolInvocation, ToolOrchestrator, ToolOutput, ToolPayload, TurnContext};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Inner state for run session.
pub struct RunStateInner {
    /// Items in the conversation.
    pub items: Vec<AgentItem>,
    /// Current turn count.
    pub turn_count: u32,
}

impl RunStateInner {
    /// Create a new run state.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            turn_count: 0,
        }
    }

    /// Build messages for the current turn from items.
    pub fn build_turn_messages(&self) -> Vec<Message> {
        self.items
            .iter()
            .filter_map(|item| match item {
                AgentItem::Message(msg) => Some(msg.clone()),
                AgentItem::Model(model) => Some(Message::Assistant {
                    content: model.content.clone(),
                }),
                AgentItem::Tool(tool) => Some(Message::Tool {
                    content: vec![Part::ToolResult(ToolResultPart {
                        tool_call_id: tool.tool_call_id.clone(),
                        tool_name: tool.tool_name.clone(),
                        content: tool.output.clone(),
                        is_error: tool.is_error,
                    })],
                }),
            })
            .collect()
    }

    /// Append an item to the state and return its index.
    pub fn append_item(&mut self, item: AgentItem) -> usize {
        let index = self.items.len();
        self.items.push(item);
        index
    }
}

impl Default for RunStateInner {
    fn default() -> Self {
        Self::new()
    }
}

/// Run session for agent execution.
pub struct RunSession {
    model: Arc<dyn LanguageModelTrait>,
    state: Arc<Mutex<RunStateInner>>,
    params: Arc<AgentParams>,
    tools: HashMap<String, Arc<dyn ToolHandler>>,
}

impl RunSession {
    /// Create a new run session.
    pub fn new(model: Arc<dyn LanguageModelTrait>, params: AgentParams) -> Self {
        // Build tool map from params
        let mut tools = HashMap::new();
        for tool in &params.tools {
            let name = tool.name();
            tools.insert(name, tool.clone());
        }

        Self {
            model,
            state: Arc::new(Mutex::new(RunStateInner::new())),
            params: Arc::new(params),
            tools,
        }
    }
}

/// Convert ToolOutput to Vec<Part> for agent consumption.
fn tool_output_to_parts(output: ToolOutput) -> Vec<Part> {
    match output {
        ToolOutput::Function { body, .. } => {
            let text = body.as_text().unwrap_or_default().to_string();
            vec![Part::Text(TextPart {
                text,
                citations: None,
            })]
        }
        ToolOutput::Mcp { result } => {
            match result {
                Ok(mcp_result) => {
                    // Convert Vec<McpContent> to text
                    let text = mcp_result.content
                        .iter()
                        .map(|c| format!("{:?}", c))
                        .collect::<Vec<_>>()
                        .join("\n");
                    vec![Part::Text(TextPart {
                        text,
                        citations: None,
                    })]
                }
                Err(e) => {
                    vec![Part::Text(TextPart {
                        text: e,
                        citations: None,
                    })]
                }
            }
        }
    }
}

#[async_trait]
impl RunSessionTrait for RunSession {
    /// Run the agent with input items (non-streaming).
    async fn run(&self, input: Vec<AgentItem>) -> Result<AgentResponse, SessionError> {
        let mut rx = self.run_events(input).await?;
        // Collect all events
        let mut final_response: Option<AgentResponse> = None;
        let mut last_error: Option<SessionError> = None;

        while let Some(event) = rx.recv().await {
            match event {
                RunStreamEvent::Complete(response) => {
                    final_response = Some(response);
                }
                RunStreamEvent::Error(err) => {
                    last_error = Some(err);
                }
                _ => {}
            }
        }

        if let Some(response) = final_response {
            Ok(response)
        } else if let Some(err) = last_error {
            Err(err)
        } else {
            Err(SessionError::Session(Some(
                "No response received".to_string(),
            )))
        }
    }

    /// Run the agent in streaming mode.
    async fn run_events(
        &self,
        input: Vec<AgentItem>,
    ) -> Result<mpsc::Receiver<RunStreamEvent>, SessionError> {
        let (tx, rx) = mpsc::channel(128);

        // Clone everything needed for the spawned task
        let model = self.model.clone();
        let state = self.state.clone();
        let params = self.params.clone();
        let tools = self.tools.clone();

        // Spawn the agent loop
        tokio::spawn(async move {
            // Phase 1: InitTurn - append input to state
            {
                let mut state_guard = state.lock().await;
                for item in input {
                    state_guard.append_item(item);
                }
            }

            // Agent loop
            loop {
                // Check max turns
                if let Some(max_turns) = params.max_turns {
                    let current_turns = state.lock().await.turn_count;
                    if current_turns >= max_turns {
                        let _ = tx
                            .send(RunStreamEvent::Error(SessionError::MaxTurnsExceeded))
                            .await;
                        break;
                    }
                }

                // Phase 2: Sampling - build model input and call stream_events
                let model_input = {
                    let state_guard = state.lock().await;
                    let messages = state_guard.build_turn_messages();

                    let mut input = LanguageModelInput::new(messages);
                    if let Some(ref system_prompt) = params.system_prompt {
                        input = input.with_system_prompt(system_prompt.clone());
                    }
                    input
                };

                // Call model streaming
                let mut model_rx = match model.stream_events(model_input).await {
                    Ok(rx) => rx,
                    Err(e) => {
                        let _ = tx.send(RunStreamEvent::Error(e.into())).await;
                        break;
                    }
                };

                // Forward model events and collect final response
                let mut final_model_response: Option<ModelResponse> = None;

                while let Some(event) = model_rx.recv().await {
                    match event {
                        ModelStreamEvent::Delta(partial) => {
                            if tx.send(RunStreamEvent::Delta(partial)).await.is_err() {
                                // Receiver dropped
                                return;
                            }
                        }
                        ModelStreamEvent::TransientError(err) => {
                            if tx.send(RunStreamEvent::TransientError(err)).await.is_err() {
                                return;
                            }
                        }
                        ModelStreamEvent::Complete(response) => {
                            final_model_response = Some(response);
                        }
                        ModelStreamEvent::Error(err) => {
                            let _ = tx.send(RunStreamEvent::Error(err.into())).await;
                            return;
                        }
                    }
                }

                // Phase 3: Handle model response
                let model_response = match final_model_response {
                    Some(r) => r,
                    None => {
                        // No response received
                        let _ = tx
                            .send(RunStreamEvent::Error(SessionError::Session(Some(
                                "No response from model".to_string(),
                            ))))
                            .await;
                        break;
                    }
                };

                // Append model response to state
                let model_index = {
                    let mut state_guard = state.lock().await;
                    state_guard.append_item(AgentItem::Model(model_response.clone()))
                };

                // Phase 4: ToolDispatch - extract tool calls and execute
                let tool_calls: Vec<_> = model_response
                    .content
                    .iter()
                    .filter_map(|part| {
                        if let Part::ToolCall(tc) = part {
                            Some(tc.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                if tool_calls.is_empty() {
                    // No tool calls - complete the run
                    let response = AgentResponse::from(model_response);
                    if tx.send(RunStreamEvent::Complete(response)).await.is_err() {
                        return;
                    }
                    break;
                }

                // Execute each tool call
                let orchestrator = ToolOrchestrator::with_default_gate();
                for tool_call in tool_calls {
                    let tool_call_id = tool_call.tool_call_id.clone();
                    let tool_name = tool_call.tool_name.clone();

                    // Send ItemStarted
                    let item_id = tool_call_id.clone();
                    let tool_item = AgentItem::Tool(AgentItemTool {
                        tool_call_id: tool_call_id.clone(),
                        tool_name: tool_name.clone(),
                        input: tool_call.args.clone(),
                        output: Vec::new(),
                        is_error: false,
                    });
                    if tx.send(RunStreamEvent::ItemStarted {
                        index: model_index,
                        item_id: item_id.clone(),
                        item: tool_item.clone(),
                    }).await.is_err() {
                        return;
                    }

                    // Get the handler from registry
                    let handler = match tools.get(&tool_name) {
                        Some(h) => h,
                        None => {
                            // Tool not found - send error
                            let error_output = vec![Part::Text(TextPart {
                                text: format!("Tool not found: {}", tool_name),
                                citations: None,
                            })];
                            if tx.send(RunStreamEvent::ItemCompleted {
                                index: model_index,
                                item_id: item_id.clone(),
                                item: AgentItem::Tool(AgentItemTool {
                                    tool_call_id: tool_call_id.clone(),
                                    tool_name: tool_name.clone(),
                                    input: tool_call.args,
                                    output: error_output.clone(),
                                    is_error: true,
                                }),
                            }).await.is_err() {
                                return;
                            }
                            continue;
                        }
                    };

                    // Create tool invocation
                    let invocation = ToolInvocation::new(
                        SessionInfo::new("session".to_string(), std::env::current_dir().unwrap_or_default()),
                        TurnContext {
                            cwd: std::env::current_dir().unwrap_or_default(),
                            turn_number: 1,
                            messages: vec![],
                        },
                        tool_call_id.clone(),
                        tool_name.clone(),
                        ToolPayload::Function {
                            arguments: tool_call.args.to_string(),
                        },
                    );

                    // Execute the tool
                    let result: Result<ToolOutput, ToolExecutionError> = orchestrator
                        .execute_trusted(handler.as_ref(), invocation)
                        .await;

                    let (output, is_error): (Vec<Part>, bool) = match result {
                        Ok(tool_output) => (tool_output_to_parts(tool_output), false),
                        Err(e) => (
                            vec![Part::Text(TextPart {
                                text: e.to_string(),
                                citations: None,
                            })],
                            true,
                        ),
                    };

                    // Update the tool item with output
                    let tool_item = AgentItemTool {
                        tool_call_id: tool_call_id.clone(),
                        tool_name: tool_name.clone(),
                        input: tool_call.args.clone(),
                        output: output.clone(),
                        is_error,
                    };

                    // Send ItemCompleted
                    if tx.send(RunStreamEvent::ItemCompleted {
                        index: model_index,
                        item_id: item_id.clone(),
                        item: AgentItem::Tool(tool_item),
                    }).await.is_err() {
                        return;
                    }

                    // Append tool result to state
                    {
                        let mut state_guard = state.lock().await;
                        state_guard.append_item(AgentItem::Tool(AgentItemTool {
                            tool_call_id,
                            tool_name,
                            input: tool_call.args,
                            output,
                            is_error,
                        }));
                    }
                }

                // Increment turn count and loop
                {
                    let mut state_guard = state.lock().await;
                    state_guard.turn_count += 1;
                }
            }
        });

        Ok(rx)
    }

    /// Close the session.
    async fn close(self: Box<Self>) -> Result<(), SessionError> {
        // For now, just allow the state to be dropped
        // In the future, could cancel any pending operations
        Ok(())
    }
}
