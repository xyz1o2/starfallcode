use ratatui::style::{Color, Style, Modifier};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ModernTheme {
    pub name: String,
    pub colors: ColorScheme,
    pub typography: Typography,
    pub spacing: Spacing,
    pub borders: BorderStyles,
}

#[derive(Clone, Debug)]
pub struct ColorScheme {
    // 基础颜色
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    
    // 语义颜色
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // 消息颜色
    pub user_message: Color,
    pub assistant_message: Color,
    pub system_message: Color,
    
    // UI元素颜色
    pub border_active: Color,
    pub border_inactive: Color,
    pub selection: Color,
    pub highlight: Color,
}

#[derive(Clone, Debug)]
pub struct Typography {
    pub title_style: Style,
    pub heading_style: Style,
    pub body_style: Style,
    pub caption_style: Style,
    pub code_style: Style,
}

#[derive(Clone, Debug)]
pub struct Spacing {
    pub panel_padding: u16,
    pub section_spacing: u16,
    pub item_spacing: u16,
}

#[derive(Clone, Debug)]
pub struct BorderStyles {
    pub panel_border: Style,
    pub active_border: Style,
    pub inactive_border: Style,
    pub section_border: Style,
}

impl ModernTheme {
    /// Dark Professional Theme - 专业深色主题
    pub fn dark_professional() -> Self {
        Self {
            name: "Dark Professional".to_string(),
            colors: ColorScheme {
                primary: Color::Rgb(100, 149, 237),      // Cornflower Blue
                secondary: Color::Rgb(147, 112, 219),     // Medium Slate Blue
                background: Color::Rgb(30, 30, 30),       // Dark Gray
                surface: Color::Rgb(45, 45, 45),          // Lighter Dark Gray
                text_primary: Color::Rgb(240, 240, 240),  // Light Gray
                text_secondary: Color::Rgb(180, 180, 180), // Medium Gray
                
                success: Color::Rgb(72, 187, 120),        // Green
                warning: Color::Rgb(255, 193, 7),         // Amber
                error: Color::Rgb(220, 53, 69),           // Red
                info: Color::Rgb(23, 162, 184),           // Cyan
                
                user_message: Color::Rgb(100, 149, 237),  // Blue
                assistant_message: Color::Rgb(72, 187, 120), // Green
                system_message: Color::Rgb(255, 193, 7),  // Amber
                
                border_active: Color::Rgb(100, 149, 237), // Primary Blue
                border_inactive: Color::Rgb(80, 80, 80),  // Dark Gray
                selection: Color::Rgb(60, 90, 150),       // Darker Blue
                highlight: Color::Rgb(255, 255, 100),     // Yellow
            },
            typography: Typography {
                title_style: Style::default()
                    .fg(Color::Rgb(240, 240, 240))
                    .add_modifier(Modifier::BOLD),
                heading_style: Style::default()
                    .fg(Color::Rgb(100, 149, 237))
                    .add_modifier(Modifier::BOLD),
                body_style: Style::default()
                    .fg(Color::Rgb(240, 240, 240)),
                caption_style: Style::default()
                    .fg(Color::Rgb(180, 180, 180))
                    .add_modifier(Modifier::ITALIC),
                code_style: Style::default()
                    .fg(Color::Rgb(255, 193, 7))
                    .bg(Color::Rgb(45, 45, 45)),
            },
            spacing: Spacing {
                panel_padding: 1,
                section_spacing: 1,
                item_spacing: 0,
            },
            borders: BorderStyles {
                panel_border: Style::default().fg(Color::Rgb(80, 80, 80)),
                active_border: Style::default().fg(Color::Rgb(100, 149, 237)),
                inactive_border: Style::default().fg(Color::Rgb(60, 60, 60)),
                section_border: Style::default().fg(Color::Rgb(70, 70, 70)),
            },
        }
    }

