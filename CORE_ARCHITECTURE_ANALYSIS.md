# 核心架构分析 - 项目的真正核心是什么

## 现状分析

### 已完成的模块
```
✅ PromptBuilder (规则提示词)
✅ SmartChatDisplay (智能显示)
✅ ToolRegistry (工具系统)
✅ CodeModificationDetector (代码修改)
✅ FileSearchEngine (文件搜索)
✅ AICodeModificationDetector (AI 检测)
```

### 问题：这些都是孤立的！

```
┌─────────────────────────────────────────────┐
│         当前架构（碎片化）                   │
├─────────────────────────────────────────────┤
│                                             │
│  PromptBuilder     SmartChatDisplay        │
│       ↓                   ↓                 │
│  [独立]            [独立]                   │
│                                             │
│  ToolRegistry      CodeModification        │
│       ↓                   ↓                 │
│  [独立]            [独立]                   │
│                                             │
│  FileSearchEngine  AIDetector              │
│       ↓                   ↓                 │
│  [独立]            [独立]                   │
│                                             │
└─────────────────────────────────────────────┘
```

## 真正的核心：对话流程引擎

### 核心应该是什么？

**对话流程引擎** - 统一协调所有模块的中枢

```
┌─────────────────────────────────────────────────────┐
│         对话流程引擎（ConversationEngine）          │
├─────────────────────────────────────────────────────┤
│                                                     │
│  1. 意图识别 (Intent Recognition)                  │
│     ├─ 用户输入 → 识别意图类型                      │
│     ├─ @mention → 文件提及                         │
│     ├─ /command → 命令执行                         │
│     └─ 普通聊天 → AI 对话                          │
│                                                     │
│  2. 上下文管理 (Context Management)                │
│     ├─ 加载相关文件内容                            │
│     ├─ 构建 PromptBuilder 消息                     │
│     ├─ 应用规则提示词                              │
│     └─ 管理对话历史                                │
│                                                     │
│  3. LLM 调用 (LLM Invocation)                      │
│     ├─ 发送消息给 LLM                              │
│     ├─ 处理流式响应                                │
│     ├─ 检测工具调用                                │
│     └─ 执行工具                                    │
│                                                     │
│  4. 响应处理 (Response Processing)                 │
│     ├─ 检测代码修改指令                            │
│     ├─ 生成 Diff 对比                              │
│     ├─ 显示建议提示                                │
│     └─ 更新 SmartChatDisplay                       │
│                                                     │
│  5. 用户交互 (User Interaction)                    │
│     ├─ 确认修改                                    │
│     ├─ 选择建议                                    │
│     ├─ 执行命令                                    │
│     └─ 继续对话                                    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## 核心流程

### 完整的对话流程

```
用户输入
   ↓
[1] 意图识别
   ├─ 是否是 @mention？
   ├─ 是否是 /command？
   ├─ 是否是普通聊天？
   └─ 是否是特殊指令？
   ↓
[2] 上下文构建
   ├─ 加载 @mention 的文件
   ├─ 使用 PromptBuilder 构建消息
   ├─ 注入规则提示词
   └─ 获取工具定义
   ↓
[3] LLM 调用
   ├─ 发送消息
   ├─ 流式接收响应
   └─ 显示思考过程
   ↓
[4] 响应分析
   ├─ 检测工具调用
   ├─ 检测代码修改
   ├─ 检测建议提示
   └─ 生成 Diff
   ↓
[5] 用户确认
   ├─ 显示修改确认对话
   ├─ 显示建议提示
   ├─ 等待用户选择
   └─ 执行或取消
   ↓
对话继续或结束
```

## 核心模块设计

### 1. ConversationEngine（对话引擎）

```rust
pub struct ConversationEngine {
    // 核心组件
    intent_recognizer: IntentRecognizer,
    context_manager: ContextManager,
    llm_client: LLMClient,
    response_processor: ResponseProcessor,
    
    // 辅助系统
    prompt_builder: PromptBuilder,
    tool_registry: ToolRegistry,
    file_search: FileSearchEngine,
    
    // 显示系统
    smart_display: SmartChatDisplay,
    
    // 状态
    conversation_history: Vec<Message>,
    current_context: ConversationContext,
}

impl ConversationEngine {
    pub async fn process_user_input(&mut self, input: String) -> Result<()> {
        // 1. 识别意图
        let intent = self.intent_recognizer.recognize(&input)?;
        
        // 2. 构建上下文
        let context = self.context_manager.build(&input, &intent)?;
        
        // 3. 调用 LLM
        let response = self.llm_client.chat(
            self.prompt_builder.build_messages(&input),
            self.tool_registry.list_definitions()
        ).await?;
        
        // 4. 处理响应
        self.response_processor.process(&response)?;
        
        // 5. 更新显示
        self.smart_display.add_message(response_message);
        
        Ok(())
    }
}
```

### 2. IntentRecognizer（意图识别）

```rust
pub enum UserIntent {
    // 文件相关
    FileMention {
        paths: Vec<String>,
        query: String,
    },
    
    // 命令相关
    Command {
        name: String,
        args: Vec<String>,
    },
    
