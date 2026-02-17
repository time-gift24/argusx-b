use crate::error::ModelError;
use crate::error::Result;
use crate::traits::Stream;
use crate::{
    LanguageModelCapability, LanguageModelInput, LanguageModelMetadata, LanguageModelTrait,
    Message, ModelResponse, ModelUsage, Part, PartialModelResponse, TextPart,
};
use async_trait::async_trait;
use bigmodel_api::{
    BigModelClient, ChatRequest, Config, Content, Message as ApiMessage, Role as ApiRole,
};

pub struct BigModelProvider {
    client: BigModelClient,
    model_id: String,
    metadata: Option<LanguageModelMetadata>,
}

impl BigModelProvider {
    pub fn new(api_key: impl Into<String>, model_id: impl Into<String>) -> Self {
        let config = Config::new(api_key);
        let client = BigModelClient::new(config);

        Self {
            client,
            model_id: model_id.into(),
            metadata: Some(LanguageModelMetadata {
                pricing: None,
                capabilities: Some(vec![
                    LanguageModelCapability::TextInput,
                    LanguageModelCapability::TextOutput,
                    LanguageModelCapability::FunctionCalling,
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

    async fn stream(
        &self,
        _input: LanguageModelInput,
    ) -> std::result::Result<
        Box<
            dyn Stream<Item = std::result::Result<PartialModelResponse, ModelError>> + Send + Unpin,
        >,
        ModelError,
    > {
        // TODO: Implement streaming
        Err(ModelError::ServerError(
            "Streaming not yet implemented".to_string(),
        ))
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
