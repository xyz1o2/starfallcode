/// AI 工具系统
///
/// 支持 LLM 函数调用（Function Calling）
///
/// # 架构
///
/// ```text
/// LLM 请求
///    ↓
/// Tool Registry (工具注册表)
///    ↓
/// Tool Execution (工具执行)
///    ↓
/// Tool Result (工具结果)
///    ↓
/// LLM 响应
/// ```
///
/// # 使用示例
///
/// ```rust,ignore
/// use starfellcode::tools::{Tool, ToolRegistry, ToolCall};
///
/// // 1. 创建工具注册表
/// let mut registry = ToolRegistry::new();
///
/// // 2. 注册工具
/// registry.register(Arc::new(MyTool));
///
/// // 3. 获取工具定义（发送给 LLM）
/// let definitions = registry.list_definitions();
///
/// // 4. 执行工具调用
/// let call = ToolCall {
///     tool_name: "my_tool".to_string(),
///     arguments: args,
/// };
/// let result = registry.execute(call).await;
/// ```

/// 核心工具系统
pub mod tool;
pub mod tool_registry;

/// 文件操作工具 - 读取、写入、修改文件
pub mod file_tools;

/// 字符串替换编辑器工具 - 替换文件中的文本内容
pub mod str_replace_tool;

/// 代码分析工具 - 语法分析、代码搜索等
pub mod code_tools;

/// 终端命令工具 - 执行系统命令
pub mod terminal_tools;

/// 项目管理工具 - 项目结构分析、依赖管理等
pub mod project_tools;

/// Todo 管理工具 - 创建和更新任务列表
pub mod todo_tool;

/// 工具使用示例
pub mod tool_examples;

// 重新导出核心类型
pub use tool::{ToolCall, ToolDefinition, ToolResult, ToolParameter};
pub use tool_registry::ToolRegistry;

// 重新导出具体工具类，方便使用
pub use file_tools::{FileReadTool, FileWriteTool, FileListTool};
pub use str_replace_tool::StrReplaceTool;
pub use code_tools::{CodeSearchTool, FunctionFinderTool, CodeStructureTool};
pub use terminal_tools::{CommandExecuteTool, EnvironmentInfoTool};
pub use project_tools::{ProjectStructureTool, DependencyAnalyzerTool, BuildTool};
pub use todo_tool::{CreateTodoListTool, UpdateTodoListTool, TodoItem, TodoStatus};