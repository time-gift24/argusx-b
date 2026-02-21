use std::io::{self, IsTerminal, Read, Write};
use std::sync::Arc;

use agent_core::{
    new_id, InputEnvelope, RunEventStream, RunStreamEvent, Runtime, SessionMeta, TurnRequest,
    UiEventStream, UiThreadEvent,
};
use agent_turn::adapters::bigmodel::{BigModelAdapterConfig, BigModelModelAdapter};
use agent_turn::effect::ToolExecutor;
use agent_turn::state::RetryPolicy;
use agent_turn::{TurnEngineConfig, TurnRuntime};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use bigmodel_api::{BigModelClient, Config};
use clap::Parser;
use futures::StreamExt;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "agent-turn-cli")]
#[command(about = "Manual test CLI for agent-turn runtime")]
struct Cli {
    #[arg(help = "Prompt text; if omitted, reads from stdin")]
    prompt: Option<String>,

    #[arg(long, env = "BIGMODEL_API_KEY")]
    api_key: String,

    #[arg(
        long,
        env = "BIGMODEL_BASE_URL",
        default_value = "https://open.bigmodel.cn/api/paas/v4"
    )]
    base_url: String,

    #[arg(long, default_value = "glm-4.5")]
    model: String,

    #[arg(long)]
    system_prompt: Option<String>,

    #[arg(long)]
    max_tokens: Option<i32>,

    #[arg(long)]
    temperature: Option<f32>,

    #[arg(long)]
    top_p: Option<f32>,

    #[arg(long)]
    session_id: Option<String>,

    #[arg(long)]
    turn_id: Option<String>,

    #[arg(long, help = "Print run stream events")]
    show_run: bool,

    #[arg(long, help = "Output all events in JSON lines")]
    json: bool,

    #[arg(long, default_value_t = 4)]
    max_parallel_tools: usize,

    #[arg(long, default_value_t = 3)]
    max_retries: u32,

    #[arg(long, default_value_t = 200)]
    base_delay_ms: u64,
}

struct CliToolExecutor;

#[async_trait]
impl ToolExecutor for CliToolExecutor {
    async fn execute_tool(
        &self,
        call: agent_core::ToolCall,
        _epoch: u64,
    ) -> Result<serde_json::Value, String> {
        match call.tool_name.as_str() {
            "echo" => Ok(call.arguments),
            other => Err(format!(
                "unsupported tool '{other}' in agent-turn-cli; built-in tools: echo"
            )),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let prompt = resolve_prompt(cli.prompt)?;

    let client = Arc::new(BigModelClient::new(
        Config::new(cli.api_key).with_base_url(cli.base_url),
    ));

    let model_cfg = BigModelAdapterConfig {
        model: cli.model,
        system_prompt: cli.system_prompt,
        max_tokens: cli.max_tokens,
        temperature: cli.temperature,
        top_p: cli.top_p,
    };

    let model = Arc::new(BigModelModelAdapter::new(client).with_config(model_cfg));
    let tools = Arc::new(CliToolExecutor);

    let runtime_cfg = TurnEngineConfig {
        max_parallel_tools: cli.max_parallel_tools,
        retry_policy: RetryPolicy {
            max_retries: cli.max_retries,
            base_delay_ms: cli.base_delay_ms,
        },
    };

    let runtime = TurnRuntime::new(model, tools, runtime_cfg);

    let request = TurnRequest {
        meta: SessionMeta::new(
            cli.session_id.unwrap_or_else(new_id),
            cli.turn_id.unwrap_or_else(new_id),
        ),
        initial_input: InputEnvelope::user_text(prompt),
    };

    let streams = runtime.run_turn(request).await?;
    stream_events(streams.run, streams.ui, cli.show_run, cli.json).await
}

fn resolve_prompt(cli_prompt: Option<String>) -> Result<String> {
    if let Some(prompt) = cli_prompt {
        let trimmed = prompt.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
    }

    if io::stdin().is_terminal() {
        bail!("missing prompt: pass text arg or pipe stdin")
    }

    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("failed to read stdin")?;

    let trimmed = buffer.trim().to_string();
    if trimmed.is_empty() {
        bail!("stdin prompt is empty")
    }

    Ok(trimmed)
}

async fn stream_events(
    mut run: RunEventStream,
    mut ui: UiEventStream,
    show_run: bool,
    as_json: bool,
) -> Result<()> {
    let mut run_open = true;
    let mut ui_open = true;
    let mut printed_text = false;

    while run_open || ui_open {
        tokio::select! {
            event = run.next(), if run_open => {
                match event {
                    Some(event) => handle_run_event(event, show_run, as_json)?,
                    None => run_open = false,
                }
            }
            event = ui.next(), if ui_open => {
                match event {
                    Some(event) => {
                        if handle_ui_event(event, as_json)? {
                            printed_text = true;
                        }
                    }
                    None => ui_open = false,
                }
            }
        }
    }

    if printed_text && !as_json {
        println!();
    }

    Ok(())
}

fn handle_run_event(event: RunStreamEvent, show_run: bool, as_json: bool) -> Result<()> {
    if as_json {
        print_json("run", &event)?;
        return Ok(());
    }

    if show_run {
        eprintln!("[run] {event:?}");
    }

    Ok(())
}

fn handle_ui_event(event: UiThreadEvent, as_json: bool) -> Result<bool> {
    if as_json {
        print_json("ui", &event)?;
        return Ok(false);
    }

    match event {
        UiThreadEvent::MessageDelta { delta, .. } => {
            print!("{delta}");
            io::stdout().flush().context("failed to flush stdout")?;
            Ok(true)
        }
        UiThreadEvent::ReasoningDelta { delta, .. } => {
            eprintln!("[reasoning] {delta}");
            Ok(false)
        }
        UiThreadEvent::ToolCallRequested {
            call_id, tool_name, ..
        } => {
            eprintln!("[tool] requested: {tool_name} ({call_id})");
            Ok(false)
        }
        UiThreadEvent::ToolCallProgress {
            call_id, status, ..
        } => {
            eprintln!("[tool] progress: {call_id} -> {status:?}");
            Ok(false)
        }
        UiThreadEvent::ToolCallCompleted { result, .. } => {
            eprintln!("[tool] completed: {}", result.call_id);
            Ok(false)
        }
        UiThreadEvent::Warning { message, .. } => {
            eprintln!("[warning] {message}");
            Ok(false)
        }
        UiThreadEvent::Error { message, .. } => {
            eprintln!("[error] {message}");
            Ok(false)
        }
        UiThreadEvent::Done { summary, stats, .. } => {
            if let Some(summary) = summary {
                eprintln!("\n[done] summary: {summary}");
            }
            eprintln!(
                "[done] stats: tools={} input_tokens={} output_tokens={}",
                stats.tool_calls_count, stats.total_input_tokens, stats.total_output_tokens
            );
            Ok(false)
        }
    }
}

fn print_json<T: Serialize>(stream: &str, event: &T) -> Result<()> {
    let line = serde_json::to_string(&serde_json::json!({
        "stream": stream,
        "event": event,
    }))
    .context("failed to serialize event")?;
    println!("{line}");
    Ok(())
}
