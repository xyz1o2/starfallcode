use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LLMProvider {
    OpenAI,
    Gemini,
    Claude,
    Ollama,
    DeepSeek,
    LocalServer,
}

impl LLMProvider {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gemini" => LLMProvider::Gemini,
            "claude" => LLMProvider::Claude,
            "ollama" => LLMProvider::Ollama,
            "deepseek" => LLMProvider::DeepSeek,
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
            LLMProvider::DeepSeek => "deepseek".to_string(),
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
            LLMProvider::DeepSeek => (
                env::var("DEEPSEEK_API_KEY")?,
                env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".to_string()),
                env::var("DEEPSEEK_BASE_URL")
                    .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string()),
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

    /// Create a default DeepSeek configuration
    pub fn default_deepseek(api_key: String) -> Self {
        Self {
            provider: LLMProvider::DeepSeek,
            api_key,
            model: "deepseek-chat".to_string(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
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

    /// 动态设置提供商
    pub fn set_provider(&mut self, provider: LLMProvider) {
        self.provider = provider.clone();
        // 根据提供商设置默认值
        match provider {
            LLMProvider::OpenAI => {
                if self.base_url.is_empty() {
                    self.base_url = "https://api.openai.com/v1/chat/completions".to_string();
                }
                if self.model.is_empty() {
                    self.model = "gpt-3.5-turbo".to_string();
                }
            }
            LLMProvider::Gemini => {
                if self.base_url.is_empty() {
                    self.base_url = "https://generativelanguage.googleapis.com/v1beta/openai/".to_string();
                }
                if self.model.is_empty() {
                    self.model = "gemini-1.5-flash".to_string();
                }
            }
            LLMProvider::Claude => {
                if self.base_url.is_empty() {
                    self.base_url = "https://api.anthropic.com/v1/messages".to_string();
                }
                if self.model.is_empty() {
                    self.model = "claude-3-sonnet".to_string();
                }
            }
            LLMProvider::Ollama => {
                self.api_key = "local".to_string();
                if self.base_url.is_empty() {
                    self.base_url = "http://localhost:11434/api/chat".to_string();
                }
                if self.model.is_empty() {
                    self.model = "mistral".to_string();
                }
            }
            LLMProvider::DeepSeek => {
                if self.base_url.is_empty() {
                    self.base_url = "https://api.deepseek.com/v1".to_string();
                }
                if self.model.is_empty() {
                    self.model = "deepseek-chat".to_string();
                }
            }
            LLMProvider::LocalServer => {
                self.api_key = "local".to_string();
                if self.model.is_empty() {
                    self.model = "liquid/lfm2-1.2b".to_string();
                }
            }
        }
    }

    /// 快速配置 OpenAI
    pub fn quick_config_openai(&mut self, api_key: String, model: Option<String>) {
        self.provider = LLMProvider::OpenAI;
        self.api_key = api_key;
        self.model = model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());
        self.base_url = "https://api.openai.com/v1/chat/completions".to_string();
    }

    /// 快速配置 Claude
    pub fn quick_config_claude(&mut self, api_key: String, model: Option<String>) {
        self.provider = LLMProvider::Claude;
        self.api_key = api_key;
        self.model = model.unwrap_or_else(|| "claude-3-sonnet".to_string());
        self.base_url = "https://api.anthropic.com/v1/messages".to_string();
    }

    /// 快速配置 Gemini
    pub fn quick_config_gemini(&mut self, api_key: String, model: Option<String>) {
        self.provider = LLMProvider::Gemini;
        self.api_key = api_key;
        self.model = model.unwrap_or_else(|| "gemini-1.5-flash".to_string());
        self.base_url = "https://generativelanguage.googleapis.com/v1beta/openai/".to_string();
    }

    /// 快速配置 Ollama
    pub fn quick_config_ollama(&mut self, model: Option<String>, base_url: Option<String>) {
        self.provider = LLMProvider::Ollama;
        self.api_key = "local".to_string();
        self.model = model.unwrap_or_else(|| "mistral".to_string());
        self.base_url = base_url.unwrap_or_else(|| "http://localhost:11434/api/chat".to_string());
    }

    /// 快速配置本地服务器
    pub fn quick_config_local(&mut self, base_url: String, model: Option<String>) {
        self.provider = LLMProvider::LocalServer;
        self.api_key = "local".to_string();
        self.base_url = base_url;
        self.model = model.unwrap_or_else(|| "liquid/lfm2-1.2b".to_string());
    }

    /// 保存配置到 .env 文件
    pub fn save_to_env(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        let env_content = self.generate_env_content();
        fs::write(".env", env_content)?;
        Ok(())
    }

    /// 生成 .env 文件内容
    fn generate_env_content(&self) -> String {
        let mut content = String::new();
        
        content.push_str("# LLM Configuration for Ghost Text Editor\n");
        content.push_str("# Choose your provider: openai, gemini, claude, ollama, local\n\n");

        // 设置当前提供商
        content.push_str(&format!("LLM_PROVIDER={}\n", self.provider.to_string()));

        // OpenAI 配置
        content.push_str("\n# === OpenAI Configuration ===\n");
        if self.provider == LLMProvider::OpenAI {
            content.push_str(&format!("OPENAI_API_KEY={}\n", self.api_key));
            content.push_str(&format!("OPENAI_MODEL={}\n", self.model));
            content.push_str(&format!("OPENAI_BASE_URL={}\n", self.base_url));
        } else {
            content.push_str("# OPENAI_API_KEY=your_openai_api_key_here\n");
            content.push_str("# OPENAI_MODEL=gpt-3.5-turbo\n");
            content.push_str("# OPENAI_BASE_URL=https://api.openai.com/v1/chat/completions\n");
        }

        // Gemini 配置
        content.push_str("\n# === Gemini Configuration ===\n");
        if self.provider == LLMProvider::Gemini {
            content.push_str(&format!("GEMINI_API_KEY={}\n", self.api_key));
            content.push_str(&format!("GEMINI_MODEL={}\n", self.model));
            content.push_str(&format!("GEMINI_BASE_URL={}\n", self.base_url));
        } else {
            content.push_str("# GEMINI_API_KEY=your_gemini_api_key_here\n");
            content.push_str("# GEMINI_MODEL=gemini-1.5-flash\n");
            content.push_str("# GEMINI_BASE_URL=https://generativelanguage.googleapis.com/v1beta/openai/\n");
        }

        // Claude 配置
        content.push_str("\n# === Claude Configuration ===\n");
        if self.provider == LLMProvider::Claude {
            content.push_str(&format!("ANTHROPIC_API_KEY={}\n", self.api_key));
            content.push_str(&format!("CLAUDE_MODEL={}\n", self.model));
            content.push_str(&format!("ANTHROPIC_BASE_URL={}\n", self.base_url));
        } else {
            content.push_str("# ANTHROPIC_API_KEY=your_anthropic_api_key_here\n");
            content.push_str("# CLAUDE_MODEL=claude-3-sonnet\n");
            content.push_str("# ANTHROPIC_BASE_URL=https://api.anthropic.com/v1/messages\n");
        }

        // Ollama 配置
        content.push_str("\n# === Ollama Configuration (Local) ===\n");
        if self.provider == LLMProvider::Ollama {
            content.push_str(&format!("OLLAMA_MODEL={}\n", self.model));
            content.push_str(&format!("OLLAMA_BASE_URL={}\n", self.base_url));
        } else {
            content.push_str("# OLLAMA_MODEL=mistral\n");
            content.push_str("# OLLAMA_BASE_URL=http://localhost:11434/api/chat\n");
        }

        // DeepSeek 配置
        content.push_str("\n# === DeepSeek Configuration ===\n");
        if self.provider == LLMProvider::DeepSeek {
            content.push_str(&format!("DEEPSEEK_API_KEY={}\n", self.api_key));
            content.push_str(&format!("DEEPSEEK_MODEL={}\n", self.model));
            content.push_str(&format!("DEEPSEEK_BASE_URL={}\n", self.base_url));
        } else {
            content.push_str("# DEEPSEEK_API_KEY=your_deepseek_api_key_here\n");
            content.push_str("# DEEPSEEK_MODEL=deepseek-chat\n");
            content.push_str("# DEEPSEEK_BASE_URL=https://api.deepseek.com/v1\n");
        }

        // 本地服务器配置
        content.push_str("\n# === Local Server Configuration ===\n");
        if self.provider == LLMProvider::LocalServer {
            content.push_str(&format!("LOCAL_MODEL={}\n", self.model));
            content.push_str(&format!("LOCAL_SERVER_URL={}\n", self.base_url));
        } else {
            content.push_str("# LOCAL_MODEL=liquid/lfm2-1.2b\n");
            content.push_str("# LOCAL_SERVER_URL=http://172.22.32.1:1234/v1/chat/completions\n");
        }

        // 通用设置
        content.push_str("\n# === General Settings ===\n");
        content.push_str(&format!("LLM_TEMPERATURE={}\n", self.temperature));
        content.push_str(&format!("LLM_MAX_TOKENS={}\n", self.max_tokens));

        content
    }

    /// 获取所有可用的提供商列表
    pub fn list_providers() -> Vec<(LLMProvider, &'static str)> {
        vec![
            (LLMProvider::OpenAI, "OpenAI GPT 模型 (需要 API 密钥)"),
            (LLMProvider::Claude, "Anthropic Claude 模型 (需要 API 密钥)"),
            (LLMProvider::Gemini, "Google Gemini 模型 (需要 API 密钥)"),
            (LLMProvider::DeepSeek, "DeepSeek 模型 (需要 API 密钥)"),
            (LLMProvider::Ollama, "Ollama 本地模型 (需要本地安装)"),
            (LLMProvider::LocalServer, "自定义本地服务器"),
        ]
    }

    /// 获取配置状态信息
    pub fn get_status_info(&self) -> String {
        let api_key_display = if self.api_key == "local" {
            "本地".to_string()
        } else {
            format!("{}...", &self.api_key[..std::cmp::min(8, self.api_key.len())])
        };
        format!(
            "当前配置:\n提供商: {}\n模型: {}\nAPI 密钥: {}\n基础 URL: {}\n温度: {}\n最大令牌: {}",
            self.provider.to_string(),
            self.model,
            api_key_display,
            self.base_url,
            self.temperature,
            self.max_tokens
        )
    }
}
