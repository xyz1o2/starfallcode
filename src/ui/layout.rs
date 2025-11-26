use crate::ui::types::{
    LayoutType, PanelType, PanelVisibility, PanelSizes, ResponsiveBreakpoints, LayoutAreas
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
};

pub struct LayoutManager {
    pub current_layout: LayoutType,
    pub panel_visibility: PanelVisibility,
    pub panel_sizes: PanelSizes,
    pub responsive_breakpoints: ResponsiveBreakpoints,
    pub terminal_size: Rect,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            current_layout: LayoutType::ThreePanel,
            panel_visibility: PanelVisibility::default(),
            panel_sizes: PanelSizes::default(),
            responsive_breakpoints: ResponsiveBreakpoints::default(),
            terminal_size: Rect::default(),
        }
    }

    /// Calculate layout areas based on current terminal size and settings
    pub fn calculate_layout(&self, terminal_size: Rect) -> LayoutAreas {
        let layout_type = self.determine_layout_type(terminal_size);
        
        match layout_type {
            LayoutType::ThreePanel => self.calculate_three_panel_layout(terminal_size),
            LayoutType::TwoPanel => self.calculate_two_panel_layout(terminal_size),
            LayoutType::SinglePanel => self.calculate_single_panel_layout(terminal_size),
            LayoutType::Overlay => self.calculate_overlay_layout(terminal_size),
        }
    }

    /// Update layout for new terminal size
    pub fn update_for_terminal_size(&mut self, size: Rect) {
        self.terminal_size = size;
        self.current_layout = self.determine_layout_type(size);
        self.update_panel_visibility_for_layout();
    }

    /// Toggle visibility of a specific panel
    pub fn toggle_panel(&mut self, panel: PanelType) {
        match panel {
            PanelType::Sidebar => {
                self.panel_visibility.sidebar = !self.panel_visibility.sidebar;
            }
            PanelType::InfoPanel => {
                self.panel_visibility.info_panel = !self.panel_visibility.info_panel;
            }
            PanelType::StatusBar => {
                self.panel_visibility.status_bar = !self.panel_visibility.status_bar;
            }
            PanelType::MainChat => {
                // Main chat should always be visible
            }
        }
        
        // Update layout type based on new visibility
        self.current_layout = self.determine_layout_type(self.terminal_size);
    }

    /// Get list of currently visible panels
    pub fn get_visible_panels(&self) -> Vec<PanelType> {
        let mut panels = Vec::new();
        
        if self.panel_visibility.sidebar {
            panels.push(PanelType::Sidebar);
        }
        
        // Main chat is always included
        panels.push(PanelType::MainChat);
        
        if self.panel_visibility.info_panel {
            panels.push(PanelType::InfoPanel);
        }
        
        panels
    }

    /// Determine appropriate layout type based on terminal size
    fn determine_layout_type(&self, terminal_size: Rect) -> LayoutType {
        let width = terminal_size.width;
        
        if width >= self.responsive_breakpoints.large_screen {
            // Large screen: show all panels if enabled
            if self.panel_visibility.sidebar && self.panel_visibility.info_panel {
                LayoutType::ThreePanel
            } else if self.panel_visibility.sidebar || self.panel_visibility.info_panel {
                LayoutType::TwoPanel
            } else {
                LayoutType::SinglePanel
            }
        } else if width >= self.responsive_breakpoints.medium_screen {
            // Medium screen: show sidebar + main, hide info panel
            if self.panel_visibility.sidebar {
                LayoutType::TwoPanel
            } else {
                LayoutType::SinglePanel
            }
        } else {
            // Small screen: only main panel
            LayoutType::SinglePanel
        }
    }

    /// Calculate three-panel layout (sidebar + main + info)
    fn calculate_three_panel_layout(&self, terminal_size: Rect) -> LayoutAreas {
        // Reserve space for status bar
        let main_area = if self.panel_visibility.status_bar {
            Rect {
                x: terminal_size.x,
                y: terminal_size.y,
                width: terminal_size.width,
                height: terminal_size.height.saturating_sub(self.panel_sizes.status_bar_height),
            }
        } else {
            terminal_size
        };

        // Calculate panel widths as percentages
        let sidebar_width = std::cmp::min(self.panel_sizes.sidebar_width, main_area.width / 4);
        let info_width = std::cmp::min(self.panel_sizes.info_panel_width, main_area.width / 4);
        let main_width = main_area.width.saturating_sub(sidebar_width + info_width);

        // Create horizontal layout
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(sidebar_width),
                Constraint::Length(main_width),
                Constraint::Length(info_width),
            ])
            .split(main_area);

        LayoutAreas {
            sidebar: Some(horizontal_chunks[0]),
            main_chat: horizontal_chunks[1],
            info_panel: Some(horizontal_chunks[2]),
            status_bar: if self.panel_visibility.status_bar {
                Rect {
                    x: terminal_size.x,
                    y: terminal_size.y + main_area.height,
                    width: terminal_size.width,
                    height: self.panel_sizes.status_bar_height,
                }
            } else {
                Rect::default()
            },
        }
    }

    /// Calculate two-panel layout (sidebar + main OR main + info)
    fn calculate_two_panel_layout(&self, terminal_size: Rect) -> LayoutAreas {
        // Reserve space for status bar
        let main_area = if self.panel_visibility.status_bar {
            Rect {
                x: terminal_size.x,
                y: terminal_size.y,
                width: terminal_size.width,
                height: terminal_size.height.saturating_sub(self.panel_sizes.status_bar_height),
            }
        } else {
            terminal_size
        };

        let (sidebar_area, main_area, info_area) = if self.panel_visibility.sidebar {
            // Sidebar + Main layout
            let sidebar_width = std::cmp::min(
                (main_area.width * 30) / 100, // 30% of width
                self.panel_sizes.sidebar_width
            );
            let main_width = main_area.width.saturating_sub(sidebar_width);

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(sidebar_width),
                    Constraint::Length(main_width),
                ])
                .split(main_area);

            (Some(horizontal_chunks[0]), horizontal_chunks[1], None)
        } else if self.panel_visibility.info_panel {
            // Main + Info layout
            let info_width = std::cmp::min(
                (main_area.width * 30) / 100, // 30% of width
                self.panel_sizes.info_panel_width
            );
            let main_width = main_area.width.saturating_sub(info_width);

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(main_width),
                    Constraint::Length(info_width),
                ])
                .split(main_area);

            (None, horizontal_chunks[0], Some(horizontal_chunks[1]))
        } else {
            // Fallback to single panel
            (None, main_area, None)
        };

        LayoutAreas {
            sidebar: sidebar_area,
            main_chat: main_area,
            info_panel: info_area,
            status_bar: if self.panel_visibility.status_bar {
                Rect {
                    x: terminal_size.x,
                    y: terminal_size.y + main_area.height,
                    width: terminal_size.width,
                    height: self.panel_sizes.status_bar_height,
                }
            } else {
                Rect::default()
            },
        }
    }

    /// Calculate single-panel layout (main only)
    fn calculate_single_panel_layout(&self, terminal_size: Rect) -> LayoutAreas {
        let main_area = if self.panel_visibility.status_bar {
            Rect {
                x: terminal_size.x,
                y: terminal_size.y,
                width: terminal_size.width,
                height: terminal_size.height.saturating_sub(self.panel_sizes.status_bar_height),
            }
        } else {
            terminal_size
        };

        LayoutAreas {
            sidebar: None,
            main_chat: main_area,
            info_panel: None,
            status_bar: if self.panel_visibility.status_bar {
                Rect {
                    x: terminal_size.x,
                    y: terminal_size.y + main_area.height,
                    width: terminal_size.width,
                    height: self.panel_sizes.status_bar_height,
                }
            } else {
                Rect::default()
            },
        }
    }

    /// Calculate overlay layout (main + floating panels)
    fn calculate_overlay_layout(&self, terminal_size: Rect) -> LayoutAreas {
        // For now, overlay is the same as single panel
        // In the future, this could support floating panels
        self.calculate_single_panel_layout(terminal_size)
    }

    /// Update panel visibility based on current layout type
    fn update_panel_visibility_for_layout(&mut self) {
        match self.current_layout {
            LayoutType::ThreePanel => {
                // All panels can be visible
            }
            LayoutType::TwoPanel => {
                // Hide info panel on medium screens if both sidebar and info are enabled
                if self.terminal_size.width < self.responsive_breakpoints.large_screen {
                    self.panel_visibility.info_panel = false;
                }
            }
            LayoutType::SinglePanel => {
                // Hide sidebar and info panel on small screens
                if self.terminal_size.width < self.responsive_breakpoints.medium_screen {
                    self.panel_visibility.sidebar = false;
                    self.panel_visibility.info_panel = false;
                }
            }
            LayoutType::Overlay => {
                // Similar to single panel for now
            }
        }
    }

    /// Get minimum required width for current layout
    pub fn get_minimum_width(&self) -> u16 {
        match self.current_layout {
            LayoutType::ThreePanel => {
                self.panel_sizes.sidebar_width + 40 + self.panel_sizes.info_panel_width
            }
            LayoutType::TwoPanel => {
                std::cmp::max(self.panel_sizes.sidebar_width, self.panel_sizes.info_panel_width) + 40
            }
            LayoutType::SinglePanel | LayoutType::Overlay => 40,
        }
    }

    /// Check if layout needs to be recalculated
    pub fn needs_recalculation(&self, new_size: Rect) -> bool {
        self.terminal_size != new_size || 
        self.determine_layout_type(new_size) != self.current_layout
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}