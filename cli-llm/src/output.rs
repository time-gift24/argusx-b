use crate::cli::OutputFormat;
use llm_sdk::{ModelResponse, PartialModelResponse};
use serde::Serialize;

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
    // For partial responses, extract text from delta
    response
        .delta
        .iter()
        .filter_map(|d| {
            if let llm_sdk::PartDelta::Text(t) = &d.part {
                Some(t.text.clone())
            } else {
                None
            }
        })
        .collect()
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
