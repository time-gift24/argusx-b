use crate::cli::OutputFormat;
use crate::providers::StreamResult;
use anyhow::Result;
use futures::StreamExt;
use llm_sdk::{ModelResponse, Part, PartDelta, PartialModelResponse, TextPart};
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct JsonOutput {
    content: String,
    usage: Option<Usage>,
    cost: Option<f64>,
}

#[derive(Serialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}

pub fn format_output(response: &ModelResponse, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_text(response),
        OutputFormat::Json => format_json(response),
        OutputFormat::Markdown => format_markdown(response),
    }
}

#[allow(dead_code)]
pub fn format_partial_output(response: &PartialModelResponse, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_partial_text(response),
        OutputFormat::Json => format_partial_json(response),
        OutputFormat::Markdown => format_partial_markdown(response),
    }
}

fn format_text(response: &ModelResponse) -> String {
    response
        .content
        .iter()
        .filter_map(|p| {
            if let llm_sdk::Part::Text(t) = p {
                Some(t.text.clone())
            } else {
                None
            }
        })
        .collect()
}

#[allow(dead_code)]
fn format_partial_text(response: &PartialModelResponse) -> String {
    if let Some(delta) = &response.delta {
        if let llm_sdk::PartDelta::Text(t) = &delta.part {
            return t.text.clone();
        }
    }
    String::new()
}

fn format_json(response: &ModelResponse) -> String {
    let content = format_text(response);
    let output = JsonOutput {
        content,
        usage: response.usage.clone().map(|u| Usage {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
        }),
        cost: response.cost,
    };
    serde_json::to_string_pretty(&output).unwrap_or_default()
}

#[allow(dead_code)]
fn format_partial_json(response: &PartialModelResponse) -> String {
    let content = format_partial_text(response);
    let output = JsonOutput {
        content,
        usage: response.usage.clone().map(|u| Usage {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
        }),
        cost: response.cost,
    };
    serde_json::to_string_pretty(&output).unwrap_or_default()
}

fn format_markdown(response: &ModelResponse) -> String {
    let content = format_text(response);
    let mut md = String::new();
    md.push_str("## Response\n\n");
    md.push_str(&content);
    if let Some(usage) = response.usage.clone() {
        md.push_str("\n\n---\n\n");
        md.push_str(&format!(
            "**Usage:** {} in, {} out",
            usage.input_tokens, usage.output_tokens
        ));
    }
    md
}

#[allow(dead_code)]
fn format_partial_markdown(response: &PartialModelResponse) -> String {
    let content = format_partial_text(response);
    let mut md = String::new();
    md.push_str("## Response\n\n");
    md.push_str(&content);
    if let Some(usage) = response.usage.clone() {
        md.push_str("\n\n---\n\n");
        md.push_str(&format!(
            "**Usage:** {} in, {} out",
            usage.input_tokens, usage.output_tokens
        ));
    }
    md
}

/// 处理流式输出，逐字打印到终端并返回完整响应
pub async fn handle_streaming(mut stream: StreamResult<'_>) -> Result<ModelResponse> {
    let mut accumulated_content = String::new();
    let mut final_response: Option<ModelResponse> = None;

    while let Some(result) = stream.next().await {
        let partial = result?;

        // 处理增量内容 - 逐字输出
        if let Some(delta) = partial.delta {
            if let PartDelta::Text(text_delta) = delta.part {
                let new_text = &text_delta.text;
                if !new_text.is_empty() {
                    print!("{}", new_text);
                    std::io::stdout().flush()?;
                    accumulated_content.push_str(new_text);
                }
            }
        }

        // 保留 usage 信息
        if partial.usage.is_some() || partial.cost.is_some() {
            final_response = Some(ModelResponse {
                content: vec![Part::Text(TextPart {
                    text: accumulated_content.clone(),
                    citations: None,
                })],
                usage: partial.usage,
                cost: partial.cost,
            });
        }
    }

    println!(); // 换行

    Ok(final_response.unwrap_or(ModelResponse {
        content: vec![Part::Text(TextPart {
            text: accumulated_content,
            citations: None,
        })],
        usage: None,
        cost: None,
    }))
}
