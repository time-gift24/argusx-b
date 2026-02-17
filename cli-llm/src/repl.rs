use crate::cli::OutputFormat;
use crate::providers::{Provider, StreamResult};
use anyhow::Result;
use futures::StreamExt;
use llm_sdk::{LanguageModelInput, Message, PartDelta};

async fn handle_stream_in_repl(mut stream: StreamResult<'_>) -> Result<String> {
    use std::io::Write;

    let mut accumulated = String::new();

    while let Some(result) = stream.next().await {
        let partial = result?;

        if let Some(delta) = partial.delta {
            if let PartDelta::Text(text_delta) = delta.part {
                let new_text = &text_delta.text;
                if !new_text.is_empty() {
                    print!("{}", new_text);
                    std::io::stdout().flush()?;
                    accumulated.push_str(new_text);
                }
            }
        }
    }

    println!();
    Ok(accumulated)
}

pub async fn run_repl<P>(provider: P, _output_format: OutputFormat) -> Result<()>
where
    P: Provider,
{
    use std::io::{self, Write};

    println!("cli-llm {} (type 'exit' to quit)", provider.name());
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
        match provider.stream(llm_input).await {
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
