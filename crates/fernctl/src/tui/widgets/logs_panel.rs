//! # Logs Panel Widget
//!
//! Displays scrollable log entries.

use crate::domain::{AppState, LogLevel, PanelFocus};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

/// Logs panel widget.
pub struct LogsPanel<'a> {
    state: &'a AppState,
}

impl<'a> LogsPanel<'a> {
    /// Creates a new logs panel.
    #[must_use]
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// Returns the border style based on focus.
    fn border_style(&self) -> Style {
        if self.state.focus == PanelFocus::Logs {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }

    /// Returns the color for a log level.
    fn level_color(level: LogLevel) -> Color {
        match level {
            LogLevel::Trace => Color::DarkGray,
            LogLevel::Debug => Color::Cyan,
            LogLevel::Info => Color::Green,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Error => Color::Red,
        }
    }

    /// Creates list items from log entries.
    fn log_items(&self) -> Vec<ListItem<'a>> {
        let selected = self.state.logs.selected();
        let is_focused = self.state.focus == PanelFocus::Logs;

        self.state
            .logs
            .filtered_entries()
            .enumerate()
            .map(|(idx, entry)| {
                let is_selected = is_focused && selected == Some(idx);

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} ", entry.formatted_time()),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("[{:5}] ", entry.level.label()),
                        Style::default().fg(Self::level_color(entry.level)),
                    ),
                    Span::styled(
                        format!("{}: ", entry.source),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(&entry.message),
                ]);

                let style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(style)
            })
            .collect()
    }
}

impl Widget for LogsPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let filter = self.state.logs.filter();
        let title = if filter.is_empty() {
            " Logs ".to_string()
        } else {
            format!(" Logs [filter: {}] ", filter)
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(self.border_style());

        let items = self.log_items();

        // Auto-scroll to bottom if no selection
        let list = List::new(items).block(block);

        Widget::render(list, area, buf);
    }
}
