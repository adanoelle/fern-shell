//! # Application State
//!
//! Central state management following the Elm architecture pattern.
//!
//! The application state is immutable and updates are performed through
//! the `update` function which returns a new state.

use super::action::{Action, ConfigSummary};
use super::log::LogBuffer;
use super::service::KnownService;
use fern_core::state::{ServiceInfo, ServiceRegistry, ServiceStatus};
use fern_core::FernPaths;

/// Which panel has focus in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelFocus {
    /// Services panel (top-left).
    #[default]
    Services,
    /// Logs panel (right).
    Logs,
    /// Config panel (bottom-left).
    Config,
}

impl PanelFocus {
    /// Returns the next panel in focus order.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Services => Self::Logs,
            Self::Logs => Self::Config,
            Self::Config => Self::Services,
        }
    }

    /// Returns the previous panel in focus order.
    #[must_use]
    pub const fn prev(self) -> Self {
        match self {
            Self::Services => Self::Config,
            Self::Logs => Self::Services,
            Self::Config => Self::Logs,
        }
    }
}

/// Central application state.
///
/// This struct holds all the state needed by the TUI dashboard.
/// Updates are performed through the `update` method.
#[derive(Debug)]
pub struct AppState {
    /// Paths for state and config files.
    pub paths: FernPaths,

    /// Registry of all services and their status.
    pub services: ServiceRegistry,

    /// Log entry buffer.
    pub logs: LogBuffer,

    /// Current configuration summary.
    pub config: ConfigSummary,

    /// Currently focused panel (TUI).
    pub focus: PanelFocus,

    /// Selected service index in the services panel.
    pub selected_service: usize,

    /// Whether the help panel is visible.
    pub show_help: bool,

    /// Last error message, if any.
    pub last_error: Option<String>,

    /// Whether the application should quit.
    pub should_quit: bool,
}

impl AppState {
    /// Creates a new application state with default values.
    #[must_use]
    pub fn new() -> Self {
        let mut services = ServiceRegistry::new();

        // Initialize with all known services in stopped state
        for service in KnownService::all() {
            services.upsert(ServiceInfo::stopped(service.name()));
        }

        Self {
            paths: FernPaths::new(),
            services,
            logs: LogBuffer::default(),
            config: ConfigSummary::default(),
            focus: PanelFocus::default(),
            selected_service: 0,
            show_help: false,
            last_error: None,
            should_quit: false,
        }
    }

    /// Creates a new application state with custom paths.
    #[must_use]
    pub fn with_paths(paths: FernPaths) -> Self {
        let mut state = Self::new();
        state.paths = paths;
        state
    }

