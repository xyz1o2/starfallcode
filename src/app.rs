use crate::ai::{
    client::LLMClient, 
    commands::{CommandParser, CommandType},
    config::LLMConfig,
    streaming::{StreamHandler, StreamingChatResponse},
};
use crate::ui::CommandHints;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct App {
    pub llm_client: Option<LLMClient>,
    pub llm_config: Option<LLMConfig>,
    pub project_context: Option<crate::ai::context::ProjectContext>,
    pub chat_input: String,
    pub chat_history: Vec<ChatMessage>,
    pub stream_handler: Arc<Mutex<Option<StreamHandler>>>,
    pub streaming_response: Arc<Mutex<StreamingChatResponse>>,
    pub is_streaming: bool,
    pub command_hints: CommandHints,
}


impl App {
    pub fn new() -> Self {
        Self {
            llm_client: None,
            llm_config: None,
            project_context: None,
            chat_input: String::new(),
            chat_history: Vec::new(),
            stream_handler: Arc::new(Mutex::new(None)),
            streaming_response: Arc::new(Mutex::new(StreamingChatResponse::new())),
            is_streaming: false,
            command_hints: CommandHints::new(),
        }
    }

    pub fn init_ai_client(&mut self, api_key: String) {
        let config = crate::ai::client::LLMConfig {
            api_key,
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            temperature: 0.7,
            max_tokens: 200,
        };
        
        self.llm_client = Some(LLMClient::new(config));
    }

    pub fn init_ai_client_with_config(&mut self, config: LLMConfig) {
        let llm_config = crate::ai::client::LLMConfig {
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        };
        
        self.llm_config = Some(config);
        self.llm_client = Some(LLMClient::new(llm_config));
    }

    /// 更新 LLM 客户端配置
    fn update_llm_client(&mut self) {
        if let Some(config) = &self.llm_config {
            let llm_config = crate::ai::client::LLMConfig {
                api_key: config.api_key.clone(),
                model: config.model.clone(),
                base_url: config.base_url.clone(),
                temperature: config.temperature,
                max_tokens: config.max_tokens,
            };
            
            self.llm_client = Some(LLMClient::new(llm_config));
        }
    }


    pub fn handle_chat_input(&mut self, c: char) {
        self.chat_input.push(c);
        // 激活命令提示
        self.command_hints.activate(&self.chat_input);
    }

    pub fn handle_chat_backspace(&mut self) {
        self.chat_input.pop();
        // 更新命令提示
        self.command_hints.activate(&self.chat_input);
    }

    pub fn handle_chat_submit(&mut self) {
        if !self.chat_input.trim().is_empty() {
            let input = self.chat_input.clone();
            
            // 检查是否是命令
            if CommandParser::has_command(&input) {
                self.handle_command(&input);
            } else {
                // 添加用户消息到聊天历史
                self.chat_history.push(ChatMessage {
                    role: "user".to_string(),
                    content: input.clone(),
                });

                // 处理提及
                let mentions = CommandParser::extract_mentions(&input);
                let mut response = String::new();
                
                for mention in mentions {
                    response.push_str(&self.process_mention(&mention));
                    response.push('\n');
                }

                // 如果没有提及，生成 AI 响应
                if response.is_empty() {
                    response = format!("Echo: {}", input);
                }

                // 添加 AI 响应到聊天历史
                self.chat_history.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response,
                });
            }

