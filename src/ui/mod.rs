/// UI 模块 - 使用 pixel_layout 进行渲染
///
/// 架构：
/// - pixel_layout.rs: 主渲染引擎（像素艺术风格）
/// - render_engine.rs: 渲染引擎（集成缓存和优化）
/// - optimized_renderer.rs: 优化渲染组件
/// - 其他模块：辅助功能（命令提示、文件搜索等）

pub mod layout;
pub mod sidebar;
pub mod info_panel;
pub mod theme;
pub mod focus;
pub mod types;
pub mod command_hints;
pub mod mention_suggestions;
pub mod file_search;
pub mod render_cache;
pub mod optimized_renderer;
pub mod render_engine;
pub mod pixel_layout;
pub mod pixel_layout_v2;
pub mod avatar;
pub mod svg_avatar;
pub mod smart_chat_display;

pub use smart_chat_display::{
    SmartChatDisplay, SmartMessage, MessageRole, MessageType,
    ThinkingDisplay, SuggestionBar, StreamingDisplay,
    CodeBlock, Issue, IssueSeverity, MessageMetadata, ChatStats,
};