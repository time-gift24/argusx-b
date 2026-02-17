use anyhow::Result;
use clap::Parser;
use futures::StreamExt;

mod cli;
mod config;
mod output;
mod repl;
mod providers;

use cli::*;
use config::Config;
use providers::Provider;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.provider {
        None => {
            println!("Available providers: bigmodel, openai, anthropic");
            println!("Use 'cli-llm <provider> --help' for more info");
        }
        Some(ProviderCommand::List) => {
            println!("Available providers:");
            println!("  - bigmodel   (supported)");
            println!("  - openai    (TODO)");
            println!("  - anthropic  (TODO)");
        }
        Some(ProviderCommand::Bigmodel(opts)) => {
            run_provider("bigmodel", opts.model, opts.common).await?;
        }
        Some(ProviderCommand::Openai(_)) => {
            println!("OpenAI provider not implemented yet");
        }
        Some(ProviderCommand::Anthropic(_)) => {
            println!("Anthropic provider not implemented yet");
        }
    }

    Ok(())
}

async fn run_provider(provider_name: &str, model: String, opts: CommonOpts) -> Result<()> {
    let config = Config::from_env(provider_name);
    let api_key = opts.api_key.or(config.get_api_key(None))
        .expect("API key required (set BIGMODEL_API_KEY or use --api-key)");

    let stream_enabled = if opts.no_stream { false } else { opts.stream };

    // Check input mode
    if opts.interactive {
        // REPL mode
        let provider = providers::bigmodel::BigmodelProvider::new(&api_key, &model);
        repl::run_repl(provider, opts.output).await?;
    } else if let Some(file) = opts.file {
        // File input mode
        let input = std::fs::read_to_string(&file)?;
        let provider = providers::bigmodel::BigmodelProvider::new(&api_key, &model);
        let messages = vec![llm_sdk::Message::user_text(input)];
        let llm_input = llm_sdk::LanguageModelInput::new(messages);

        if stream_enabled {
            // Stream output
            let mut stream = provider.stream(llm_input).await?;
            while let Some(result) = stream.next().await {
                let response = result?;
                print!("{}", output::format_output(&response, &opts.output));
            }
        } else {
            let response = provider.generate(llm_input).await?;
            println!("{}", output::format_output(&response, &opts.output));
        }
    } else {
        // Pipe mode - read from stdin
        use std::io::{self, Read};
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;

        let provider = providers::bigmodel::BigmodelProvider::new(&api_key, &model);
        let messages = vec![llm_sdk::Message::user_text(input.trim())];
        let llm_input = llm_sdk::LanguageModelInput::new(messages);

        if stream_enabled {
            let mut stream = provider.stream(llm_input).await?;
            while let Some(result) = stream.next().await {
                let response = result?;
                print!("{}", output::format_output(&response, &opts.output));
            }
        } else {
            let response = provider.generate(llm_input).await?;
            println!("{}", output::format_output(&response, &opts.output));
        }
    }

    Ok(())
}
