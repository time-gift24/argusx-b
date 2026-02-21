use crate::error::ModelError;
use crate::error::Result;
use crate::{
    ContentDelta, LanguageModelCapability, LanguageModelInput, LanguageModelMetadata,
    LanguageModelTrait, Message, ModelResponse, ModelStreamEvent, ModelUsage, Part, PartDelta,
    PartialModelResponse, TextPart, TextPartDelta, TransientErrorEvent,
};
use async_trait::async_trait;
use bigmodel_api::{
    BigModelClient, ChatRequest, Config, Content, Message as ApiMessage,
    Role as ApiRole,
};
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct BigModelProvider {
    client: Arc<BigModelClient>,
    model_id: String,
    metadata: Option<LanguageModelMetadata>,
}

impl BigModelProvider {
    pub fn new(api_key: impl Into<String>, model_id: impl Into<String>) -> Self {
        let config = Config::new(api_key);
        let client = BigModelClient::new(config);

        Self {
            client: Arc::new(client),
            model_id: model_id.into(),
            metadata: Some(LanguageModelMetadata {
                pricing: None,
                capabilities: Some(vec![
                    LanguageModelCapability::TextInput,
                    LanguageModelCapability::TextOutput,
                ]),
            }),
        }
    }
}

#[async_trait]
impl LanguageModelTrait for BigModelProvider {
    fn provider(&self) -> &'static str {
        "bigmodel"
    }

    fn model_id(&self) -> String {
        self.model_id.clone()
    }

    fn metadata(&self) -> Option<LanguageModelMetadata> {
        self.metadata.clone()
    }

    async fn generate(&self, input: LanguageModelInput) -> Result<ModelResponse> {
        // Convert input to BigModel format
        let messages = convert_messages(input.messages, input.system_prompt);

        let mut request = ChatRequest::new(self.model_id.clone(), messages);

        if let Some(t) = input.temperature {
            request = request.temperature(t);
        }
        if let Some(mt) = input.max_tokens {
            request = request.max_tokens(mt as i32);
        }

        let response = self
            .client
            .chat(request)
            .await
            .map_err(|e| ModelError::ServerError(e.to_string()))?;

        Ok(convert_response(response))
    }

    async fn stream_events(
        &self,
        input: LanguageModelInput,
    ) -> std::result::Result<mpsc::Receiver<ModelStreamEvent>, ModelError> {
        let (tx, rx) = mpsc::channel(input.effective_buffer_capacity());

        // Spawn a task to handle the streaming
        let model_id = self.model_id.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            // Convert input to BigModel format
            let messages = convert_messages(input.messages, input.system_prompt);

            let request = ChatRequest::new(model_id.clone(), messages).stream();

            // Note: temperature and max_tokens would be set here if present in input

            // Get the stream from bigmodel-api
            let mut stream = client.chat_stream(request);

            // Collect all deltas and send through channel
            let mut final_response_content: Option<String> = None;

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let partial = convert_chunk_response(chunk);
                        if let Some(ref delta) = partial.delta {
                            if let PartDelta::Text(ref text_delta) = delta.part {
                                final_response_content = Some(
                                    final_response_content
                                        .unwrap_or_default()
                                        .to_string()
                                        + &text_delta.text,
                                );
                            }
                        }
                        if tx.send(ModelStreamEvent::Delta(partial)).await.is_err() {
                            // Receiver dropped, stop streaming
                            return;
                        }
                    }
                    Err(e) => {
                        // Send transient error
                        let _ = tx
                            .send(ModelStreamEvent::TransientError(TransientErrorEvent {
                                message: e.to_string(),
                                is_retryable: true,
                                retry_count: 0,
                            }))
                            .await;
                        // For now, just break on error (retry logic would go here)
                        break;
                    }
                }
            }

            // Send completion event
            if let Some(content) = final_response_content {
                let response = ModelResponse {
                    content: vec![Part::Text(TextPart {
                        text: content,
                        citations: None,
                    })],
                    usage: None,
                    cost: None,
                };
                let _ = tx.send(ModelStreamEvent::Complete(response)).await;
            } else {
                // No content received, send error
                let _ = tx
                    .send(ModelStreamEvent::Error(ModelError::ServerError(
                        "No response from model".to_string(),
                    )))
                    .await;
            }
        });

        Ok(rx)
    }
}

// Helper functions for type conversion
fn convert_messages(messages: Vec<Message>, system_prompt: Option<String>) -> Vec<ApiMessage> {
    let mut result = Vec::new();

    if let Some(prompt) = system_prompt {
        result.push(ApiMessage {
            role: ApiRole::System,
            content: Content::Text(prompt),
            reasoning_content: None,
        });
    }

    for msg in messages {
        match msg {
            Message::User { content } => {
                result.push(ApiMessage {
                    role: ApiRole::User,
                    content: convert_content(content),
                    reasoning_content: None,
                });
            }
            Message::Assistant { content } => {
                result.push(ApiMessage {
                    role: ApiRole::Assistant,
                    content: convert_content(content),
                    reasoning_content: None,
                });
            }
            Message::Tool { content } => {
                result.push(ApiMessage {
                    role: ApiRole::Tool,
                    content: convert_content(content),
                    reasoning_content: None,
                });
            }
        }
    }

    result
}

fn convert_content(parts: Vec<Part>) -> Content {
    for part in parts {
        if let Part::Text(tp) = part {
            return Content::Text(tp.text);
        }
    }
    Content::Text(String::new())
}

fn convert_response(response: bigmodel_api::ChatResponse) -> ModelResponse {
    let content = response
        .choices
        .first()
        .map(|c| {
            let text = match &c.message.content {
                bigmodel_api::Content::Text(s) => s.clone(),
                _ => String::new(),
            };
            vec![Part::Text(TextPart {
                text,
                citations: None,
            })]
        })
        .unwrap_or_default();

    ModelResponse {
        content,
        usage: response.usage.map(|u| ModelUsage {
            input_tokens: u.prompt_tokens as u64,
            output_tokens: u.completion_tokens as u64,
        }),
        cost: None,
    }
}

fn convert_chunk_response(chunk: bigmodel_api::ChatResponseChunk) -> PartialModelResponse {
    let delta = chunk.choices.first().map(|c| ContentDelta {
        index: c.index as usize,
        part: PartDelta::Text(TextPartDelta {
            text: c.delta.content.clone().unwrap_or_default(),
        }),
    });

    PartialModelResponse {
        delta,
        usage: None,
        cost: None,
    }
}
