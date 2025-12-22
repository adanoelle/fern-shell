//! # Help Panel Widget
//!
//! Displays keyboard shortcuts and help information.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

/// Help panel widget (overlay).
pub struct HelpPanel;

impl HelpPanel {
    /// Creates a new help panel.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Creates the help content.
    fn help_lines() -> Vec<Line<'static>> {
        let key_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);
        let section_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        vec![
            Line::from(vec![Span::styled("Navigation", section_style)]),
            Line::from(vec![
                Span::styled("Tab      ", key_style),
                Span::styled("Switch panel focus", desc_style),
            ]),
            Line::from(vec![
                Span::styled("j/↓      ", key_style),
                Span::styled("Select next item", desc_style),
            ]),
            Line::from(vec![
                Span::styled("k/↑      ", key_style),
                Span::styled("Select previous item", desc_style),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("Services", section_style)]),
            Line::from(vec![
                Span::styled("s        ", key_style),
                Span::styled("Start selected service", desc_style),
            ]),
            Line::from(vec![
                Span::styled("t        ", key_style),
                Span::styled("Stop selected service", desc_style),
            ]),
            Line::from(vec![
                Span::styled("R        ", key_style),
                Span::styled("Restart selected service", desc_style),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("Logs", section_style)]),
            Line::from(vec![
                Span::styled("/        ", key_style),
                Span::styled("Filter logs", desc_style),
            ]),
            Line::from(vec![
                Span::styled("c        ", key_style),
                Span::styled("Clear logs", desc_style),
            ]),
            Line::from(vec![
                Span::styled("G        ", key_style),
                Span::styled("Jump to end", desc_style),
            ]),
            Line::from(vec![
                Span::styled("g        ", key_style),
                Span::styled("Jump to start", desc_style),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("General", section_style)]),
            Line::from(vec![
                Span::styled("r        ", key_style),
                Span::styled("Reload shell config", desc_style),
            ]),
            Line::from(vec![
                Span::styled("?        ", key_style),
                Span::styled("Toggle this help", desc_style),
            ]),
            Line::from(vec![
                Span::styled("q        ", key_style),
                Span::styled("Quit", desc_style),
            ]),
        ]
    }
}

impl Default for HelpPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HelpPanel {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area first (overlay)
        Clear.render(area, buf);

        let block = Block::default()
            .title(" Help (press ? to close) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let lines = Self::help_lines();
        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);

        Widget::render(paragraph, area, buf);
    }
}
