# Grok CLI

A conversational AI CLI tool powered by Grok with intelligent text editor capabilities and tool usage.

## Features

- **🤖 Conversational AI**: Natural language interface powered by Grok
- **📝 Smart File Operations**: AI automatically uses tools to view, create, and edit files
- **⚡ Bash Integration**: Execute shell commands through natural conversation
- **🔧 Automatic Tool Selection**: AI intelligently chooses the right tools for your requests
- **🔌 MCP Tools**: Extend capabilities with Model Context Protocol servers (Linear, GitHub, etc.)
- **💬 Interactive UI**: Beautiful terminal interface built with Ratatui

## Installation

First, make sure you have Rust and Cargo installed on your system. Then:

```bash
git clone <repository-url>
cd grok-cli
cargo build --release
```

## Usage

### Basic Usage
```bash
# Interactive mode
cargo run

# With initial message
cargo run "What files are in this directory?"

# Headless mode - process a single prompt
cargo run -- --prompt "Summarize the main components in src/"

# With API key
cargo run -- --api-key your_api_key_here "Write a simple Rust program"
```

### Configuration

You can set your API key in several ways:

1. Environment variable:
```bash
export GROK_API_KEY=your_api_key_here
cargo run
```

2. Command line:
```bash
cargo run -- --api-key your_api_key_here "Your prompt"
```

3. Configuration file: The application will look for settings in `~/.grok/user-settings.json`

### MCP (Model Context Protocol)

Manage MCP servers with the built-in commands:

```bash
# Add an MCP server
cargo run -- mcp add my-server --transport stdio --command "my-mcp-server"

# Remove an MCP server
cargo run -- mcp remove my-server
```

## Commands

- `/help` - Show help information
- `/model <model-name>` - Switch to a different AI model
- `/settings` - Show current settings

## Environment Variables

- `GROK_API_KEY` - Your Grok API key
- `GROK_BASE_URL` - API base URL (default: https://api.x.ai/v1)
- `GROK_MODEL` - Default model to use
- `GROK_MAX_TOKENS` - Maximum tokens for responses (default: 1536)

## Project Structure

```
grok-cli/
├── src/
│   ├── agent/          # Main agent logic
│   ├── grok/           # Grok API client
│   ├── tools/          # Tool implementations (text editor, bash, etc.)
│   ├── types/          # Type definitions
│   ├── ui/             # Terminal UI components
│   ├── utils/          # Utility functions
│   ├── mcp/            # Model Context Protocol
│   ├── commands/       # CLI command definitions
│   └── main.rs         # Entry point
├── Cargo.toml          # Dependencies and build configuration
└── README.md
```

## Configuration Files

The application uses the following configuration files:

- `~/.grok/user-settings.json` - User-level settings (API key, default model, etc.)
- `./.grok/settings.json` - Project-level settings (model, MCP servers)

## Development

To run in development mode:

```bash
cargo run
```

To run tests:

```bash
cargo test
```

To format the code:

```bash
cargo fmt
```

To check for issues:

```bash
cargo clippy
```

## License

MIT