    /// Light Clean Theme - 简洁浅色主题
    pub fn light_clean() -> Self {
        Self {
            name: "Light Clean".to_string(),
            colors: ColorScheme {
                primary: Color::Rgb(0, 123, 255),         // Bootstrap Blue
                secondary: Color::Rgb(108, 117, 125),     // Bootstrap Gray
                background: Color::Rgb(255, 255, 255),    // White
                surface: Color::Rgb(248, 249, 250),       // Light Gray
                text_primary: Color::Rgb(33, 37, 41),     // Dark Gray
                text_secondary: Color::Rgb(108, 117, 125), // Medium Gray
                
                success: Color::Rgb(40, 167, 69),         // Green
                warning: Color::Rgb(255, 193, 7),         // Amber
                error: Color::Rgb(220, 53, 69),           // Red
                info: Color::Rgb(23, 162, 184),           // Cyan
                
                user_message: Color::Rgb(0, 123, 255),    // Blue
                assistant_message: Color::Rgb(40, 167, 69), // Green
                system_message: Color::Rgb(255, 193, 7),  // Amber
                
                border_active: Color::Rgb(0, 123, 255),   // Primary Blue
                border_inactive: Color::Rgb(206, 212, 218), // Light Gray
                selection: Color::Rgb(230, 240, 255),     // Light Blue
                highlight: Color::Rgb(255, 235, 59),      // Yellow
            },
            typography: Typography {
                title_style: Style::default()
                    .fg(Color::Rgb(33, 37, 41))
                    .add_modifier(Modifier::BOLD),
                heading_style: Style::default()
                    .fg(Color::Rgb(0, 123, 255))
                    .add_modifier(Modifier::BOLD),
                body_style: Style::default()
                    .fg(Color::Rgb(33, 37, 41)),
                caption_style: Style::default()
                    .fg(Color::Rgb(108, 117, 125))
                    .add_modifier(Modifier::ITALIC),
                code_style: Style::default()
                    .fg(Color::Rgb(220, 53, 69))
                    .bg(Color::Rgb(248, 249, 250)),
            },
            spacing: Spacing {
                panel_padding: 1,
                section_spacing: 1,
                item_spacing: 0,
            },
            borders: BorderStyles {
                panel_border: Style::default().fg(Color::Rgb(206, 212, 218)),
                active_border: Style::default().fg(Color::Rgb(0, 123, 255)),
                inactive_border: Style::default().fg(Color::Rgb(233, 236, 239)),
                section_border: Style::default().fg(Color::Rgb(220, 220, 220)),
            },
        }
    }

