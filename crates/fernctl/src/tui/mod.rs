//! # Terminal UI
//!
//! Interactive TUI dashboard for Fern Shell.
//!
//! ## Layout
//!
//! ```text
//! ┌─ Fern Control Plane ──────────────────────────────────────────────────┐
//! │                                                              [?] Help │
//! ├───────────────────────┬───────────────────────────────────────────────┤
//! │  Services             │  Logs                                         │
//! │  ─────────────────    │  ───────────────────────────────────────────  │
//! │  ● OBS     connected  │  12:34:56 [INFO] Obs: Connected to OBS        │
//! │  ● Shell   running    │  12:34:57 [INFO] Workspaces: Switch to ws 2   │
//! │  ○ Theme   idle       │  12:35:01 [WARN] Workspaces: Icon not found   │
//! │                       │  12:35:15 [INFO] ConfigLoader: Loaded config  │
//! │  ─────────────────    │                                               │
//! │  [s]tart [t]op [r]est │                                               │
//! ├───────────────────────┤                                               │
//! │  Config               │                                               │
//! │  ─────────────────    │                                               │
//! │  Theme: catppuccin    │                                               │
//! │  Variant: mocha       │                                               │
//! │  Bar: top, 40px       │                                               │
//! │                       │                                               │
//! │  [e]dit [R]eload      │                                               │
//! ├───────────────────────┴───────────────────────────────────────────────┤
//! │  [Tab] Switch panel  [q] Quit  [r] Reload  [?] Help                   │
//! └───────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Modules
//!
//! - [`app`] - Main TUI application loop
//! - [`event`] - Event handling
//! - [`layout`] - Panel arrangement
//! - [`widgets`] - UI components

pub mod app;
pub mod event;
pub mod layout;
pub mod widgets;

pub use app::TuiApp;
