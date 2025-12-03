pub mod mcp {
    use clap::Subcommand;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Subcommand)]
    pub enum McpCommand {
        /// Add an MCP server
        #[command(arg_required_else_help = true)]
        Add {
            /// Name for the MCP server
            name: String,
            /// Transport type (stdio, http, sse, streamable_http)
            #[arg(short = 't', long = "transport", default_value = "stdio")]
            transport: String,
            /// Command to run for stdio transport
            #[arg(long = "command")]
            command: Option<String>,
            /// Arguments for the command
            #[arg(long = "args")]
            args: Option<String>,
        },
        /// Remove an MCP server
        #[command(arg_required_else_help = true)]
        Remove {
            /// Name of the MCP server to remove
            name: String,
        },
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct MCPServerConfig {
        pub name: String,
        pub transport: TransportConfig,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub command: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub env: Option<HashMap<String, String>>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct TransportConfig {
        #[serde(rename = "type")]
        pub transport_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub command: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub args: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub env: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub headers: Option<HashMap<String, String>>,
    }
}

// Export the functions that are used in main.rs
pub use crate::commands::mcp_functions::{add_mcp_server, remove_mcp_server};

mod mcp_functions {
    use crate::commands::mcp::MCPServerConfig;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ProjectSettings {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub mcp_servers: Option<HashMap<String, MCPServerConfig>>,
    }

    pub fn add_mcp_server(name: String, config: MCPServerConfig) -> Result<(), Box<dyn std::error::Error>> {
        // In a complete implementation, this would save the server config to project settings
        println!("Added MCP server: {}", name);
        Ok(())
    }

    pub fn remove_mcp_server(name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // In a complete implementation, this would remove the server config from project settings
        println!("Removed MCP server: {}", name);
        Ok(())
    }
}