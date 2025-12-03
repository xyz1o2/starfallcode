/// Vibe Coding 工作流状态面板 UI
/// 显示当前阶段、进度和项目信息

use ratatui::{
    Frame,
    layout::{Rect, Constraint, Direction, Layout, Alignment},
    style::{Style, Color, Modifier},
    widgets::{Block, Borders, Paragraph, Gauge, List, ListItem, Wrap},
};
use crate::core::vibe_coding::{VibeStatus, VibeStage};

pub struct VibePanel;

impl VibePanel {
    pub fn render(frame: &mut Frame, area: Rect, status: &VibeStatus) {
        // 布局：顶部标题 + 主要内容
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 标题
                Constraint::Min(5),     // 主要内容
            ])
            .split(area);

        // 渲染标题
        Self::render_title(frame, main_layout[0], status);

        // 渲染内容
        Self::render_content(frame, main_layout[1], status);
    }

    fn render_title(frame: &mut Frame, area: Rect, status: &VibeStatus) {
        let title = format!(" Vibe Coding - {} ", status.stage_name);
        let stage_indicator = format!(
            " [{}] ",
            match status.stage {
                VibeStage::Conceptualization => "1/5",
                VibeStage::Generation => "2/5",
                VibeStage::Iteration => "3/5",
                VibeStage::Validation => "4/5",
                VibeStage::Deployment => "5/5",
            }
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let text = Paragraph::new(stage_indicator)
            .block(block)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);

        frame.render_widget(text, area);
    }

    fn render_content(frame: &mut Frame, area: Rect, status: &VibeStatus) {
        // 分割为主要信息和进度条
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // 阶段描述
                Constraint::Length(3),  // 进度条
                Constraint::Min(5),     // 统计信息
            ])
            .split(area);

        // 1. 阶段描述
        let description = Paragraph::new(status.stage_description.as_str())
            .block(Block::default().title(" 阶段描述 ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        frame.render_widget(description, content_layout[0]);

        // 2. 进度条
        let progress = match status.stage {
            VibeStage::Conceptualization => 20.0,
            VibeStage::Generation => 40.0,
            VibeStage::Iteration => 60.0,
            VibeStage::Validation => 80.0,
            VibeStage::Deployment => 100.0,
        };

        let gauge = Gauge::default()
            .block(Block::default().title(" 工作流进度 ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Magenta))
            .percent(progress as u16);
        frame.render_widget(gauge, content_layout[1]);

        // 3. 统计信息
        Self::render_stats(frame, content_layout[2], status);
    }

    fn render_stats(frame: &mut Frame, area: Rect, status: &VibeStatus) {
        let stats_items = vec![
            ListItem::new(format!("📊 总变更数: {}", status.changes_count)),
            ListItem::new(format!("✅ 已完成: {}", status.completed_changes)),
            ListItem::new(format!("⏳ 进行中: {}", status.changes_count.saturating_sub(status.completed_changes))),
        ];

        let stats = List::new(stats_items)
            .block(
                Block::default()
                    .title(" 项目统计 ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
            );

        frame.render_widget(stats, area);
    }
}

pub struct StageTimeline;

impl StageTimeline {
    pub fn render_mini(frame: &mut Frame, area: Rect, current_stage: VibeStage) {
        let stages = vec![
            ("概念化", VibeStage::Conceptualization),
            ("生成", VibeStage::Generation),
            ("迭代", VibeStage::Iteration),
            ("验证", VibeStage::Validation),
            ("部署", VibeStage::Deployment),
        ];

        let items: Vec<ListItem> = stages
            .into_iter()
            .map(|(name, stage)| {
                let symbol = if stage == current_stage { "▶" } else { "○" };
                let style = if stage == current_stage {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if (stage as u8) < (current_stage as u8) {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Gray)
                };

                ListItem::new(format!("{} {}", symbol, name)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" 阶段时间线 ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray))
            );

        frame.render_widget(list, area);
    }
}
