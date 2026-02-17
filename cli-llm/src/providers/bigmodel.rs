use super::*;
use async_stream::stream;
use futures::StreamExt;
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

    async fn stream<'a>(&'a self, input: LanguageModelInput) -> Result<StreamResult<'a>> {
        // 获取真正的流式 stream
        let inner_stream = llm_sdk::LanguageModelTrait::stream(&self.inner, input).await?;

        // 使用 async_stream 创建一个真正的流式 stream
        // 这会保持真正的流式特性，不会预先收集所有数据
        let stream = stream! {
            // 获取 futures stream 的所有权
            let mut futures_stream = inner_stream;

            // 使用 futures::StreamExt::next() 来逐个获取项
            // 这保持了真正的流式特性
            while let Some(item) = futures_stream.next().await {
                match item {
                    Ok(partial) => yield Ok(partial),
                    Err(e) => yield Err(anyhow::anyhow!("{}", e)),
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
