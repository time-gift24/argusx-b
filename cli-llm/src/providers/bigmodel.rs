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

    async fn stream(&self, input: LanguageModelInput) -> Result<StreamResult> {
        // For now, convert to non-streaming and return as a single-item stream
        // A proper streaming implementation requires a Waker-based adapter
        // to bridge llm_sdk::Stream with futures::Stream
        let response = self.inner.generate(input).await?;

        // Return a stream with single response
        let stream = futures::stream::iter(vec![Ok(response)]);
        Ok(Box::pin(stream))
    }
}
