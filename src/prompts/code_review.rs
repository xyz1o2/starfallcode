//! 代码审查提示词模块
//! 
//! 提供代码审查和优化的系统提示词

use super::PromptGenerator;

pub struct CodeReviewPrompts;

impl PromptGenerator for CodeReviewPrompts {
    fn generate(&self, message_count: usize) -> String {
        let base_prompt = Self::base_prompt();
        let focus_areas = Self::focus_areas(message_count);

        format!("{}\n\n{}", base_prompt, focus_areas)
    }
}

impl CodeReviewPrompts {
    /// 代码审查基础提示
    fn base_prompt() -> &'static str {
        "You are an expert code reviewer. When reviewing code, focus on:

1. **Correctness**
   - Identify logical errors and edge cases
   - Check for potential runtime errors
   - Verify algorithm correctness

2. **Performance**
   - Identify performance bottlenecks
   - Suggest algorithmic improvements
   - Recommend caching or optimization strategies

3. **Maintainability**
   - Check code clarity and readability
   - Verify naming conventions
   - Assess code organization and structure

4. **Security**
   - Identify security vulnerabilities
   - Check input validation
   - Review authentication and authorization

5. **Testing**
   - Suggest test coverage improvements
   - Recommend test strategies
   - Identify untested edge cases

6. **Best Practices**
   - Apply language-specific conventions
   - Suggest design pattern improvements
   - Recommend refactoring opportunities"
    }

    /// 根据对话历史调整审查重点
    fn focus_areas(message_count: usize) -> String {
        match message_count {
            0..=2 => "Start with a high-level overview of the code structure and main concerns. Then dive into specific issues.".to_string(),
            3..=8 => "Focus on the most impactful issues first. Provide specific, actionable recommendations with code examples.".to_string(),
            _ => "Provide targeted feedback on specific areas. Suggest advanced optimizations and architectural improvements.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_review_prompt_contains_key_areas() {
        let prompt = CodeReviewPrompts.generate(0);
        assert!(prompt.contains("Correctness"));
        assert!(prompt.contains("Performance"));
        assert!(prompt.contains("Security"));
    }
}
