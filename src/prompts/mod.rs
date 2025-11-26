//! 提示词管理模块
//! 
//! 本模块管理所有 AI 配对编程的系统提示词，按功能分类存储

pub mod pair_programming;
pub mod code_review;
pub mod debugging;

pub use pair_programming::PairProgrammingPrompts;
pub use code_review::CodeReviewPrompts;
pub use debugging::DebuggingPrompts;

/// 提示词生成器特征
pub trait PromptGenerator {
    /// 根据对话历史长度生成适应性提示词
    fn generate(&self, message_count: usize) -> String;
}

/// 获取配对编程提示词
pub fn get_pair_programming_prompt(message_count: usize) -> String {
    PairProgrammingPrompts.generate(message_count)
}

/// 获取代码审查提示词
pub fn get_code_review_prompt(message_count: usize) -> String {
    CodeReviewPrompts.generate(message_count)
}

/// 获取调试提示词
pub fn get_debugging_prompt(message_count: usize) -> String {
    DebuggingPrompts.generate(message_count)
}
