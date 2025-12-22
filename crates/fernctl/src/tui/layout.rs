//! # TUI Layout
//!
//! Defines the panel arrangement for the TUI dashboard.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Layout areas for the TUI dashboard.
#[derive(Debug, Clone)]
pub struct AppLayout {
    /// Header area (title bar).
    pub header: Rect,
    /// Left column (services + config).
    pub left: Rect,
    /// Services panel area.
    pub services: Rect,
    /// Config panel area.
    pub config: Rect,
    /// Logs panel area (right side).
    pub logs: Rect,
    /// Footer area (keybindings).
    pub footer: Rect,
}

impl AppLayout {
    /// Creates the layout for the given terminal size.
    #[must_use]
    pub fn new(area: Rect) -> Self {
        // Main vertical split: header, body, footer
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Header
                Constraint::Min(10),    // Body
                Constraint::Length(1),  // Footer
            ])
            .split(area);

        let header = vertical[0];
        let body = vertical[1];
        let footer = vertical[2];

        // Body horizontal split: left panel, logs panel
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Left column
                Constraint::Percentage(70), // Logs
            ])
            .split(body);

        let left = horizontal[0];
        let logs = horizontal[1];

        // Left column vertical split: services, config
        let left_vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Services
                Constraint::Percentage(40), // Config
            ])
            .split(left);

        let services = left_vertical[0];
        let config = left_vertical[1];

        Self {
            header,
            left,
            services,
            config,
            logs,
            footer,
        }
    }
}
