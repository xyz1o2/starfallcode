use ratatui::{
    layout::Rect,
    style::{Color, Style},
};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PanelType {
    Sidebar,
    MainChat,
    InfoPanel,
    StatusBar,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LayoutType {
    ThreePanel,     // 侧边栏 + 主区域 + 信息面板
    TwoPanel,       // 侧边栏 + 主区域
    SinglePanel,    // 仅主区域
    Overlay,        // 主区域 + 浮动面板
}

#[derive(Clone, Debug)]
pub struct PanelVisibility {
    pub sidebar: bool,
    pub main_chat: bool,
    pub info_panel: bool,
    pub status_bar: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            sidebar: true,
            main_chat: true,
            info_panel: true,
            status_bar: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PanelSizes {
    pub sidebar_width: u16,
    pub info_panel_width: u16,
    pub status_bar_height: u16,
}

impl Default for PanelSizes {
    fn default() -> Self {
        Self {
            sidebar_width: 25,
            info_panel_width: 25,
            status_bar_height: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResponsiveBreakpoints {
    pub large_screen: u16,   // > 120 characters
    pub medium_screen: u16,  // 80-120 characters
    pub small_screen: u16,   // < 80 characters
}

impl Default for ResponsiveBreakpoints {
    fn default() -> Self {
        Self {
            large_screen: 120,
            medium_screen: 80,
            small_screen: 40,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayoutAreas {
    pub sidebar: Option<Rect>,
    pub main_chat: Rect,
    pub info_panel: Option<Rect>,
    pub status_bar: Rect,
}

#[derive(Clone, Debug)]
pub enum PanelAction {
    FocusNext,
    FocusPrevious,
    FocusSidebar,
    FocusChat,
    FocusInfo,
    ToggleSidebar,
    ToggleInfoPanel,
}

#[derive(Clone, Debug)]
pub enum ChatAction {
    SendMessage,
    ClearInput,
    ScrollUp,
    ScrollDown,
    ClearHistory,
}

#[derive(Clone, Debug)]
pub enum GlobalAction {
    Quit,
    SwitchTheme,
    ShowHelp,
    ToggleFullscreen,
}

#[derive(Clone, Debug)]
pub enum SidebarAction {
    SelectNext,
    SelectPrevious,
    Activate,
    ExpandSection,
    CollapseSection,
}

#[derive(Clone, Debug)]
pub enum UIEvent {
    KeyPress(crossterm::event::KeyEvent),
    Resize(u16, u16),
    Focus(PanelType),
    ThemeChange(String),
}

#[derive(Clone, Debug)]
pub enum UIAction {
    None,
    Panel(PanelAction),
    Chat(ChatAction),
    Global(GlobalAction),
    Sidebar(SidebarAction),
    Redraw,
}

// Message types for enhanced chat display
#[derive(Clone, Debug)]
pub struct EnhancedChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub status: MessageStatus,
    pub metadata: MessageMetadata,
}

#[derive(Clone, Debug)]
pub enum MessageStatus {
    Sent,
    Receiving,
    Complete,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct MessageMetadata {
    pub tokens: Option<u32>,
    pub model: Option<String>,
    pub processing_time: Option<std::time::Duration>,
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            tokens: None,
            model: None,
            processing_time: None,
        }
    }
}

// Sidebar section types
#[derive(Clone, Debug)]
pub enum SidebarSection {
    ChatHistory(ChatHistorySection),
    QuickCommands(QuickCommandsSection),
    SystemStatus(SystemStatusSection),
    Settings(SettingsSection),
}

#[derive(Clone, Debug)]
pub struct ChatHistorySection {
    pub sessions: Vec<ChatSession>,
    pub selected_session: Option<usize>,
    pub max_display_items: usize,
}

#[derive(Clone, Debug)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub message_count: usize,
    pub last_updated: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct QuickCommandsSection {
    pub commands: Vec<QuickCommand>,
    pub categories: Vec<CommandCategory>,
}

#[derive(Clone, Debug)]
pub struct QuickCommand {
    pub name: String,
    pub description: String,
    pub shortcut: Option<String>,
    pub category: String,
}

#[derive(Clone, Debug)]
pub struct CommandCategory {
    pub name: String,
    pub expanded: bool,
    pub commands: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SystemStatusSection {
    pub connection_status: ConnectionStatus,
    pub model_info: Option<ModelInfo>,
    pub performance_stats: PerformanceStats,
}

#[derive(Clone, Debug)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Clone, Debug)]
pub struct PerformanceStats {
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub response_time: Option<std::time::Duration>,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            memory_usage: 0,
            cpu_usage: 0.0,
            response_time: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SettingsSection {
    pub theme_name: String,
    pub auto_save: bool,
    pub notifications: bool,
}

// Info panel section types
#[derive(Clone, Debug)]
pub enum InfoSection {
    ModelInfo(ModelInfoSection),
    TokenStats(TokenStatsSection),
    HelpInfo(HelpInfoSection),
    ErrorLog(ErrorLogSection),
    SessionStats(SessionStatsSection),
}

#[derive(Clone, Debug)]
pub struct ModelInfoSection {
    pub current_model: String,
    pub provider: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub connection_status: ConnectionStatus,
}

#[derive(Clone, Debug)]
pub struct TokenStatsSection {
    pub tokens_used: u32,
    pub tokens_remaining: Option<u32>,
    pub cost_estimate: Option<f64>,
    pub session_tokens: u32,
}

#[derive(Clone, Debug)]
pub struct HelpInfoSection {
    pub current_context: String,
    pub available_shortcuts: Vec<ShortcutInfo>,
    pub tips: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ShortcutInfo {
    pub key: String,
    pub description: String,
    pub context: String,
}

#[derive(Clone, Debug)]
pub struct ErrorLogSection {
    pub errors: Vec<ErrorEntry>,
    pub max_entries: usize,
}

#[derive(Clone, Debug)]
pub struct ErrorEntry {
    pub timestamp: DateTime<Utc>,
    pub level: ErrorLevel,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Clone, Debug)]
pub enum ErrorLevel {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Clone, Debug)]
pub struct SessionStatsSection {
    pub session_duration: std::time::Duration,
    pub messages_sent: u32,
    pub messages_received: u32,
    pub average_response_time: Option<std::time::Duration>,
}

// Status bar types
#[derive(Clone, Debug)]
pub struct StatusItem {
    pub content: String,
    pub style: Style,
    pub priority: u8,
    pub min_width: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub timestamp: DateTime<Utc>,
    pub auto_dismiss: Option<std::time::Duration>,
}

#[derive(Clone, Debug)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

// Focus management types
#[derive(Clone, Debug)]
pub struct FocusIndicators {
    pub active_border_style: Style,
    pub inactive_border_style: Style,
    pub focus_highlight: Color,
}

impl Default for FocusIndicators {
    fn default() -> Self {
        Self {
            active_border_style: Style::default().fg(Color::Cyan),
            inactive_border_style: Style::default().fg(Color::DarkGray),
            focus_highlight: Color::Cyan,
        }
    }
}

// UI State
#[derive(Clone, Debug)]
pub struct UIState {
    pub is_fullscreen: bool,
    pub show_help: bool,
    pub last_interaction: DateTime<Utc>,
    pub dirty_panels: std::collections::HashSet<PanelType>,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            is_fullscreen: false,
            show_help: false,
            last_interaction: Utc::now(),
            dirty_panels: std::collections::HashSet::new(),
        }
    }
}