            // 清空输入和命令提示
            self.chat_input.clear();
            self.command_hints.clear();
        }
    }

    fn handle_command(&mut self, input: &str) {
        if let Some(cmd) = CommandParser::parse_command(input) {
            // 添加用户命令到聊天历史
            self.chat_history.push(ChatMessage {
                role: "user".to_string(),
                content: input.to_string(),
            });

            let response = match cmd.command_type {
                CommandType::Help => CommandParser::get_help(),
                CommandType::Clear => {
                    self.chat_history.clear();
                    "✓ 聊天历史已清除".to_string()
                }
                CommandType::History => {
                    if self.chat_history.is_empty() {
                        "聊天历史为空".to_string()
                    } else {
                        let mut hist = String::from("📜 聊天历史:\n");
                        for (i, msg) in self.chat_history.iter().enumerate() {
                            hist.push_str(&format!("{}. [{}]: {}\n", i + 1, msg.role, msg.content));
                        }
                        hist
                    }
                }
                CommandType::Model => {
                    if let Some(config) = &self.llm_config {
                        if cmd.args.is_empty() {
                            format!("📊 当前模型: {}", config.model)
                        } else {
                            format!("模型设置为: {}", cmd.args.join(" "))
                        }
                    } else {
                        "未配置 LLM".to_string()
                    }
                }
                CommandType::Provider => {
                    if let Some(config) = &self.llm_config {
                        format!("🔌 当前提供商: {}", config.provider.to_string())
                    } else {
                        "未配置 LLM".to_string()
                    }
                }
                CommandType::Temperature => {
                    if let Some(config) = &self.llm_config {
                        format!("🌡️ 当前温度: {}", config.temperature)
                    } else {
                        "未配置 LLM".to_string()
                    }
                }
                CommandType::MaxTokens => {
                    if let Some(config) = &self.llm_config {
                        format!("📝 最大令牌数: {}", config.max_tokens)
                    } else {
                        "未配置 LLM".to_string()
                    }
                }
                CommandType::Status => {
                    if let Some(config) = &self.llm_config {
                        format!("📈 {}", config.get_status_info())
                    } else {
                        "❌ LLM 未配置".to_string()
                    }
                }
                CommandType::ListProviders => {
                    let mut response = String::from("🔌 可用的 AI 提供商:\n\n");
                    for (provider, description) in LLMConfig::list_providers() {
                        response.push_str(&format!("• {}: {}\n", provider.to_string(), description));
                    }
                    response
                }
                CommandType::SetProvider => {
                    if cmd.args.is_empty() {
                        "❌ 请指定提供商名称。例如: /set-provider openai".to_string()
                    } else {
                        let provider_name = &cmd.args[0];
                        let provider = crate::ai::config::LLMProvider::from_string(provider_name);
                        
                        if let Some(config) = &mut self.llm_config {
                            config.set_provider(provider.clone());
                            self.update_llm_client();
                            format!("✓ 提供商已切换到: {}", provider.to_string())
                        } else {
                            "❌ 请先配置 LLM".to_string()
                        }
                    }
                }
                CommandType::SetApiKey => {
                    if cmd.args.is_empty() {
                        "❌ 请提供 API 密钥。例如: /set-api-key your-key-here".to_string()
                    } else {
                        let api_key = cmd.args.join(" ");
                        if let Some(config) = &mut self.llm_config {
                            config.api_key = api_key;
                            self.update_llm_client();
                            "✓ API 密钥已更新".to_string()
                        } else {
                            "❌ 请先配置 LLM".to_string()
                        }
                    }
                }
                CommandType::SetModel => {
                    if cmd.args.is_empty() {
                        "❌ 请指定模型名称。例如: /set-model gpt-4".to_string()
                    } else {
                        let model = cmd.args.join(" ");
                        if let Some(config) = &mut self.llm_config {
                            config.model = model.clone();
                            self.update_llm_client();
                            format!("✓ 模型已设置为: {}", model)
                        } else {
                            "❌ 请先配置 LLM".to_string()
                        }
                    }
                }
                CommandType::SetBaseUrl => {
                    if cmd.args.is_empty() {
                        "❌ 请提供基础 URL。例如: /set-base-url https://api.example.com".to_string()
                    } else {
                        let base_url = cmd.args.join(" ");
                        if let Some(config) = &mut self.llm_config {
                            config.base_url = base_url.clone();
                            self.update_llm_client();
                            format!("✓ 基础 URL 已设置为: {}", base_url)
                        } else {
                            "❌ 请先配置 LLM".to_string()
                        }
                    }
                }
                CommandType::ConfigOpenAI => {
                    if cmd.args.is_empty() {
                        "❌ 请提供 API 密钥。例如: /openai sk-your-key gpt-4".to_string()
                    } else {
                        let api_key = cmd.args[0].clone();
                        let model = cmd.args.get(1).cloned();
                        
                        if let Some(config) = &mut self.llm_config {
                            config.quick_config_openai(api_key, model.clone());
                        } else {
                            let mut new_config = LLMConfig::default_openai(api_key);
                            if let Some(m) = model.clone() {
                                new_config.model = m;
                            }
                            self.llm_config = Some(new_config);
                        }
                        self.update_llm_client();
                        format!("✓ OpenAI 配置完成 - 模型: {}", 
                               model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()))
                    }
                }
                CommandType::ConfigClaude => {
                    if cmd.args.is_empty() {
                        "❌ 请提供 API 密钥。例如: /claude your-key claude-3-opus".to_string()
                    } else {
                        let api_key = cmd.args[0].clone();
                        let model = cmd.args.get(1).cloned();
                        
                        if let Some(config) = &mut self.llm_config {
                            config.quick_config_claude(api_key, model.clone());
                        } else {
                            let mut new_config = LLMConfig::default_openai(api_key); // 临时使用，会被覆盖
                            new_config.quick_config_claude(new_config.api_key.clone(), model.clone());
                            self.llm_config = Some(new_config);
                        }
                        self.update_llm_client();
                        format!("✓ Claude 配置完成 - 模型: {}", 
                               model.unwrap_or_else(|| "claude-3-sonnet".to_string()))
                    }
                }
                CommandType::ConfigGemini => {
                    if cmd.args.is_empty() {
                        "❌ 请提供 API 密钥。例如: /gemini your-key gemini-pro".to_string()
                    } else {
                        let api_key = cmd.args[0].clone();
                        let model = cmd.args.get(1).cloned();
                        
                        if let Some(config) = &mut self.llm_config {
                            config.quick_config_gemini(api_key, model.clone());
                        } else {
                            let mut new_config = LLMConfig::default_gemini(api_key);
                            if let Some(m) = model.clone() {
                                new_config.model = m;
                            }
                            self.llm_config = Some(new_config);
                        }
                        self.update_llm_client();
                        format!("✓ Gemini 配置完成 - 模型: {}", 
                               model.unwrap_or_else(|| "gemini-1.5-flash".to_string()))
                    }
                }
                CommandType::ConfigOllama => {
                    let model = cmd.args.get(0).cloned();
                    let base_url = cmd.args.get(1).cloned();
                    
                    if let Some(config) = &mut self.llm_config {
                        config.quick_config_ollama(model.clone(), base_url.clone());
                    } else {
                        let mut new_config = LLMConfig::default_ollama();
                        if let Some(m) = model.clone() {
                            new_config.model = m;
                        }
                        if let Some(url) = base_url.clone() {
                            new_config.base_url = url;
                        }
                        self.llm_config = Some(new_config);
                    }
                    self.update_llm_client();
                    format!("✓ Ollama 配置完成 - 模型: {}", 
                           model.unwrap_or_else(|| "mistral".to_string()))
                }
                CommandType::ConfigLocal => {
                    if cmd.args.is_empty() {
                        "❌ 请提供服务器 URL。例如: /local http://localhost:1234/v1/chat/completions".to_string()
                    } else {
                        let base_url = cmd.args[0].clone();
                        let model = cmd.args.get(1).cloned();
                        
                        if let Some(config) = &mut self.llm_config {
                            config.quick_config_local(base_url.clone(), model.clone());
                        } else {
                            let new_config = LLMConfig::default_local_server(base_url.clone());
                            self.llm_config = Some(new_config);
                        }
                        self.update_llm_client();
                        format!("✓ 本地服务器配置完成 - URL: {}", base_url)
                    }
                }
                CommandType::SaveConfig => {
                    if let Some(config) = &self.llm_config {
                        match config.save_to_env() {
                            Ok(_) => "✓ 配置已保存到 .env 文件".to_string(),
                            Err(e) => format!("❌ 保存配置失败: {}", e),
                        }
                    } else {
                        "❌ 没有配置可保存".to_string()
                    }
                }
                CommandType::LoadConfig => {
                    match LLMConfig::from_env() {
                        Ok(config) => {
                            self.llm_config = Some(config);
                            self.update_llm_client();
                            "✓ 配置已从 .env 文件重新加载".to_string()
                        }
                        Err(e) => format!("❌ 加载配置失败: {}", e),
                    }
                }
                CommandType::Unknown => "❌ 未知命令。输入 /help 获取帮助".to_string(),
            };

            // 添加命令响应到聊天历史
            self.chat_history.push(ChatMessage {
                role: "system".to_string(),
                content: response,
            });
        }
    }

    fn process_mention(&self, mention: &crate::ai::commands::Mention) -> String {
        use crate::ai::commands::MentionType;
        
        match mention.mention_type {
            MentionType::Model => {
                if let Some(config) = &self.llm_config {
                    format!("📊 [模型: {}]", config.model)
                } else {
                    "[模型: 未配置]".to_string()
                }
            }
            MentionType::Provider => {
                if let Some(config) = &self.llm_config {
                    format!("🔌 [提供商: {}]", config.provider.to_string())
                } else {
                    "[提供商: 未配置]".to_string()
                }
            }
            MentionType::History => {
                format!("📜 [聊天历史: {} 条消息]", self.chat_history.len())
            }
            MentionType::File => {
                format!("📄 [文件: {}]", mention.target)
            }
            MentionType::Unknown => {
                format!("[未知提及: {}]", mention.target)
            }
        }
    }

    /// 启动流式聊天
    pub async fn start_streaming_chat(&mut self, prompt: &str) {
        if let Some(ref client) = self.llm_client {
            self.is_streaming = true;
            let handler = StreamHandler::new();
            *self.stream_handler.lock().await = Some(handler.clone());
            
            // 添加用户消息
            self.chat_history.push(ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            });

            let client = client.clone();
            let prompt = prompt.to_string();
            let handler = handler.clone();
            let streaming_response = Arc::clone(&self.streaming_response);

            // 在后台任务中处理流式响应
            tokio::spawn(async move {
                let callback = |token: String| {
                    let _ = handler.send_token(token.clone());
                    true
                };

                match client.generate_completion_stream(&prompt, callback).await {
                    Ok(_) => {
                        let _ = handler.send_done();
                        let mut resp = streaming_response.lock().await;
                        resp.mark_complete();
                    }
                    Err(e) => {
                        let _ = handler.send_error(e.to_string());
                    }
                }
            });
        }
    }

    /// 获取流式响应内容
    pub async fn get_streaming_content(&self) -> String {
        self.streaming_response.lock().await.get_content().to_string()
    }

    /// 完成流式响应并添加到历史
    pub async fn finalize_streaming_response(&mut self) {
        let response = self.streaming_response.lock().await;
        if !response.get_content().is_empty() {
            self.chat_history.push(ChatMessage {
                role: "assistant".to_string(),
                content: response.get_content().to_string(),
            });
        }
        drop(response);
        
        // 重置流式响应
        self.streaming_response.lock().await.reset();
        self.is_streaming = false;
    }

}