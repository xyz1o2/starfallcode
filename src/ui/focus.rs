use crate::ui::types::{PanelType, FocusIndicators};
use ratatui::style::{Color, Style};

pub struct FocusManager {
    pub current_focus: PanelType,
    pub focus_history: Vec<PanelType>,
    pub focus_indicators: FocusIndicators,
    pub available_panels: Vec<PanelType>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            current_focus: PanelType::MainChat,
            focus_history: vec![PanelType::MainChat],
            focus_indicators: FocusIndicators::default(),
            available_panels: vec![
                PanelType::Sidebar,
                PanelType::MainChat,
                PanelType::InfoPanel,
            ],
        }
    }

    /// Set focus to a specific panel
    pub fn set_focus(&mut self, panel: PanelType) {
        if self.available_panels.contains(&panel) {
            // Add current focus to history if it's different
            if self.current_focus != panel {
                self.focus_history.push(self.current_focus.clone());
                
                // Limit history size
                if self.focus_history.len() > 10 {
                    self.focus_history.remove(0);
                }
            }
            
            self.current_focus = panel;
        }
    }

    /// Cycle focus to the next available panel
    pub fn cycle_focus(&mut self, available_panels: &[PanelType]) {
        self.available_panels = available_panels.to_vec();
        
        if let Some(current_index) = available_panels.iter().position(|p| *p == self.current_focus) {
            let next_index = (current_index + 1) % available_panels.len();
            self.set_focus(available_panels[next_index].clone());
        } else if !available_panels.is_empty() {
            self.set_focus(available_panels[0].clone());
        }
    }

    /// Cycle focus to the previous available panel
    pub fn cycle_focus_backward(&mut self, available_panels: &[PanelType]) {
        self.available_panels = available_panels.to_vec();
        
        if let Some(current_index) = available_panels.iter().position(|p| *p == self.current_focus) {
            let prev_index = if current_index == 0 {
                available_panels.len() - 1
            } else {
                current_index - 1
            };
            self.set_focus(available_panels[prev_index].clone());
        } else if !available_panels.is_empty() {
            self.set_focus(available_panels[available_panels.len() - 1].clone());
        }
    }

    /// Go back to the previous panel in history
    pub fn focus_previous(&mut self) {
        if let Some(previous_panel) = self.focus_history.pop() {
            self.current_focus = previous_panel;
        }
    }

    /// Get the style for a panel based on focus state
    pub fn get_focus_style(&self, panel: PanelType) -> Style {
        if panel == self.current_focus {
            self.focus_indicators.active_border_style
        } else {
            self.focus_indicators.inactive_border_style
        }
    }

    /// Check if a panel is currently focused
    pub fn is_focused(&self, panel: &PanelType) -> bool {
        self.current_focus == *panel
    }

    /// Update available panels based on layout
    pub fn update_available_panels(&mut self, panels: Vec<PanelType>) {
        self.available_panels = panels;
        
        // If current focus is not available, switch to the first available panel
        if !self.available_panels.contains(&self.current_focus) && !self.available_panels.is_empty() {
            self.set_focus(self.available_panels[0].clone());
        }
    }

    /// Get the currently focused panel
    pub fn get_focused_panel(&self) -> PanelType {
        self.current_focus.clone()
    }

    /// Update focus indicators with theme colors
    pub fn update_theme_colors(&mut self, active_color: Color, inactive_color: Color, highlight_color: Color) {
        self.focus_indicators = FocusIndicators {
            active_border_style: Style::default().fg(active_color),
            inactive_border_style: Style::default().fg(inactive_color),
            focus_highlight: highlight_color,
        };
    }

    /// Get focus indicator for highlighting focused elements
    pub fn get_highlight_color(&self) -> Color {
        self.focus_indicators.focus_highlight
    }

    /// Reset focus to default state
    pub fn reset(&mut self) {
        self.current_focus = PanelType::MainChat;
        self.focus_history.clear();
        self.focus_history.push(PanelType::MainChat);
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}