    // 聊天相关
    Chat {
        query: String,
        context_files: Vec<String>,
    },
    
    // 代码相关
    CodeReview {
        files: Vec<String>,
        focus: String,
    },
    
    // 调试相关
    Debug {
        issue: String,
        files: Vec<String>,
    },
}

pub struct IntentRecognizer;

impl IntentRecognizer {
    pub fn recognize(&self, input: &str) -> Result<UserIntent> {
        // 1. 检测 @mention
        if input.contains('@') {
            return self.extract_file_mention(input);
        }
        
        // 2. 检测 /command
        if input.starts_with('/') {
            return self.extract_command(input);
        }
        
        // 3. 检测关键词
        if self.contains_code_keywords(input) {
            return self.extract_code_intent(input);
        }
        
        // 4. 默认聊天
        Ok(UserIntent::Chat {
            query: input.to_string(),
            context_files: vec![],
        })
    }
}
```

### 3. ContextManager（上下文管理）

```rust
pub struct ConversationContext {
    pub user_input: String,
    pub intent: UserIntent,
    pub files: Vec<FileContent>,
    pub rules: String,
    pub history: Vec<Message>,
    pub tools: Vec<ToolDefinition>,
}

pub struct ContextManager {
    file_search: FileSearchEngine,
    prompt_builder: PromptBuilder,
}

impl ContextManager {
    pub async fn build(
        &self,
        input: &str,
        intent: &UserIntent,
    ) -> Result<ConversationContext> {
        // 1. 加载文件
        let files = match intent {
            UserIntent::FileMention { paths, .. } => {
                self.load_files(paths).await?
            }
            _ => vec![],
        };
        
        // 2. 加载规则
        let rules = self.prompt_builder.load_augment_rules()?;
        
        // 3. 构建消息
        let messages = self.prompt_builder.build_messages(input);
        
        Ok(ConversationContext {
            user_input: input.to_string(),
            intent: intent.clone(),
            files,
            rules,
            history: messages,
            tools: vec![],
        })
    }
}
```

### 4. ResponseProcessor（响应处理）

```rust
pub struct ResponseProcessor {
    code_detector: AICodeModificationDetector,
    suggestion_generator: SuggestionGenerator,
}

impl ResponseProcessor {
    pub fn process(&self, response: &str) -> Result<ProcessedResponse> {
        // 1. 检测代码修改
        let modifications = self.code_detector.detect(response)?;
        
        // 2. 生成建议
        let suggestions = self.suggestion_generator.generate(response)?;
        
        // 3. 提取关键信息
        let key_points = self.extract_key_points(response)?;
        
        Ok(ProcessedResponse {
            content: response.to_string(),
            modifications,
            suggestions,
            key_points,
        })
    }
}
```

## 完善方向

### 优先级 1：核心引擎（必须）
- [ ] 实现 `ConversationEngine`
- [ ] 实现 `IntentRecognizer`
- [ ] 实现 `ContextManager`
- [ ] 实现 `ResponseProcessor`

### 优先级 2：集成现有模块（必须）
- [ ] 集成 `PromptBuilder`
- [ ] 集成 `SmartChatDisplay`
- [ ] 集成 `ToolRegistry`
- [ ] 集成 `CodeModificationDetector`

### 优先级 3：增强功能（可选）
- [ ] 对话历史持久化
- [ ] 上下文压缩
- [ ] 多轮对话优化
- [ ] 错误恢复机制

## 架构图

```
┌─────────────────────────────────────────────────────────┐
│                    User Interface                       │
│                  (Ratatui TUI)                          │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              ConversationEngine                         │
│  (对话流程引擎 - 核心)                                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ IntentRecognizer (意图识别)                      │  │
│  │ ContextManager (上下文管理)                      │  │
│  │ ResponseProcessor (响应处理)                     │  │
│  └──────────────────────────────────────────────────┘  │
│                     ↓                                   │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 辅助系统                                         │  │
│  ├──────────────────────────────────────────────────┤  │
│  │ • PromptBuilder (规则提示词)                     │  │
│  │ • ToolRegistry (工具系统)                        │  │
│  │ • FileSearchEngine (文件搜索)                    │  │
│  │ • SmartChatDisplay (智能显示)                    │  │
│  └──────────────────────────────────────────────────┘  │
│                     ↓                                   │
│  ┌──────────────────────────────────────────────────┐  │
│  │ LLM Client (LLM 客户端)                          │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

## 总结

### 核心是什么？
**对话流程引擎** - 统一协调用户输入、意图识别、上下文管理、LLM 调用、响应处理的中枢系统。

### 为什么这是核心？
1. **统一入口** - 所有用户交互都通过它
2. **流程控制** - 管理完整的对话生命周期
3. **模块协调** - 整合所有已完成的模块
4. **可扩展** - 易于添加新功能

### 如何完善？
1. 实现 `ConversationEngine` 和相关组件
2. 集成现有的所有模块
3. 添加错误处理和恢复机制
4. 优化性能和用户体验

### 预期效果
```
用户输入 → 意图识别 → 上下文构建 → LLM 调用 → 响应处理 → 用户交互
                                                    ↓
                                            完整的对话体验
```
