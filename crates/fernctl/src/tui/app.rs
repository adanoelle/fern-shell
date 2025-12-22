//! # TUI Application
//!
//! Main TUI event loop and rendering.

use crate::adapters::{reload_shell, ServiceController, StateWatcher};
use crate::domain::{Action, AppState, KnownService};
use crate::error::{FernctlError, Result};
use crate::tui::event::{key_to_action, poll_event};
use crate::tui::layout::AppLayout;
use crate::tui::widgets::{ConfigPanel, HelpPanel, LogsPanel, ServicesPanel};
use crossterm::{
    event::Event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fern_core::FernPaths;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;

/// TUI Application.
///
/// Manages the terminal UI lifecycle, event handling, and rendering.
pub struct TuiApp {
    state: AppState,
    service_controller: ServiceController,
    state_watcher: Option<StateWatcher>,
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
}

impl TuiApp {
    /// Creates a new TUI application.
    #[must_use]
    pub fn new() -> Self {
        let paths = FernPaths::new();
        Self {
            state: AppState::with_paths(paths.clone()),
            service_controller: ServiceController::new(paths),
            state_watcher: None,
            terminal: None,
        }
    }

    /// Runs the TUI application.
    ///
    /// This is the main event loop that handles input, state updates,
    /// and rendering until the user quits.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal operations fail.
    pub fn run(&mut self) -> Result<()> {
        self.setup_terminal()?;
        self.setup_watcher()?;
        self.load_initial_state()?;

        let result = self.event_loop();

        self.restore_terminal()?;

        result
    }

    /// Sets up the terminal for TUI rendering.
    fn setup_terminal(&mut self) -> Result<()> {
        enable_raw_mode().map_err(|e| FernctlError::tui_io("enabling raw mode", e))?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)
            .map_err(|e| FernctlError::tui_io("entering alternate screen", e))?;

        let backend = CrosstermBackend::new(stdout);
        let terminal =
            Terminal::new(backend).map_err(|e| FernctlError::tui_io("creating terminal", e))?;

        self.terminal = Some(terminal);
        Ok(())
    }

    /// Restores the terminal to its original state.
    fn restore_terminal(&mut self) -> Result<()> {
        if let Some(ref mut terminal) = self.terminal {
            disable_raw_mode().map_err(|e| FernctlError::tui_io("disabling raw mode", e))?;

            execute!(terminal.backend_mut(), LeaveAlternateScreen)
                .map_err(|e| FernctlError::tui_io("leaving alternate screen", e))?;

            terminal
                .show_cursor()
                .map_err(|e| FernctlError::tui_io("showing cursor", e))?;
        }
        Ok(())
    }

    /// Sets up the state file watcher.
    fn setup_watcher(&mut self) -> Result<()> {
        let paths = self.state.paths.clone();
        let mut watcher = StateWatcher::new(paths, 100)?;
        watcher.start()?;
        self.state_watcher = Some(watcher);
        Ok(())
    }

    /// Loads initial state from files.
    fn load_initial_state(&mut self) -> Result<()> {
        // Load config
        let config_path = self.state.paths.config_json();
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let summary = crate::domain::action::ConfigSummary::from_json(&json);
                    self.state.update(Action::ConfigChanged(summary));
                }
            }
        }

        // Update service status
        for service in KnownService::all() {
            let is_running = self.service_controller.is_running(*service);
            let status = if is_running {
                fern_core::ServiceStatus::Running
            } else {
                fern_core::ServiceStatus::Stopped
            };
            let info = fern_core::ServiceInfo::new(service.name(), status);
            self.state.update(Action::ServiceStateChanged {
                name: service.name().to_string(),
                info,
            });
        }

        Ok(())
    }

    /// Main event loop.
    fn event_loop(&mut self) -> Result<()> {
        loop {
            // Render
            self.render()?;

            // Poll for events
            if let Some(event) = poll_event(Duration::from_millis(100)) {
                if let Event::Key(key) = event {
                    let action =
                        key_to_action(key, self.state.focus, self.state.selected_service);

                    if let Some(action) = action {
                        self.handle_action(action)?;
                    }
                }
            }

            // Check for state file changes
            if let Some(ref watcher) = self.state_watcher {
                while let Some(change) = watcher.try_recv() {
                    if let Some(action) = StateWatcher::to_action(change) {
                        self.state.update(action);
                    }
                }
            }

            // Check for quit
            if self.state.should_quit {
                break;
            }

            // Reap any finished child processes
            self.service_controller.reap_children();
        }

        Ok(())
    }

    /// Handles an action.
    fn handle_action(&mut self, action: Action) -> Result<()> {
        match &action {
            Action::StartService(service) => {
                if let Err(e) = self.service_controller.start(*service) {
                    self.state.update(Action::Error(e.to_string()));
                    return Ok(());
                }
            }
            Action::StopService(service) => {
                if let Err(e) = self.service_controller.stop(*service) {
                    self.state.update(Action::Error(e.to_string()));
                    return Ok(());
                }
            }
            Action::RestartService(service) => {
                if let Err(e) = self.service_controller.restart(*service) {
                    self.state.update(Action::Error(e.to_string()));
                    return Ok(());
                }
            }
            Action::ReloadShell => {
                if let Err(e) = reload_shell() {
                    self.state.update(Action::Error(e.to_string()));
                    return Ok(());
                }
            }
            _ => {}
        }

        // Update state
        self.state.update(action);
        Ok(())
    }

    /// Renders the TUI.
    fn render(&mut self) -> Result<()> {
        // Extract state needed for rendering to avoid borrow issues
        let state = &self.state;

        let terminal = self
            .terminal
            .as_mut()
            .ok_or_else(|| FernctlError::tui("terminal not initialized"))?;

        terminal
            .draw(|frame| {
                render_frame(frame, state);
            })
            .map_err(|e| FernctlError::tui_io("drawing frame", e))?;

        Ok(())
    }

    /// Returns a reference to the application state.
    #[must_use]
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Returns a mutable reference to the application state.
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders a single frame.
fn render_frame(frame: &mut Frame, state: &AppState) {
    let layout = AppLayout::new(frame.area());

    // Header
    render_header(frame, layout.header, state);

    // Panels
    frame.render_widget(ServicesPanel::new(state), layout.services);
    frame.render_widget(LogsPanel::new(state), layout.logs);
    frame.render_widget(ConfigPanel::new(state), layout.config);

    // Footer
    render_footer(frame, layout.footer);

    // Help overlay
    if state.show_help {
        let help_area = centered_rect(60, 70, frame.area());
        frame.render_widget(HelpPanel::new(), help_area);
    }

    // Error message
    if let Some(ref error) = state.last_error {
        let error_area = Rect::new(
            layout.footer.x,
            layout.footer.y.saturating_sub(1),
            layout.footer.width,
            1,
        );
        let error_text =
            Paragraph::new(format!("Error: {}", error)).style(Style::default().fg(Color::Red));
        frame.render_widget(error_text, error_area);
    }
}

/// Renders the header.
fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let running = state.running_service_count();
    let total = KnownService::all().len();

    let title = format!(
        " Fern Control Plane ({}/{} services running) ",
        running, total
    );

    let header = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left);

    let help_hint = Paragraph::new("[?] Help")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Right);

    // Split header area
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    frame.render_widget(header, chunks[0]);
    frame.render_widget(help_hint, chunks[1]);
}

/// Renders the footer.
fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[Tab]", Style::default().fg(Color::Cyan)),
        Span::raw(" Switch  "),
        Span::styled("[j/k]", Style::default().fg(Color::Cyan)),
        Span::raw(" Navigate  "),
        Span::styled("[s]", Style::default().fg(Color::Cyan)),
        Span::raw("tart  "),
        Span::styled("[t]", Style::default().fg(Color::Cyan)),
        Span::raw("op  "),
        Span::styled("[r]", Style::default().fg(Color::Cyan)),
        Span::raw(" Reload  "),
        Span::styled("[?]", Style::default().fg(Color::Cyan)),
        Span::raw(" Help  "),
        Span::styled("[q]", Style::default().fg(Color::Cyan)),
        Span::raw(" Quit"),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

/// Creates a centered rectangle within the given area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
