use llm_cli::cli::OutputFormat;
use llm_cli::output::format_output;
use llm_sdk::{ModelResponse, ModelUsage, Part, TextPart};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_format_output_never_panics(content in ".*") {
        let response = ModelResponse {
            content: vec![Part::Text(TextPart {
                text: content.clone(),
                citations: None,
            })],
            usage: None,
            cost: None,
        };

        // Should never panic for any format
        let _ = format_output(&response, &OutputFormat::Text);
        let _ = format_output(&response, &OutputFormat::Json);
        let _ = format_output(&response, &OutputFormat::Markdown);
    }

    #[test]
    fn test_format_output_text_preserves_content(content in ".*") {
        let response = ModelResponse {
            content: vec![Part::Text(TextPart {
                text: content.clone(),
                citations: None,
            })],
            usage: None,
            cost: None,
        };

        let output = format_output(&response, &OutputFormat::Text);
        prop_assert!(output.contains(&content));
    }

    #[test]
    fn test_format_output_json_valid_json(content in "[a-zA-Z0-9 ]*") {
        let response = ModelResponse {
            content: vec![Part::Text(TextPart {
                text: content,
                citations: None,
            })],
            usage: Some(ModelUsage {
                input_tokens: 10,
                output_tokens: 20,
            }),
            cost: Some(0.001),
        };

        let output = format_output(&response, &OutputFormat::Json);
        // Should be valid JSON (serde_json can parse it)
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        prop_assert!(parsed.is_object());
    }

    #[test]
    fn test_format_output_markdown_contains_response(content in ".*") {
        let response = ModelResponse {
            content: vec![Part::Text(TextPart {
                text: content.clone(),
                citations: None,
            })],
            usage: None,
            cost: None,
        };

        let output = format_output(&response, &OutputFormat::Markdown);
        prop_assert!(output.contains("## Response"));
        prop_assert!(output.contains(&content));
    }

    #[test]
    fn test_format_output_markdown_with_usage(
        content in "[a-zA-Z ]*",
        input_tokens in 0u64..10000,
        output_tokens in 0u64..10000
    ) {
        let response = ModelResponse {
            content: vec![Part::Text(TextPart {
                text: content,
                citations: None,
            })],
            usage: Some(ModelUsage {
                input_tokens,
                output_tokens,
            }),
            cost: None,
        };

        let output = format_output(&response, &OutputFormat::Markdown);
        prop_assert!(output.contains("**Usage:**"));
        let expected_in = format!("{} in", input_tokens);
        let expected_out = format!("{} out", output_tokens);
        prop_assert!(output.contains(&expected_in));
        prop_assert!(output.contains(&expected_out));
    }
}
