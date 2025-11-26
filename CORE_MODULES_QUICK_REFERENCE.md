# 核心模块快速参考

## 🎯 三个核心模块

### 1️⃣ 上下文优化 (Context Optimizer)
```rust
// 创建
let optimizer = ContextWindowOptimizer::new(ContextConfig::default());

// 优化消息
let optimized = optimizer.optimize_context(messages);

// 检查是否需要优化
if optimizer.needs_optimization(&messages) {
    // 执行优化
}

// 获取统计信息
let stats = optimizer.get_stats(&messages);
```

**关键参数**:
- `max_tokens`: 最大令牌数 (默认: 4000)
- `reserve_output_tokens`: 保留输出令牌 (默认: 1000)
- `enable_summarization`: 启用摘要 (默认: true)

---

### 2️⃣ 工具系统 (Pair Programming Tools)
```rust
// 创建
let mut tools = PairProgrammingTools::new();

// YOLO 模式
tools.enable_yolo_mode();
tools.disable_yolo_mode();

// 获取工具
let available = tools.get_available_tools();
let by_type = tools.get_tools_by_type("file_ops");
let by_priority = tools.get_tools_by_priority();

// 执行工具
let result = tools.execute_tool("file_read", params).await;
if result.success {
    println!("{}", result.output);
}
```

**可用工具**:
- `file_read`, `file_write`, `file_delete`, `file_list`
- `code_analyze`, `search_code`, `git_status`

---

### 3️⃣ 文件处理器 (Code File Handler)
```rust
// 创建
let mut handler = CodeFileHandler::new();

// YOLO 模式
handler.enable_yolo_mode();

// 文件操作
handler.read_file("path/to/file");
handler.write_file("path", "content");
handler.create_file("path", "content");
handler.delete_file("path", confirmed);

// 目录操作
handler.list_directory(".");
handler.search_files(".", "*.rs");

// 代码分析
handler.get_file_info("file.rs");
handler.get_code_context("file.rs");
```

**返回值**: `FileOperationResult { success, message, data }`

---

## 🔗 集成管理器

```rust
// 创建
let mut manager = IntegrationManager::new();

// 启用 YOLO 模式
manager.enable_yolo_mode();

// 访问子模块
manager.context_optimizer.optimize_context(msgs);
manager.tools.execute_tool("file_read", params).await;
manager.file_handler.read_file("file.rs");

// 获取状态
println!("{}", manager.get_status());
```

---

## 📋 常见操作

### 优化聊天历史
```rust
let optimized = manager.context_optimizer.optimize_context(messages);
if optimized.was_truncated {
    println!("历史已优化，摘要已添加");
}
```

### 读取代码文件
```rust
let result = manager.file_handler.read_file("src/main.rs");
if result.success {
    let content = result.data.unwrap();
}
```

### 分析代码结构
```rust
let result = manager.file_handler.get_code_context("src/main.rs");
// 返回: 函数、导入、类、摘要
```

### 执行文件操作
```rust
let mut params = ToolParams::new();
params.insert("path".to_string(), "file.txt".to_string());
let result = manager.tools.execute_tool("file_read", params).await;
```

### 启用 YOLO 模式
```rust
manager.enable_yolo_mode();
// 现在删除文件无需确认
manager.file_handler.delete_file("temp.rs", false);
```

---

## ⚙️ 配置示例

### 保守配置（小模型）
```rust
let config = ContextConfig {
    max_tokens: 2000,
    reserve_output_tokens: 500,
    min_messages_to_keep: 3,
    enable_summarization: true,
};
```

### 激进配置（大模型）
```rust
let config = ContextConfig {
    max_tokens: 100000,
    reserve_output_tokens: 5000,
    min_messages_to_keep: 10,
    enable_summarization: false,
};
```

---

## 🚨 错误处理

```rust
// 工具执行
match tools.execute_tool("file_read", params).await {
    Ok(result) => {
        if result.success {
            println!("成功: {}", result.output);
        } else {
            println!("失败: {}", result.error.unwrap_or_default());
        }
    }
    Err(e) => println!("错误: {}", e),
}

// 文件操作
let result = handler.read_file("file.rs");
if !result.success {
    println!("错误: {}", result.message);
}
```

---

## 📊 性能提示

- **上下文优化**: 消息数 > 50 时自动启用
- **文件读取**: 大于 10MB 的文件可能很慢
- **代码分析**: 支持最多 1000 行代码
- **搜索**: 限制在 10K 文件以内

---

## 🎮 YOLO 模式

```rust
// 启用
manager.enable_yolo_mode();

// 现在以下操作无需确认:
manager.file_handler.delete_file("file.rs", false);
manager.tools.execute_tool("file_delete", params).await;

// 禁用
manager.disable_yolo_mode();
```

**⚠️ 警告**: YOLO 模式会跳过所有确认，请谨慎使用！

---

## 📚 模块位置

| 模块 | 文件 | 导出 |
|------|------|------|
| ContextOptimizer | `src/core/context_optimizer.rs` | `core::context_optimizer` |
| PairProgrammingTools | `src/ai/tools.rs` | `ai::tools` |
| CodeFileHandler | `src/utils/code_file_handler.rs` | `utils::code_file_handler` |
| IntegrationManager | `src/core/integration.rs` | `core::integration` |

---

## 🔍 调试

```rust
// 获取统计信息
let stats = optimizer.get_stats(&messages);
println!("总令牌数: {}", stats.total_tokens);

// 检查工具状态
for tool in tools.get_available_tools() {
    println!("工具: {} (优先级: {})", tool.name, tool.priority);
}

// 获取管理器状态
println!("{}", manager.get_status());
```

---

## 💡 最佳实践

✅ **DO**:
- 定期优化上下文
- 检查操作结果
- 在生产环境禁用 YOLO
- 使用 try_recv 处理异步结果

❌ **DON'T**:
- 不要忽视 `was_truncated` 标志
- 不要在生产环境启用 YOLO
- 不要处理超大文件（>100MB）
- 不要忽视错误消息

---

**快速参考卡** | 版本 1.0 | 2025-11-27