    /// Updates the state based on an action.
    ///
    /// This is the central update function following the Elm architecture.
    /// All state changes should flow through this method.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::ServiceStateChanged { name: _, info } => {
                self.services.upsert(info);
                self.last_error = None;
            }

            Action::StartService(service) => {
                // Mark service as starting
                let info = ServiceInfo::new(service.name(), ServiceStatus::Starting);
                self.services.upsert(info);
            }

            Action::StopService(service) => {
                // Mark service as stopping
                let info = ServiceInfo::new(service.name(), ServiceStatus::Stopping);
                self.services.upsert(info);
            }

            Action::RestartService(service) => {
                // Mark service as stopping (will be started after)
                let info = ServiceInfo::new(service.name(), ServiceStatus::Stopping);
                self.services.upsert(info);
            }

            Action::LogReceived(entry) => {
                self.logs.push(entry);
            }

            Action::LogsSync(entries) => {
                // Sync logs from file - add only new entries
                self.logs.sync(entries);
            }

            Action::ScrollLogs(offset) => {
                self.logs.scroll(offset);
            }

            Action::ClearLogs => {
                self.logs.clear();
            }

            Action::SetLogFilter(filter) => {
                self.logs.set_filter(filter);
            }

            Action::ConfigChanged(summary) => {
                self.config = summary;
            }

            Action::ReloadShell => {
                // This action is handled by the adapter, state doesn't change
            }

            Action::ApplyTheme { .. } => {
                // This action is handled by the adapter, state doesn't change
            }

            Action::FocusNext => {
                self.focus = self.focus.next();
            }

            Action::FocusPrev => {
                self.focus = self.focus.prev();
            }

            Action::ToggleHelp => {
                self.show_help = !self.show_help;
            }

            Action::SelectNext => {
                match self.focus {
                    PanelFocus::Services => {
                        let count = KnownService::all().len();
                        if count > 0 {
                            self.selected_service = (self.selected_service + 1) % count;
                        }
                    }
                    PanelFocus::Logs => {
                        self.logs.select_next();
                    }
                    PanelFocus::Config => {
                        // Config panel doesn't have selection
                    }
                }
            }

            Action::SelectPrev => {
                match self.focus {
                    PanelFocus::Services => {
                        let count = KnownService::all().len();
                        if count > 0 {
                            self.selected_service = self
                                .selected_service
                                .checked_sub(1)
                                .unwrap_or(count.saturating_sub(1));
                        }
                    }
                    PanelFocus::Logs => {
                        self.logs.select_prev();
                    }
                    PanelFocus::Config => {
                        // Config panel doesn't have selection
                    }
                }
            }

            Action::Tick => {
                // Tick actions could update uptime counters, etc.
            }

            Action::Error(message) => {
                self.last_error = Some(message);
            }

            Action::Quit => {
                self.should_quit = true;
            }
        }
    }

    /// Returns the currently selected service.
    #[must_use]
    pub fn selected_service(&self) -> Option<KnownService> {
        KnownService::all().get(self.selected_service).copied()
    }

    /// Returns the info for a known service.
    #[must_use]
    pub fn service_info(&self, service: KnownService) -> Option<&ServiceInfo> {
        self.services.find(service.name())
    }

    /// Returns `true` if a service is running.
    #[must_use]
    pub fn is_service_running(&self, service: KnownService) -> bool {
        self.service_info(service)
            .map_or(false, ServiceInfo::is_running)
    }

    /// Returns the count of running services.
    #[must_use]
    pub fn running_service_count(&self) -> usize {
        self.services.running().count()
    }

    /// Returns the count of unhealthy services.
    #[must_use]
    pub fn unhealthy_service_count(&self) -> usize {
        self.services.unhealthy().count()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::log::LogEntry;

    #[test]
    fn panel_focus_cycling() {
        let mut focus = PanelFocus::Services;

        focus = focus.next();
        assert_eq!(focus, PanelFocus::Logs);

        focus = focus.next();
        assert_eq!(focus, PanelFocus::Config);

        focus = focus.next();
        assert_eq!(focus, PanelFocus::Services);

        focus = focus.prev();
        assert_eq!(focus, PanelFocus::Config);
    }

    #[test]
    fn app_state_update_focus() {
        let mut state = AppState::new();

        assert_eq!(state.focus, PanelFocus::Services);

        state.update(Action::FocusNext);
        assert_eq!(state.focus, PanelFocus::Logs);

        state.update(Action::FocusPrev);
        assert_eq!(state.focus, PanelFocus::Services);
    }

    #[test]
    fn app_state_update_logs() {
        let mut state = AppState::new();

        assert!(state.logs.is_empty());

        state.update(Action::LogReceived(LogEntry::info("test", "hello")));
        assert_eq!(state.logs.len(), 1);

        state.update(Action::ClearLogs);
        assert!(state.logs.is_empty());
    }

    #[test]
    fn app_state_quit() {
        let mut state = AppState::new();

        assert!(!state.should_quit);

        state.update(Action::Quit);
        assert!(state.should_quit);
    }

    #[test]
    fn app_state_toggle_help() {
        let mut state = AppState::new();

        assert!(!state.show_help);

        state.update(Action::ToggleHelp);
        assert!(state.show_help);

        state.update(Action::ToggleHelp);
        assert!(!state.show_help);
    }
}
