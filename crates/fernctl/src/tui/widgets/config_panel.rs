//! # Config Panel Widget
//!
//! Displays current configuration overview.

use crate::domain::{AppState, PanelFocus};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Config panel widget.
pub struct ConfigPanel<'a> {
    state: &'a AppState,
}

impl<'a> ConfigPanel<'a> {
    /// Creates a new config panel.
    #[must_use]
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// Returns the border style based on focus.
    fn border_style(&self) -> Style {
        if self.state.focus == PanelFocus::Config {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }

    /// Creates the config display lines.
    fn config_lines(&self) -> Vec<Line<'a>> {
        let config = &self.state.config;

        let mut lines = vec![];

        // Theme
        if let Some(ref theme) = config.theme {
            lines.push(Line::from(vec![
                Span::styled("Theme: ", Style::default().fg(Color::DarkGray)),
                Span::styled(theme.clone(), Style::default().fg(Color::White)),
            ]));
        }

        // Variant
        if let Some(ref variant) = config.variant {
            lines.push(Line::from(vec![
                Span::styled("Variant: ", Style::default().fg(Color::DarkGray)),
                Span::styled(variant.clone(), Style::default().fg(Color::White)),
            ]));
        }

        // Accent color
        if let Some(ref accent) = config.accent_color {
            lines.push(Line::from(vec![
                Span::styled("Accent: ", Style::default().fg(Color::DarkGray)),
                Span::styled(accent.clone(), Style::default().fg(Color::Magenta)),
            ]));
        }

        // Bar position and height
        let bar_info = match (&config.bar_position, config.bar_height) {
            (Some(pos), Some(height)) => format!("{}, {}px", pos, height),
            (Some(pos), None) => pos.clone(),
            (None, Some(height)) => format!("{}px", height),
            (None, None) => "default".to_string(),
        };

        lines.push(Line::from(vec![
            Span::styled("Bar: ", Style::default().fg(Color::DarkGray)),
            Span::styled(bar_info, Style::default().fg(Color::White)),
        ]));

        // If no config loaded, show placeholder
        if lines.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No config loaded",
                Style::default().fg(Color::DarkGray),
            )]));
        }

        lines
    }
}

impl Widget for ConfigPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Config ")
            .borders(Borders::ALL)
            .border_style(self.border_style());

        let lines = self.config_lines();
        let paragraph = Paragraph::new(lines).block(block);

        Widget::render(paragraph, area, buf);
    }
}
