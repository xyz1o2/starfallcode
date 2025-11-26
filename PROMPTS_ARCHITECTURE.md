# 提示词架构文档

## 概述

项目采用模块化的提示词管理系统，将不同类型的 AI 系统提示词分离到独立的模块中，便于维护、扩展和测试。

## 目录结构

```
src/prompts/
├── mod.rs                    # 提示词模块主入口
├── pair_programming.rs       # 配对编程提示词
├── code_review.rs           # 代码审查提示词
└── debugging.rs             # 调试提示词
```

## 模块说明

### 1. `mod.rs` - 主模块

**职责：**
- 定义 `PromptGenerator` 特征
- 导出各个提示词模块
- 提供便捷的公共接口

**关键接口：**
```rust
pub trait PromptGenerator {
    fn generate(&self, message_count: usize) -> String;
}

pub fn get_pair_programming_prompt(message_count: usize) -> String
pub fn get_code_review_prompt(message_count: usize) -> String
pub fn get_debugging_prompt(message_count: usize) -> String
```

### 2. `pair_programming.rs` - 配对编程提示词

**特点：**
- 提供详细的代码建议
- 解释设计决策
- 考虑对话历史
- 建议最佳实践

**适应性提示：**
- **初始阶段（0 条消息）**：欢迎用户，询问项目背景
- **建立阶段（1-4 条消息）**：详细和教育性回复
- **成熟阶段（5-10 条消息）**：简洁但全面的回复
- **专家阶段（10+ 条消息）**：专家级别的建议

### 3. `code_review.rs` - 代码审查提示词

**审查重点：**
- 正确性检查
- 性能优化
- 可维护性评估
- 安全性审查
- 测试覆盖
- 最佳实践建议

### 4. `debugging.rs` - 调试提示词

**调试方法：**
- 系统化方法
- 根本原因分析
- 解决方案策略
- 预防措施
- 文档记录

## 使用示例

### 在 App 中使用

```rust
use crate::prompts;

// 获取配对编程提示词
let prompt = prompts::get_pair_programming_prompt(message_count);

// 或直接使用 PromptGenerator 特征
use crate::prompts::PromptGenerator;
let prompt = PairProgrammingPrompts.generate(message_count);
```

### 添加新的提示词类型

1. 创建新文件 `src/prompts/new_type.rs`
2. 实现 `PromptGenerator` 特征
3. 在 `mod.rs` 中导出

```rust
// src/prompts/new_type.rs
use super::PromptGenerator;

pub struct NewTypePrompts;

impl PromptGenerator for NewTypePrompts {
    fn generate(&self, message_count: usize) -> String {
        // 实现提示词生成逻辑
    }
}
```

## 设计原则

### 1. 单一职责
每个模块负责一种特定类型的提示词

### 2. 可扩展性
通过 `PromptGenerator` 特征实现统一接口

### 3. 适应性
根据对话历史长度动态调整提示词

### 4. 可测试性
每个模块包含单元测试

### 5. 可维护性
清晰的代码结构和文档注释

## 提示词生成流程

```
┌─────────────────────────────────────┐
│ 用户输入消息                         │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ App.start_streaming_chat()          │
│ 获取消息计数                        │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ App.generate_system_prompt()        │
│ 调用 prompts 模块                   │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ PromptGenerator.generate()          │
│ 根据消息计数生成适应性提示          │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│ 返回完整系统提示词                  │
│ 发送给 LLM                          │
└─────────────────────────────────────┘
```

## 最佳实践

### 1. 提示词编写

- 使用清晰的结构和格式
- 包含具体的指导和例子
- 考虑不同的用户经验水平
- 提供可操作的建议

### 2. 适应性设计

- 根据对话历史调整详细程度
- 初期提供更多教育性内容
- 后期提供更专业的建议

### 3. 测试

- 为每个提示词类型编写单元测试
- 验证提示词包含关键元素
- 测试不同的消息计数场景

### 4. 文档

- 为每个模块添加文档注释
- 说明提示词的目的和特点
- 提供使用示例

## 扩展指南

### 添加新的提示词类型

1. **创建新模块**
```rust
// src/prompts/refactoring.rs
pub struct RefactoringPrompts;

impl PromptGenerator for RefactoringPrompts {
    fn generate(&self, message_count: usize) -> String {
        // 实现逻辑
    }
}
```

2. **在 mod.rs 中导出**
```rust
pub mod refactoring;
pub use refactoring::RefactoringPrompts;

pub fn get_refactoring_prompt(message_count: usize) -> String {
    RefactoringPrompts.generate(message_count)
}
```

3. **在 App 中使用**
```rust
let prompt = prompts::get_refactoring_prompt(message_count);
```

## 性能考虑

- 提示词在运行时生成，不存储在内存中
- 字符串分配是最小的
- 适应性逻辑基于简单的消息计数

## 未来改进

1. **提示词版本控制** - 跟踪提示词的演变
2. **A/B 测试** - 比较不同的提示词效果
3. **用户反馈** - 根据用户反馈优化提示词
4. **多语言支持** - 支持不同语言的提示词
5. **动态提示词** - 基于用户偏好定制提示词
