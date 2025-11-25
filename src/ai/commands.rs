#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    Help,
    Clear,
    History,
    Model,
    Temperature,
    MaxTokens,
    Provider,
    Status,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub command_type: CommandType,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum MentionType {
    Model,      // @model - 提及当前模型
    Provider,   // @provider - 提及当前提供商
    History,    // @history - 提及聊天历史
    File,       // @file - 提及文件
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Mention {
    pub mention_type: MentionType,
    pub target: String,
}

pub struct CommandParser;

impl CommandParser {
    /// 解析命令（以 / 开头）
    pub fn parse_command(input: &str) -> Option<Command> {
        if !input.starts_with('/') {
            return None;
        }

        let trimmed = input[1..].trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        let command_type = match parts[0] {
            "help" | "h" => CommandType::Help,
            "clear" | "c" => CommandType::Clear,
            "history" | "hist" => CommandType::History,
            "model" | "m" => CommandType::Model,
            "temp" | "temperature" => CommandType::Temperature,
            "tokens" | "max_tokens" => CommandType::MaxTokens,
            "provider" | "p" => CommandType::Provider,
            "status" | "s" => CommandType::Status,
            _ => CommandType::Unknown,
        };

        let args = parts[1..]
            .iter()
            .map(|s| s.to_string())
            .collect();

        Some(Command {
            command_type,
            args,
        })
    }

    /// 解析提及（以 @ 开头）
    pub fn parse_mention(input: &str) -> Option<Mention> {
        if !input.starts_with('@') {
            return None;
        }

        let trimmed = input[1..].trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        let mention_type = match parts[0] {
            "model" => MentionType::Model,
            "provider" => MentionType::Provider,
            "history" => MentionType::History,
            "file" => MentionType::File,
            _ => MentionType::Unknown,
        };

        let target = parts[1..]
            .join(" ");

        Some(Mention {
            mention_type,
            target,
        })
    }

    /// 检查输入是否包含命令
    pub fn has_command(input: &str) -> bool {
        input.trim().starts_with('/')
    }

    /// 检查输入是否包含提及
    pub fn has_mention(input: &str) -> bool {
        input.contains('@')
    }

    /// 提取所有提及
    pub fn extract_mentions(input: &str) -> Vec<Mention> {
        let mut mentions = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            if ch == '@' {
                // 找到 @ 符号，提取提及
                chars.next(); // 消费 @

                let mut mention_str = String::from("@");
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                        mention_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if let Some(mention) = Self::parse_mention(&mention_str) {
                    mentions.push(mention);
                }
            } else {
                chars.next();
            }
        }

        mentions
    }

    /// 获取命令帮助文本
    pub fn get_help() -> String {
        r#"
╔════════════════════════════════════════════════════════════════╗
║                    可用命令                                    ║
╠════════════════════════════════════════════════════════════════╣
║ /help, /h              - 显示此帮助信息                        ║
║ /clear, /c             - 清除聊天历史                          ║
║ /history, /hist        - 显示聊天历史                          ║
║ /model, /m [name]      - 显示或设置模型                        ║
║ /temp, /temperature N  - 设置温度参数 (0.0-1.0)               ║
║ /tokens, /max_tokens N - 设置最大令牌数                        ║
║ /provider, /p          - 显示当前 LLM 提供商                   ║
║ /status, /s            - 显示应用状态                          ║
╠════════════════════════════════════════════════════════════════╣
║                    可用提及                                    ║
╠════════════════════════════════════════════════════════════════╣
║ @model                 - 提及当前模型                          ║
║ @provider              - 提及当前提供商                        ║
║ @history               - 提及聊天历史                          ║
║ @file [filename]       - 提及文件内容                          ║
╚════════════════════════════════════════════════════════════════╝
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help_command() {
        let cmd = CommandParser::parse_command("/help");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().command_type, CommandType::Help);
    }

    #[test]
    fn test_parse_model_command_with_args() {
        let cmd = CommandParser::parse_command("/model gpt-4");
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert_eq!(cmd.command_type, CommandType::Model);
        assert_eq!(cmd.args, vec!["gpt-4"]);
    }

    #[test]
    fn test_parse_mention() {
        let mention = CommandParser::parse_mention("@model");
        assert!(mention.is_some());
        assert_eq!(mention.unwrap().mention_type, MentionType::Model);
    }

    #[test]
    fn test_extract_mentions() {
        let mentions = CommandParser::extract_mentions("Hey @model, what about @provider?");
        assert_eq!(mentions.len(), 2);
    }
}
