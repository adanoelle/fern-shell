//! Convenient re-exports for frond applications.
//!
//! ```ignore
//! use frond::prelude::*;
//! ```

pub use crate::application::Application;
pub use crate::cmd::Cmd;
pub use crate::error::{Error, Result};
pub use crate::run;
pub use crate::sub::Sub;

// Re-export commonly used ratatui types
pub use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
pub use ratatui::style::{Color, Modifier, Style, Stylize};
pub use ratatui::text::{Line, Span, Text};
pub use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap};
pub use ratatui::Frame;

// Re-export commonly used crossterm types
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Re-export time types commonly used with subscriptions
pub use std::time::Duration;
