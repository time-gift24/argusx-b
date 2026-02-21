use super::*;
use llm_sdk::{BigModelProvider, LanguageModelInput, LanguageModelTrait, ModelResponse};

pub struct BigmodelProvider {
    inner: BigModelProvider,
}

impl BigmodelProvider {
    pub fn new(api_key: &str, model_id: &str) -> Self {
        Self {
            inner: BigModelProvider::new(api_key, model_id),
        }
    }
}

#[async_trait]
impl Provider for BigmodelProvider {
    fn name(&self) -> &str {
        "bigmodel"
    }

    async fn generate(&self, input: LanguageModelInput) -> Result<ModelResponse> {
        Ok(self.inner.generate(input).await?)
    }

    async fn stream_events(&self, input: LanguageModelInput) -> Result<StreamResult> {
        Ok(self.inner.stream_events(input).await?)
    }
}
