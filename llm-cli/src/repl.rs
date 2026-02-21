use crate::cli::OutputFormat;
use crate::output;
use crate::providers::{Provider, StreamResult};
use anyhow::Result;
use llm_sdk::{LanguageModelInput, Message};

async fn handle_stream_in_repl(stream: StreamResult) -> Result<String> {
    let response = output::handle_streaming(stream).await?;
    // 从 ModelResponse 中提取文本内容
    let text = response
        .content
        .iter()
        .filter_map(|p| {
            if let llm_sdk::Part::Text(t) = p {
                Some(t.text.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");
    Ok(text)
}

pub async fn run_repl<P>(provider: P, _output_format: OutputFormat) -> Result<()>
where
    P: Provider,
{
    use std::io::{self, Write};

    println!("llm-cli {} (type 'exit' to quit)", provider.name());
    println!();

    let mut messages = Vec::new();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        messages.push(Message::user_text(input));

        let llm_input = LanguageModelInput::new(messages.clone());

        // 使用流式输出
        match provider.stream_events(llm_input).await {
            Ok(stream) => {
                let text = handle_stream_in_repl(stream).await?;
                messages.push(Message::assistant_text(text));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
