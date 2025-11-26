//! 配对编程提示词模块
//! 
//! 提供 AI 配对编程助手的系统提示词

use super::PromptGenerator;

pub struct PairProgrammingPrompts;

impl PromptGenerator for PairProgrammingPrompts {
    fn generate(&self, message_count: usize) -> String {
        // 核心思想：按优先级注入上下文
        // 1. 加载项目配置文件 (CLAUDE.md / .claude/config.md)
        let project_config = Self::load_project_config();
        
        // 2. 基础身份和角色
        let base_prompt = Self::base_prompt();
        
        // 3. 项目上下文 (自动扫描)
        let project_context = Self::project_context();
        
        // 4. 根据对话历史的适应性提示
        let context_prompt = Self::context_prompt(message_count);
        
        // 5. 格式化和文件操作指南
        let formatting_prompt = Self::formatting_prompt();

        // 按优先级组合
        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}",
            project_config,
            base_prompt,
            project_context,
            context_prompt,
            formatting_prompt
        )
    }
}

impl PairProgrammingPrompts {
    /// 加载项目配置文件 (CLAUDE.md / .claude/config.md)
    /// 这是 Claude CLI / Gemini CLI 的核心思想
    fn load_project_config() -> String {
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".to_string());
        
        // 按优先级查找配置文件
        let config_paths = vec![
            format!("{}/.claude/config.md", cwd),
            format!("{}/CLAUDE.md", cwd),
            format!("{}/GEMINI.md", cwd),
            format!("{}/AI.md", cwd),
        ];
        
        for path in config_paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if !content.trim().is_empty() {
                    return format!(
                        "**Project Configuration (from {}):**\n\n{}",
                        path, content
                    );
                }
            }
        }
        
        // 如果没有找到配置文件，返回空字符串
        String::new()
    }
    
    /// 项目上下文信息 - 动态扫描用户工作目录
    fn project_context() -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let cwd = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        // 扫描目录结构（最多3层深度）
        let dir_tree = Self::scan_directory_tree(&cwd, 3);
        
        // 检测项目类型
        let project_type = Self::detect_project_type(&cwd);
        
        format!(
            r#"**Project Context:**
- **Operating System:** {} ({})
- **Current Working Directory:** {}
- **Project Type:** {}
**Directory Structure:**
```
{}
```
**When Creating/Modifying Files - IMPORTANT:**
ALWAYS announce your intention BEFORE providing code:

1. First, state your intention clearly:
   - "I will create a file at `src/main.rs`"
   - "I will modify `config.json`"
   - "I will delete `old_file.txt`"

2. Then provide the file operation instruction:
   - `create file \`path/to/file.ext\``
   - `modify \`path/to/file.ext\``
   - `delete \`path/to/file.ext\``

3. Finally, provide the code block:
   ```language
   code content here
   ```

Example:
"I will create a new HTML file at `src/index.html` with a simple Hello World page.

create file \`src/index.html\`

\`\`\`html
<!DOCTYPE html>
<html>
<head><title>Hello</title></head>
<body><h1>Hello, World!</h1></body>
</html>
\`\`\`"

The system will automatically detect these instructions and show a confirmation dialog BEFORE executing file operations. Wait for user confirmation before proceeding."#,
            os, arch, cwd, project_type, dir_tree
        )
    }
    
    /// 扫描目录树结构
    fn scan_directory_tree(path: &str, max_depth: usize) -> String {
        Self::scan_dir_recursive(path, 0, max_depth, "")
    }
    
    /// 递归扫描目录
    fn scan_dir_recursive(path: &str, current_depth: usize, max_depth: usize, prefix: &str) -> String {
        if current_depth > max_depth {
            return String::new();
        }
        
        let mut result = String::new();
        
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut items: Vec<_> = entries
                .filter_map(|e| e.ok())
                .collect();
            items.sort_by_key(|a| {
                (
                    !a.path().is_dir(),
                    a.file_name().to_string_lossy().to_string(),
                )
            });
            
            for (idx, entry) in items.iter().enumerate() {
                let is_last = idx == items.len() - 1;
                let path = entry.path();
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                
                // 跳过隐藏文件和常见的忽略目录
                if file_name_str.starts_with('.') 
                    || file_name_str == "node_modules"
                    || file_name_str == "target"
                    || file_name_str == ".git"
                    || file_name_str == "__pycache__"
                {
                    continue;
                }
                
                let connector = if is_last { "└── " } else { "├── " };
                let next_prefix = if is_last { "    " } else { "│   " };
                
                result.push_str(&format!("{}{}{}\n", prefix, connector, file_name_str));
                
                if path.is_dir() && current_depth < max_depth {
                    let sub_tree = Self::scan_dir_recursive(
                        path.to_str().unwrap_or(""),
                        current_depth + 1,
                        max_depth,
                        &format!("{}{}", prefix, next_prefix),
                    );
                    result.push_str(&sub_tree);
                }
            }
        }
        
        result
    }
    
    /// 检测项目类型
    fn detect_project_type(path: &str) -> String {
        let mut project_types = Vec::new();
        
        // 检查 Cargo.toml
        if std::path::Path::new(&format!("{}/Cargo.toml", path)).exists() {
            project_types.push("Rust (Cargo)");
        }
        
        // 检查 package.json
        if std::path::Path::new(&format!("{}/package.json", path)).exists() {
            project_types.push("Node.js/JavaScript");
        }
        
        // 检查 pyproject.toml 或 requirements.txt
        if std::path::Path::new(&format!("{}/pyproject.toml", path)).exists()
            || std::path::Path::new(&format!("{}/requirements.txt", path)).exists()
        {
            project_types.push("Python");
        }
        
        // 检查 go.mod
        if std::path::Path::new(&format!("{}/go.mod", path)).exists() {
            project_types.push("Go");
        }
        
        // 检查 pom.xml
        if std::path::Path::new(&format!("{}/pom.xml", path)).exists() {
            project_types.push("Java (Maven)");
        }
        
        if project_types.is_empty() {
            "Unknown/Generic Project".to_string()
        } else {
            project_types.join(", ")
        }
    }

    /// 基础系统提示
    fn base_prompt() -> &'static str {
        "You are an expert AI pair programming assistant. Your role is to:

1. **Provide detailed, actionable code suggestions**
   - Write clean, maintainable, and efficient code
   - Follow language-specific best practices and conventions
   - Consider performance implications

2. **Explain the 'why' behind your recommendations**
   - Help the user understand design decisions
   - Explain trade-offs and alternatives
   - Teach programming concepts when relevant

3. **Consider context from the conversation history**
   - Reference previous discussion points
   - Build on established patterns
   - Maintain consistency with existing code

4. **Suggest best practices and design patterns**
   - Apply SOLID principles
   - Use appropriate design patterns
   - Recommend architectural improvements

5. **Help with debugging and optimization**
   - Identify potential bugs and issues
   - Suggest performance improvements
   - Provide testing strategies

6. **Provide examples when explaining concepts**
   - Show practical code examples
   - Include edge cases and error handling
   - Demonstrate best practices"
    }

    /// 根据对话历史长度的上下文提示
    fn context_prompt(message_count: usize) -> String {
        match message_count {
            0 => "This is the start of a new session. Be welcoming and ask clarifying questions about the user's goals, tech stack, and project context.".to_string(),
            1..=4 => "You're building context with the user. Be thorough and educational in your responses. Ask follow-up questions to better understand their needs.".to_string(),
            5..=10 => "You have good context now. Provide concise but comprehensive responses. Reference previous discussion when relevant. Start suggesting optimizations.".to_string(),
            _ => "You have extensive context. Provide focused, expert-level responses. Anticipate needs based on conversation history. Suggest advanced optimizations and refactoring.".to_string(),
        }
    }

    /// 格式化和输出指南
    fn formatting_prompt() -> &'static str {
        r#"**Formatting Guidelines:**
- Always format code in markdown code blocks with language specification
- Use clear section headers for different parts of your response
- Include comments in code for complex logic
- Provide brief explanations before and after code examples
- Use bullet points for lists and step-by-step instructions
- Highlight important warnings or considerations

**File Creation Instructions:**
When you want to create or modify files, use the following format:

For creating a new file:
create file `path/to/file.ext`

Then provide the code in a markdown code block:
```language
code content here
```

For modifying an existing file:
modify `path/to/file.ext`

Then provide the new code in a markdown code block.

Examples:
- create file `src/main.rs`
- create file `index.html`
- modify `src/app.rs`
- modify `package.json`

This format allows the system to automatically detect and confirm file operations with the user before execution."#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_session_prompt() {
        let prompt = PairProgrammingPrompts.generate(0);
        assert!(prompt.contains("welcoming"));
        assert!(prompt.contains("clarifying questions"));
    }

    #[test]
    fn test_experienced_session_prompt() {
        let prompt = PairProgrammingPrompts.generate(20);
        assert!(prompt.contains("expert-level"));
        assert!(prompt.contains("Anticipate needs"));
    }

    #[test]
    fn test_prompt_contains_base_elements() {
        let prompt = PairProgrammingPrompts.generate(5);
        assert!(prompt.contains("pair programming assistant"));
        assert!(prompt.contains("code blocks"));
    }
}
