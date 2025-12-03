use crate::mcp::{MCPServerConfig, TransportConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, MCPServerConfig>>,
}

// Predefined servers that can be used as reference
pub const PREDEFINED_SERVERS: &[(&str, &str)] = &[
    ("linear", "Linear integration server"),
    ("github", "GitHub integration server"),
    // Add more predefined servers as needed
];

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

pub fn load_mcp_config() -> crate::mcp::MCPConfig {
    crate::mcp::load_mcp_config()
}