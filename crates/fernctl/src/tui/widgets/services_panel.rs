//! # Services Panel Widget
//!
//! Displays the status of all known services.

use crate::domain::{AppState, KnownService, PanelFocus};
use fern_core::state::ServiceStatus;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

/// Services panel widget.
pub struct ServicesPanel<'a> {
    state: &'a AppState,
}

impl<'a> ServicesPanel<'a> {
    /// Creates a new services panel.
    #[must_use]
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    /// Returns the border style based on focus.
    fn border_style(&self) -> Style {
        if self.state.focus == PanelFocus::Services {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    }

    /// Creates list items for services.
    fn service_items(&self) -> Vec<ListItem<'a>> {
        KnownService::all()
            .iter()
            .enumerate()
            .map(|(idx, service)| {
                let info = self.state.service_info(*service);
                let is_selected = idx == self.state.selected_service
                    && self.state.focus == PanelFocus::Services;

                let (indicator, indicator_color) = match info.map(|i| &i.status) {
                    Some(ServiceStatus::Running) => ("●", Color::Green),
                    Some(ServiceStatus::Starting) => ("◐", Color::Yellow),
                    Some(ServiceStatus::Stopping) => ("◐", Color::Yellow),
                    Some(ServiceStatus::Failed(_)) => ("✗", Color::Red),
                    Some(ServiceStatus::Stopped) | Some(ServiceStatus::Disabled) | None => {
                        ("○", Color::DarkGray)
                    }
                };

                let status_text = match info.map(|i| &i.status) {
                    Some(ServiceStatus::Running) => "running",
                    Some(ServiceStatus::Starting) => "starting",
                    Some(ServiceStatus::Stopping) => "stopping",
                    Some(ServiceStatus::Failed(msg)) => msg.as_str(),
                    Some(ServiceStatus::Stopped) => "stopped",
                    Some(ServiceStatus::Disabled) => "disabled",
                    None => "unknown",
                };

                let line = Line::from(vec![
                    Span::styled(
                        format!("{} ", indicator),
                        Style::default().fg(indicator_color),
                    ),
                    Span::styled(
                        format!("{:12}", service.display_name()),
                        Style::default().add_modifier(if is_selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                    ),
                    Span::styled(
                        status_text,
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                let style = if is_selected {
                    Style::default()
                        .bg(Color::Rgb(94, 234, 212))
                        .fg(Color::Black)
                } else {
                    Style::default()
                };

                ListItem::new(line).style(style)
            })
            .collect()
    }
}

impl Widget for ServicesPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Services ")
            .borders(Borders::ALL)
            .border_style(self.border_style());

        let items = self.service_items();
        let list = List::new(items).block(block);

        Widget::render(list, area, buf);
    }
}
