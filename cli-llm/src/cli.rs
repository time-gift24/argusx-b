use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "cli-llm")]
#[command(about = "CLI for LLM providers")]
pub struct Cli {
    #[command(subcommand)]
    pub provider: Option<ProviderCommand>,
}

#[derive(Subcommand)]
pub enum ProviderCommand {
    /// List all available providers
    List,
    /// Use BigModel provider
    Bigmodel(BigmodelOpts),
    /// Use OpenAI provider (TODO)
    Openai(OpenaiOpts),
    /// Use Anthropic provider (TODO)
    Anthropic(AnthropicOpts),
}

#[derive(Parser)]
pub struct CommonOpts {
    /// API key (or use environment variable BIGMODEL_API_KEY)
    #[arg(long)]
    pub api_key: Option<String>,
    /// Input mode
    #[arg(short, long)]
    pub interactive: bool,
    /// Input from file
    #[arg(short, long)]
    pub file: Option<String>,
    /// Output format: json, text, markdown
    #[arg(short, long, default_value = "text")]
    pub output: OutputFormat,
    /// Enable streaming (default: true)
    #[arg(long, default_value = "true")]
    pub stream: bool,
    /// Disable streaming
    #[arg(long)]
    pub no_stream: bool,
}

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Markdown,
}

#[derive(Parser)]
pub struct BigmodelOpts {
    #[arg(long, default_value = "glm-4")]
    pub model: String,
    #[command(flatten)]
    pub common: CommonOpts,
}

#[derive(Parser)]
pub struct OpenaiOpts {
    #[arg(long, default_value = "gpt-4o")]
    pub model: String,
    #[command(flatten)]
    pub common: CommonOpts,
}

#[derive(Parser)]
pub struct AnthropicOpts {
    #[arg(long, default_value = "claude-3-5-sonnet-20241022")]
    pub model: String,
    #[command(flatten)]
    pub common: CommonOpts,
}
