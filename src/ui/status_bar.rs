use crate::ui::types::{StatusItem, Notification, NotificationLevel};
use crate::ui::theme::ModernTheme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::Duration;

pub struct ModernStatusBar {
    pub left_items: Vec<StatusItem>,
    pub center_items: Vec<StatusItem>,
    pub right_items: Vec<StatusItem>,
    pub notifications: Vec<Notification>,
    pub status_data: HashMap<String, String>,
    pub last_update: DateTime<Utc>,
}

impl ModernStatusBar {
    pub fn new() -> Self {
        let mut status_bar = Self {
            left_items: Vec::new(),
            center_items: Vec::new(),
            right_items: Vec::new(),
            notifications: Vec::new(),
            status_data: HashMap::new(),
            last_update: Utc::now(),
        };
        
        status_bar.init_default_items();
        status_bar
    }

    /// Initialize default status items
    fn init_default_items(&mut self) {
        // Left items - Mode and connection status
        self.left_items = vec![
            StatusItem {
                content: "CHAT".to_string(),
                style: Style::default().add_modifier(Modifier::BOLD),
                priority: 10,
                min_width: Some(6),
            },
            StatusItem {
                content: "🔴 Disconnected".to_string(),
                style: Style::default(),
                priority: 9,
                min_width: Some(15),
            },
        ];

        // Center items - Current context
        self.center_items = vec![
            StatusItem {
                content: "Ready".to_string(),
                style: Style::default().add_modifier(Modifier::ITALIC),
                priority: 5,
                min_width: None,
            },
        ];

        // Right items - Time and shortcuts
        self.right_items = vec![
            StatusItem {
                content: "F1:Help".to_string(),
                style: Style::default(),
                priority: 3,
                min_width: Some(8),
            },
            StatusItem {
                content: Utc::now().format("%H:%M:%S").to_string(),
                style: Style::default(),
                priority: 8,
                min_width: Some(8),
            },
        ];

        // Initialize status data
        self.status_data.insert("mode".to_string(), "CHAT".to_string());
        self.status_data.insert("connection".to_string(), "Disconnected".to_string());
        self.status_data.insert("context".to_string(), "Ready".to_string());
    }

