use crate::cli::OutputFormat;
use llm_sdk::ModelResponse;
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

fn format_text(response: &ModelResponse) -> String {
    response.content.iter()
        .filter_map(|p| {
            if let llm_sdk::Part::Text(t) = p {
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

fn format_markdown(response: &ModelResponse) -> String {
    let content = format_text(response);
    let mut md = String::new();
    md.push_str("## Response\n\n");
    md.push_str(&content);
    if let Some(usage) = response.usage.clone() {
        md.push_str("\n\n---\n\n");
        md.push_str(&format!("**Usage:** {} in, {} out",
            usage.input_tokens, usage.output_tokens));
    }
    md
}
