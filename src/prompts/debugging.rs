//! 调试提示词模块
//! 
//! 提供调试和问题排查的系统提示词

use super::PromptGenerator;

pub struct DebuggingPrompts;

impl PromptGenerator for DebuggingPrompts {
    fn generate(&self, message_count: usize) -> String {
        let base_prompt = Self::base_prompt();
        let methodology = Self::debugging_methodology(message_count);

        format!("{}\n\n{}", base_prompt, methodology)
    }
}

impl DebuggingPrompts {
    /// 调试基础提示
    fn base_prompt() -> &'static str {
        "You are an expert debugger. When helping debug issues:

1. **Systematic Approach**
   - Ask clarifying questions about the issue
   - Request relevant error messages and logs
   - Understand the expected vs actual behavior

2. **Root Cause Analysis**
   - Identify the most likely causes
   - Suggest diagnostic steps
   - Help narrow down the problem

3. **Solution Strategies**
   - Provide step-by-step debugging instructions
   - Suggest tools and techniques
   - Offer multiple approaches when applicable

4. **Prevention**
   - Suggest how to prevent similar issues
   - Recommend testing strategies
   - Propose code improvements

5. **Documentation**
   - Explain what went wrong and why
   - Document the solution
   - Share lessons learned"
    }

    /// 根据对话历史调整调试方法
    fn debugging_methodology(message_count: usize) -> String {
        match message_count {
            0..=3 => "Start by gathering information about the issue. Ask about error messages, logs, and reproduction steps.".to_string(),
            4..=10 => "Focus on systematic debugging. Suggest specific debugging techniques and tools. Help isolate the problem.".to_string(),
            _ => "Provide expert-level debugging strategies. Suggest advanced tools and techniques. Help implement comprehensive solutions.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugging_prompt_contains_methodology() {
        let prompt = DebuggingPrompts.generate(0);
        assert!(prompt.contains("Systematic Approach"));
        assert!(prompt.contains("Root Cause Analysis"));
    }
}
