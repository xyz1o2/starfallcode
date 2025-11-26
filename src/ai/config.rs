use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LLMProvider {
    OpenAI,
    Gemini,
    Claude,
    Ollama,
    LocalServer,
}

impl LLMProvider {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gemini" => LLMProvider::Gemini,
            "claude" => LLMProvider::Claude,
            "ollama" => LLMProvider::Ollama,
            "local" | "localserver" => LLMProvider::LocalServer,
            _ => LLMProvider::OpenAI,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            LLMProvider::OpenAI => "openai".to_string(),
            LLMProvider::Gemini => "gemini".to_string(),
            LLMProvider::Claude => "claude".to_string(),
            LLMProvider::Ollama => "ollama".to_string(),
            LLMProvider::LocalServer => "local_server".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl LLMConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        // Load .env file if it exists
        let _ = dotenv::dotenv();

        let provider_str = env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());
        let provider = LLMProvider::from_string(&provider_str);

        let (api_key, model, base_url) = match provider {
            LLMProvider::OpenAI => (
                env::var("OPENAI_API_KEY")?,
                env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
                env::var("OPENAI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string()),
            ),
            LLMProvider::Gemini => (
                env::var("GEMINI_API_KEY")?,
                env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".to_string()),
                env::var("GEMINI_BASE_URL").unwrap_or_else(|_| {
                    "https://generativelanguage.googleapis.com/v1beta/openai/".to_string()
                }),
            ),
            LLMProvider::Claude => (
                env::var("ANTHROPIC_API_KEY")?,
                env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-3-sonnet".to_string()),
                env::var("ANTHROPIC_BASE_URL")
                    .unwrap_or_else(|_| "https://api.anthropic.com/v1/messages".to_string()),
            ),
            LLMProvider::Ollama => (
                "local".to_string(),
                env::var("OLLAMA_MODEL").unwrap_or_else(|_| "mistral".to_string()),
                env::var("OLLAMA_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:11434/api/chat".to_string()),
            ),
            LLMProvider::LocalServer => (
                "local".to_string(),
                env::var("LOCAL_MODEL").unwrap_or_else(|_| "liquid/lfm2-1.2b".to_string()),
                env::var("LOCAL_SERVER_URL")
                    .unwrap_or_else(|_| "http://172.22.32.1:1234/v1/chat/completions".to_string()),
            ),
        };

        let temperature = env::var("LLM_TEMPERATURE")
            .unwrap_or_else(|_| "0.7".to_string())
            .parse()
            .unwrap_or(0.7);

        let max_tokens = env::var("LLM_MAX_TOKENS")
            .unwrap_or_else(|_| "200".to_string())
            .parse()
            .unwrap_or(200);

        Ok(LLMConfig {
            provider,
            api_key,
            model,
            base_url,
            temperature,
            max_tokens,
        })
    }

    /// Create a default OpenAI configuration
    pub fn default_openai(api_key: String) -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            api_key,
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            temperature: 0.7,
            max_tokens: 200,
        }
    }

    /// Create a default Gemini configuration
    pub fn default_gemini(api_key: String) -> Self {
        Self {
            provider: LLMProvider::Gemini,
            api_key,
            model: "gemini-1.5-flash".to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai/".to_string(),
            temperature: 0.7,
            max_tokens: 200,
        }
    }

    /// Create a default Ollama configuration (local)
    pub fn default_ollama() -> Self {
        Self {
            provider: LLMProvider::Ollama,
            api_key: "local".to_string(),
            model: "mistral".to_string(),
            base_url: "http://localhost:11434/api/chat".to_string(),
            temperature: 0.7,
            max_tokens: 200,
        }
    }

    /// Create a local server configuration
    pub fn default_local_server(base_url: String) -> Self {
        Self {
            provider: LLMProvider::LocalServer,
            api_key: "local".to_string(),
            model: "liquid/lfm2-1.2b".to_string(),
            base_url,
            temperature: 0.7,
            max_tokens: 200,
        }
    }
}
