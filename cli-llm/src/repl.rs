use crate::output::format_output;
use crate::cli::OutputFormat;
use crate::providers::Provider;
use anyhow::Result;
use llm_sdk::{Message, LanguageModelInput};

pub async fn run_repl<P>(provider: P, output_format: OutputFormat) -> Result<()>
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

        match provider.generate(llm_input).await {
            Ok(response) => {
                let text = format_output(&response, &OutputFormat::Text);
                println!("{}", format_output(&response, &output_format));
                println!();

                // Add assistant response to conversation
                messages.push(Message::assistant_text(text));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
