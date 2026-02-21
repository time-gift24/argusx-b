use super::*;

#[allow(dead_code)]
pub struct AnthropicProvider;

impl AnthropicProvider {
    #[allow(dead_code)]
    pub fn new(_api_key: &str) -> Self {
        Self
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn generate(&self, _input: LanguageModelInput) -> Result<ModelResponse> {
        todo!("Anthropic provider not implemented")
    }

    async fn stream_events(&self, _input: LanguageModelInput) -> Result<StreamResult> {
        todo!("Anthropic provider not implemented")
    }
}
