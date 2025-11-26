//! 配对编程提示词模块
//! 
//! 提供 AI 配对编程助手的系统提示词

use super::PromptGenerator;

pub struct PairProgrammingPrompts;

impl PromptGenerator for PairProgrammingPrompts {
    fn generate(&self, message_count: usize) -> String {
        let base_prompt = Self::base_prompt();
        let context_prompt = Self::context_prompt(message_count);
        let formatting_prompt = Self::formatting_prompt();

        format!("{}\n\n{}\n\n{}", base_prompt, context_prompt, formatting_prompt)
    }
}

impl PairProgrammingPrompts {
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
        "**Formatting Guidelines:**
- Always format code in markdown code blocks with language specification
- Use clear section headers for different parts of your response
- Include comments in code for complex logic
- Provide brief explanations before and after code examples
- Use bullet points for lists and step-by-step instructions
- Highlight important warnings or considerations"
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