    /// High Contrast Theme - 高对比度主题
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".to_string(),
            colors: ColorScheme {
                primary: Color::White,
                secondary: Color::Rgb(255, 255, 0),       // Bright Yellow
                background: Color::Black,
                surface: Color::Rgb(20, 20, 20),          // Very Dark Gray
                text_primary: Color::White,
                text_secondary: Color::Rgb(200, 200, 200),
                
                success: Color::Rgb(0, 255, 0),           // Bright Green
                warning: Color::Rgb(255, 255, 0),         // Bright Yellow
                error: Color::Rgb(255, 0, 0),             // Bright Red
                info: Color::Rgb(0, 255, 255),            // Bright Cyan
                
                user_message: Color::Rgb(0, 255, 255),    // Bright Cyan
                assistant_message: Color::Rgb(0, 255, 0), // Bright Green
                system_message: Color::Rgb(255, 255, 0),  // Bright Yellow
                
                border_active: Color::White,
                border_inactive: Color::Rgb(128, 128, 128), // Gray
                selection: Color::Rgb(0, 0, 255),         // Bright Blue
                highlight: Color::Rgb(255, 0, 255),       // Bright Magenta
            },
            typography: Typography {
                title_style: Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                heading_style: Style::default()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
                body_style: Style::default()
                    .fg(Color::White),
                caption_style: Style::default()
                    .fg(Color::Rgb(200, 200, 200))
                    .add_modifier(Modifier::ITALIC),
                code_style: Style::default()
                    .fg(Color::Rgb(0, 255, 255))
                    .bg(Color::Rgb(20, 20, 20)),
            },
            spacing: Spacing {
                panel_padding: 1,
                section_spacing: 1,
                item_spacing: 0,
            },
            borders: BorderStyles {
                panel_border: Style::default().fg(Color::White),
                active_border: Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
                inactive_border: Style::default().fg(Color::Rgb(128, 128, 128)),
                section_border: Style::default().fg(Color::Rgb(160, 160, 160)),
            },
        }
    }

    /// Terminal Classic Theme - 经典终端主题
    pub fn terminal_classic() -> Self {
        Self {
            name: "Terminal Classic".to_string(),
            colors: ColorScheme {
                primary: Color::Green,
                secondary: Color::Yellow,
                background: Color::Black,
                surface: Color::Black,
                text_primary: Color::Green,
                text_secondary: Color::Rgb(0, 200, 0),    // Lighter Green
                
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                info: Color::Cyan,
                
                user_message: Color::White,
                assistant_message: Color::Green,
                system_message: Color::Yellow,
                
                border_active: Color::Green,
                border_inactive: Color::Rgb(0, 100, 0),   // Dark Green
                selection: Color::Rgb(0, 80, 0),          // Very Dark Green
                highlight: Color::Yellow,
            },
            typography: Typography {
                title_style: Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                heading_style: Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
                body_style: Style::default()
                    .fg(Color::Green),
                caption_style: Style::default()
                    .fg(Color::Rgb(0, 150, 0))
                    .add_modifier(Modifier::DIM),
                code_style: Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::Black),
            },
            spacing: Spacing {
                panel_padding: 1,
                section_spacing: 1,
                item_spacing: 0,
            },
            borders: BorderStyles {
                panel_border: Style::default().fg(Color::Green),
                active_border: Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                inactive_border: Style::default().fg(Color::Rgb(0, 100, 0)),
                section_border: Style::default().fg(Color::Rgb(0, 120, 0)),
            },
        }
    }

    /// Get all available themes
    pub fn all_themes() -> HashMap<String, ModernTheme> {
        let mut themes = HashMap::new();
        
        let dark_prof = Self::dark_professional();
        themes.insert(dark_prof.name.clone(), dark_prof);
        
        let light_clean = Self::light_clean();
        themes.insert(light_clean.name.clone(), light_clean);
        
        let high_contrast = Self::high_contrast();
        themes.insert(high_contrast.name.clone(), high_contrast);
        
        let terminal_classic = Self::terminal_classic();
        themes.insert(terminal_classic.name.clone(), terminal_classic);
        
        themes
    }

    /// Get theme by name, fallback to dark professional
    pub fn get_theme(name: &str) -> ModernTheme {
        match name {
            "Light Clean" => Self::light_clean(),
            "High Contrast" => Self::high_contrast(),
            "Terminal Classic" => Self::terminal_classic(),
            _ => Self::dark_professional(), // Default fallback
        }
    }

    /// Get style for message based on role
    pub fn get_message_style(&self, role: &str) -> Style {
        let color = match role {
            "user" => self.colors.user_message,
            "assistant" => self.colors.assistant_message,
            "system" => self.colors.system_message,
            _ => self.colors.text_primary,
        };
        Style::default().fg(color)
    }

    /// Get border style based on focus state
    pub fn get_border_style(&self, focused: bool) -> Style {
        if focused {
            self.borders.active_border
        } else {
            self.borders.inactive_border
        }
    }

    /// Get selection style
    pub fn get_selection_style(&self) -> Style {
        Style::default()
            .bg(self.colors.selection)
            .fg(self.colors.text_primary)
    }

    /// Get highlight style
    pub fn get_highlight_style(&self) -> Style {
        Style::default()
            .bg(self.colors.highlight)
            .fg(self.colors.background)
            .add_modifier(Modifier::BOLD)
    }
}