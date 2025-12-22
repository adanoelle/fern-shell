//! # TUI Event Handling
//!
//! Handles keyboard and terminal events.

use crate::domain::{Action, KnownService, PanelFocus};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Polls for the next event with a timeout.
///
/// Returns `None` if no event is available within the timeout.
pub fn poll_event(timeout: Duration) -> Option<Event> {
    if event::poll(timeout).ok()? {
        event::read().ok()
    } else {
        None
    }
}

/// Converts a key event to an action based on current focus.
pub fn key_to_action(key: KeyEvent, focus: PanelFocus, selected_service: usize) -> Option<Action> {
    // Global keys
    match key.code {
        KeyCode::Char('q') => return Some(Action::Quit),
        KeyCode::Char('?') => return Some(Action::ToggleHelp),
        KeyCode::Tab => return Some(Action::FocusNext),
        KeyCode::BackTab => return Some(Action::FocusPrev),
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::NONE) => {
            return Some(Action::ReloadShell);
        }
        _ => {}
    }

    // Panel-specific keys
    match focus {
        PanelFocus::Services => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::SelectNext),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::SelectPrev),
            KeyCode::Char('s') => {
                let service = KnownService::all().get(selected_service).copied()?;
                Some(Action::StartService(service))
            }
            KeyCode::Char('t') => {
                let service = KnownService::all().get(selected_service).copied()?;
                Some(Action::StopService(service))
            }
            KeyCode::Char('R') => {
                let service = KnownService::all().get(selected_service).copied()?;
                Some(Action::RestartService(service))
            }
            _ => None,
        },
        PanelFocus::Logs => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::ScrollLogs(1)),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::ScrollLogs(-1)),
            KeyCode::Char('G') => Some(Action::ScrollLogs(i32::MAX)),
            KeyCode::Char('g') => Some(Action::ScrollLogs(i32::MIN)),
            KeyCode::Char('c') => Some(Action::ClearLogs),
            KeyCode::PageDown => Some(Action::ScrollLogs(10)),
            KeyCode::PageUp => Some(Action::ScrollLogs(-10)),
            _ => None,
        },
        PanelFocus::Config => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Some(Action::SelectNext),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::SelectPrev),
            _ => None,
        },
    }
}
