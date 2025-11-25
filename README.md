# Starfall Code - 幽灵文本编辑器

Starfall Code 是一个用 Rust 编写的基于终端的文本编辑器，具有类似 GitHub Copilot 的 AI 驱动的"幽灵文本"建议功能。该编辑器使用 ratatui 和 crossterm 在终端中渲染，提供具有预测文本功能的现代编辑体验。

## 主要功能

- 基于终端的文本编辑，使用 ratatui
- 幽灵文本建议（AI 驱动的补全显示为淡色文本）
- 按 Tab 键接受幽灵文本建议
- 按 Escape 键清除幽灵文本建议
- 基本的光标移动和文本输入

## 预览

（此处可以添加编辑器的截图或演示 GIF，当前为占位符）

## 安装

### 先决条件

- Rust 1.70 或更高版本
- Cargo 包管理器

### 构建和运行

```bash
# 克隆仓库
git clone https://github.com/your-username/starfallcode.git
cd starfallcode

# 构建项目
cargo build --release

# 运行项目
cargo run
```

### 2. 运行编辑器

```bash
cargo run --release
```

### 3. 使用编辑器

- `Tab`: 在编辑器和聊天框之间切换
- `Enter`: 在编辑器中换行，在聊天中发送消息
- `Ctrl+Q`: 退出应用
- `Esc`: 从聊天框返回编辑器

## 使用说明

编辑器以终端模式启动，具有以下按键绑定：

- `Tab`: 在编辑器和聊天框之间切换
- `Enter`: 在编辑器中换行，在聊天中发送消息
- `Esc`: 清除幽灵文本建议或从聊天框返回
- 方向键: 移动光标
- `Ctrl+Q`: 退出应用
- 标准输入: 插入文本
- `ui/`: 终端用户界面组件（编辑器渲染）
- `core/`: 核心文本编辑组件（目前主要是占位符）
- `ai/`: AI 完成功能（目前主要是占位符）
- `events/`: 事件处理（目录存在但没有可见文件）
- `utils/`: 实用函数（目录存在但没有可见文件）

## 依赖项

编辑器使用以下关键依赖：

- `ropey`: 高效的绳索数据结构用于文本缓冲区管理
- `ratatui`: 终端用户界面渲染
- `crossterm`: 跨平台终端操作
- `tokio`: 用于潜在 AI 集成的异步运行时
- `tree-sitter`: 语法分析（已集成但尚未完全利用）
- `reqwest`: 用于 AI API 调用的 HTTP 客户端（未来实现）

## 架构

- 代码遵循 Rust 习惯用法和命名约定
- UI 渲染与应用逻辑分离
- 模块化设计，为 UI、核心功能和 AI 特性提供专用模块
- 异步设计，准备进行 AI 集成

## AI 组件

AI 功能已计划但目前由占位符模块组成：

- `ai/client.rs`: 将处理与 AI 服务的通信
- `ai/context.rs`: 将管理 AI 完成的上下文
- `ai/fim.rs`: 将实现填空功能（FIM）

## 核心组件

- `core/buffer.rs`: 计划中的缓冲区操作（当前为占位符）
- `core/cursor.rs`: 计划中的光标管理（当前为占位符）

## 贡献

我们欢迎各种形式的贡献！要贡献，请：

1. Fork 仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 未来计划

- 实现完整的 AI 完成管道
- 添加语法高亮
- 实现文件打开/保存功能
- 添加更多编辑器功能（撤销/重做、搜索等）
- 实现插件系统

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [ratatui](https://github.com/ratatui-org/ratatui) - 终端 UI 库
- [crossterm](https://github.com/crossterm-rs/crossterm) - 跨平台终端操作
- [ropey](https://github.com/cessen/ropey) - 高效文本缓冲区管理
- [tree-sitter](https://github.com/tree-sitter/tree-sitter) - 语法分析