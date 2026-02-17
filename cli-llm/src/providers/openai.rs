use super::*;

pub struct OpenaiProvider;

impl OpenaiProvider {
    pub fn new(_api_key: &str) -> Self {
        Self
    }
}

#[async_trait]
impl Provider for OpenaiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn generate(&self, _input: LanguageModelInput) -> Result<ModelResponse> {
        todo!("OpenAI provider not implemented")
    }

    async fn stream(&self, _input: LanguageModelInput) -> Result<StreamResult> {
        todo!("OpenAI provider not implemented")
    }
}