    /// Render the status bar
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &ModernTheme) {
        // Create background block
        let block = Block::default()
            .style(Style::default()
                .bg(theme.colors.surface)
                .fg(theme.colors.text_primary));

        frame.render_widget(block, area);

        if area.width < 10 {
            return; // Not enough space to render anything meaningful
        }

        // Calculate layout for left, center, and right sections
        let available_width = area.width as usize;
        let left_width = self.calculate_section_width(&self.left_items, available_width / 3);
        let right_width = self.calculate_section_width(&self.right_items, available_width / 3);
        let center_width = available_width.saturating_sub(left_width + right_width);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(left_width as u16),
                Constraint::Length(center_width as u16),
                Constraint::Length(right_width as u16),
            ])
            .split(area);

        // Render each section
        self.render_section(frame, &self.left_items, chunks[0], Alignment::Left, theme);
        self.render_section(frame, &self.center_items, chunks[1], Alignment::Center, theme);
        self.render_section(frame, &self.right_items, chunks[2], Alignment::Right, theme);

        // Render notifications if any
        if !self.notifications.is_empty() {
            self.render_notifications(frame, area, theme);
        }
    }

    /// Render a section of status items
    fn render_section(
        &self,
        frame: &mut Frame,
        items: &[StatusItem],
        area: Rect,
        alignment: Alignment,
        theme: &ModernTheme,
    ) {
        if items.is_empty() || area.width == 0 {
            return;
        }

        let mut spans = Vec::new();
        let available_width = area.width as usize;
        let mut used_width = 0;

        // Sort items by priority (higher priority first)
        let mut sorted_items = items.to_vec();
        sorted_items.sort_by(|a, b| b.priority.cmp(&a.priority));

        for (i, item) in sorted_items.iter().enumerate() {
            let item_width = item.min_width.unwrap_or(item.content.len() as u16) as usize;
            
            // Check if we have space for this item
            if used_width + item_width > available_width {
                break;
            }

            // Add separator between items
            if i > 0 && used_width > 0 {
                spans.push(Span::styled(" | ", theme.typography.caption_style));
                used_width += 3;
            }

            // Add the item
            spans.push(Span::styled(&item.content, item.style));
            used_width += item_width;
        }

        if !spans.is_empty() {
            let line = Line::from(spans);
            let paragraph = Paragraph::new(vec![line])
                .alignment(alignment)
                .style(Style::default()
                    .bg(theme.colors.surface)
                    .fg(theme.colors.text_primary));

            frame.render_widget(paragraph, area);
        }
    }

    /// Render notifications overlay
    fn render_notifications(&self, frame: &mut Frame, area: Rect, theme: &ModernTheme) {
        if let Some(notification) = self.notifications.last() {
            // Create a small overlay area for the notification
            let notification_width = std::cmp::min(50, area.width);
            let notification_area = Rect {
                x: area.x + (area.width - notification_width) / 2,
                y: area.y,
                width: notification_width,
                height: 1,
            };

            let (icon, color) = match notification.level {
                NotificationLevel::Info => ("ℹ️", theme.colors.info),
                NotificationLevel::Success => ("✅", theme.colors.success),
                NotificationLevel::Warning => ("⚠️", theme.colors.warning),
                NotificationLevel::Error => ("❌", theme.colors.error),
            };

            let notification_text = format!("{} {}", icon, notification.message);
            let notification_line = Line::from(Span::styled(
                notification_text,
                Style::default()
                    .fg(color)
                    .bg(theme.colors.background)
                    .add_modifier(Modifier::BOLD),
            ));

            let notification_paragraph = Paragraph::new(vec![notification_line])
                .alignment(Alignment::Center);

            frame.render_widget(notification_paragraph, notification_area);
        }
    }

    /// Calculate the width needed for a section
    fn calculate_section_width(&self, items: &[StatusItem], max_width: usize) -> usize {
        let mut total_width = 0;
        let mut sorted_items = items.to_vec();
        sorted_items.sort_by(|a, b| b.priority.cmp(&a.priority));

        for (i, item) in sorted_items.iter().enumerate() {
            let item_width = item.min_width.unwrap_or(item.content.len() as u16) as usize;
            let separator_width = if i > 0 { 3 } else { 0 }; // " | "
            
            if total_width + item_width + separator_width > max_width {
                break;
            }
            
            total_width += item_width + separator_width;
        }

        std::cmp::min(total_width, max_width)
    }

    /// Update a status item by key
    pub fn update_status(&mut self, key: &str, value: String) {
        self.status_data.insert(key.to_string(), value.clone());
        self.last_update = Utc::now();

        // Update corresponding status items
        match key {
            "mode" => {
                if let Some(item) = self.left_items.get_mut(0) {
                    item.content = value;
                }
            }
            "connection" => {
                let icon = match value.as_str() {
                    "Connected" => "🟢",
                    "Connecting" => "🟡",
                    "Disconnected" => "🔴",
                    _ => "❓",
                };
                if let Some(item) = self.left_items.get_mut(1) {
                    item.content = format!("{} {}", icon, value);
                }
            }
            "context" => {
                if let Some(item) = self.center_items.get_mut(0) {
                    item.content = value;
                }
            }
            "time" => {
                if let Some(item) = self.right_items.last_mut() {
                    item.content = value;
                }
            }
            _ => {
                // Handle custom status updates
            }
        }
    }

    /// Update item content by index
    fn update_item_content(&mut self, items: &mut [StatusItem], index: usize, content: String) {
        if let Some(item) = items.get_mut(index) {
            item.content = content;
        }
    }

    /// Add a notification
    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
        
        // Limit notification history
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }

    /// Clear all notifications
    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    /// Remove expired notifications
    pub fn cleanup_notifications(&mut self) {
        let now = Utc::now();
        self.notifications.retain(|notification| {
            if let Some(auto_dismiss) = notification.auto_dismiss {
                let elapsed = now.signed_duration_since(notification.timestamp);
                elapsed.to_std().unwrap_or(Duration::from_secs(0)) < auto_dismiss
            } else {
                true // Keep notifications without auto-dismiss
            }
        });
    }

    /// Update time display
    pub fn update_time(&mut self) {
        let current_time = Utc::now().format("%H:%M:%S").to_string();
        self.update_status("time", current_time);
    }

    /// Set connection status
    pub fn set_connection_status(&mut self, status: &str) {
        self.update_status("connection", status.to_string());
    }

    /// Set current mode
    pub fn set_mode(&mut self, mode: &str) {
        self.update_status("mode", mode.to_uppercase());
    }

    /// Set context information
    pub fn set_context(&mut self, context: &str) {
        self.update_status("context", context.to_string());
    }

    /// Add shortcut hint
    pub fn add_shortcut_hint(&mut self, shortcut: String) {
        let shortcut_item = StatusItem {
            content: shortcut,
            style: Style::default(),
            priority: 2,
            min_width: None,
        };
        
        // Insert before the time (last item)
        if !self.right_items.is_empty() {
            let last_index = self.right_items.len() - 1;
            self.right_items.insert(last_index, shortcut_item);
        } else {
            self.right_items.push(shortcut_item);
        }
    }

    /// Clear shortcut hints
    pub fn clear_shortcut_hints(&mut self) {
        // Keep only the time item (last item with highest priority)
        self.right_items.retain(|item| item.priority >= 8);
    }

    /// Show temporary message
    pub fn show_message(&mut self, message: String, level: NotificationLevel, duration: Option<Duration>) {
        let notification = Notification {
            message,
            level,
            timestamp: Utc::now(),
            auto_dismiss: duration,
        };
        self.add_notification(notification);
    }

    /// Get status value by key
    pub fn get_status(&self, key: &str) -> Option<&String> {
        self.status_data.get(key)
    }

    /// Check if there are active notifications
    pub fn has_notifications(&self) -> bool {
        !self.notifications.is_empty()
    }

    /// Get the most recent notification
    pub fn get_latest_notification(&self) -> Option<&Notification> {
        self.notifications.last()
    }
}

impl Default for ModernStatusBar {
    fn default() -> Self {
        Self::new()
    }
}