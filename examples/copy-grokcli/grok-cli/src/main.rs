mod agent;
mod grok;
mod tools;
mod types;
mod utils;
mod mcp;
mod commands;
mod ui;

use clap::{Parser, Subcommand};
use tokio;

#[derive(Subcommand)]
enum Commands {
    /// Manage MCP (Model Context Protocol) servers
    Mcp {
        #[command(subcommand)]
        command: crate::commands::mcp::McpCommand,
    },
}

#[derive(Parser)]
#[command(name = "grok")]
#[command(about = "A conversational AI CLI tool powered by Grok with text editor capabilities")]
struct CliArgs {
    /// Initial message to send to Grok
    #[arg(value_parser)]
    message: Vec<String>,

    /// Set working directory
    #[arg(short = 'd', long = "directory", default_value = ".")]
    directory: String,

    /// Grok API key (or set GROK_API_KEY env var)
    #[arg(short = 'k', long = "api-key")]
    api_key: Option<String>,

    /// Grok API base URL (or set GROK_BASE_URL env var)
    #[arg(short = 'u', long = "base-url")]
    base_url: Option<String>,

    /// AI model to use
    #[arg(short = 'm', long = "model")]
    model: Option<String>,

    /// Process a single prompt and exit (headless mode)
    #[arg(long = "prompt")]
    prompt: Option<String>,

    /// Maximum number of tool execution rounds (default: 400)
    #[arg(long = "max-tool-rounds", default_value = "400")]
    max_tool_rounds: u32,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    let args = CliArgs::parse();

    // Handle subcommands first
    match args.command {
        Some(Commands::Mcp { command }) => {
            handle_mcp_command(command).await?;
            return Ok(());
        }
        None => {
            // Continue with main app logic
        }
    }

    // Change directory if specified
    if args.directory != "." {
        std::env::set_current_dir(&args.directory)?;
    }

    let settings_manager = utils::settings_manager::get_settings_manager().await?;
    let settings = settings_manager.load_user_settings().await?;

    // Get API key from args, environment, or settings
    let api_key_env = args.api_key
        .or_else(|| std::env::var("GROK_API_KEY").ok());

    let api_key = if let Some(key) = api_key_env {
        key
    } else if let Some(key) = settings.api_key {
        key
    } else {
        // No API key provided - if in headless mode, exit with error; if interactive, allow to continue but warn
        if args.prompt.is_some() {
            eprintln!("❌ Error: API key required. Set GROK_API_KEY environment variable, use --api-key flag, or set \"apiKey\" field in ~/.grok/user-settings.json");
            std::process::exit(1);
        } else {
            // No API key but in interactive mode - set to a placeholder to allow UI to start
            "API_KEY_NOT_SET".to_string()
        }
    };

    let base_url = args.base_url
        .or_else(|| std::env::var("GROK_BASE_URL").ok())
        .or(settings.base_url)
        .unwrap_or_else(|| "https://api.x.ai/v1".to_string());

    let model = args.model
        .or_else(|| std::env::var("GROK_MODEL").ok())
        .or(settings.default_model);

    let is_openai_compatible = settings.is_openai_compatible;

    if let Some(prompt) = args.prompt {
        // Headless mode: process prompt and exit
        if api_key == "API_KEY_NOT_SET" {
            eprintln!("❌ Error: API key required for headless mode.");
            std::process::exit(1);
        }

        let mut agent = agent::GrokAgent::new(&api_key, base_url, model, Some(args.max_tool_rounds), is_openai_compatible).await?;

        // Process the prompt
        let chat_entries = agent.process_user_message(&prompt).await?;

        // Output results
        for entry in chat_entries {
            println!("{}", serde_json::to_string(&entry)?);
        }
    } else {
        // Interactive mode: launch UI
        println!("🤖 Starting Grok CLI Conversational Assistant...\n");

        let mut agent = agent::GrokAgent::new(&api_key, base_url, model, Some(args.max_tool_rounds), is_openai_compatible).await?;
        let initial_message = args.message.join(" ");

        ui::run_app(agent, initial_message).await?;
    }

    Ok(())
}

async fn handle_mcp_command(command: crate::commands::mcp::McpCommand) -> Result<(), Box<dyn std::error::Error>> {
    use crate::commands::mcp::{MCPServerConfig, TransportConfig};
    use std::collections::HashMap;

    match command {
        crate::commands::mcp::McpCommand::Add { name, transport, command, args } => {
            // Create the transport configuration
            let transport_config = TransportConfig {
                transport_type: transport,
                command,
                args: args.map(|s| vec![s]), // Simplified - in a real implementation handle multiple args
                env: None,
                url: None,
                headers: None,
            };

            // Create the server configuration
            let server_config = MCPServerConfig {
                name: name.clone(),
                transport: transport_config,
                command: None,
                args: None,
                env: None,
            };

            // Add the server (in a real implementation this would save to config file)
            crate::commands::add_mcp_server(name, server_config)?;
            println!("Added MCP server successfully");
        },
        crate::commands::mcp::McpCommand::Remove { name } => {
            // Remove the server (in a real implementation this would remove from config file)
            crate::commands::remove_mcp_server(&name)?;
            println!("Removed MCP server successfully");
        },
    }
    Ok(())
}