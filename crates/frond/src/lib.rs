//! # Frond - Elm Architecture for Rust TUIs
//!
//! Frond is a full implementation of The Elm Architecture (TEA) for building
//! terminal user interfaces in Rust. It provides a predictable, testable way
//! to build TUI applications.
//!
//! ## Core Concepts
//!
//! - **Model**: Your entire application state (single source of truth)
//! - **Msg**: Every possible thing that can happen
//! - **update**: Pure state transitions that return `(Model, Cmd)`
//! - **view**: Pure rendering function
//! - **subscriptions**: Declare what external events you care about
//!
//! ## Example
//!
//! ```ignore
//! use frond::prelude::*;
//!
//! struct MyApp;
//!
//! impl Application for MyApp {
//!     type Model = Counter;
//!     type Msg = Msg;
//!
//!     fn init() -> (Self::Model, Cmd<Self::Msg>) {
//!         (Counter { value: 0 }, Cmd::none())
//!     }
//!
//!     fn update(model: Self::Model, msg: Self::Msg) -> (Self::Model, Cmd<Self::Msg>) {
//!         match msg {
//!             Msg::Increment => (Counter { value: model.value + 1 }, Cmd::none()),
//!             Msg::Decrement => (Counter { value: model.value - 1 }, Cmd::none()),
//!             Msg::Quit => (model.with_should_quit(true), Cmd::none()),
//!         }
//!     }
//!
//!     fn view(model: &Self::Model, frame: &mut Frame) {
//!         // Render your UI here
//!     }
//!
//!     fn subscriptions(_model: &Self::Model) -> Sub<Self::Msg> {
//!         Sub::on_key(|key| match key.code {
//!             KeyCode::Char('q') => Some(Msg::Quit),
//!             KeyCode::Up => Some(Msg::Increment),
//!             KeyCode::Down => Some(Msg::Decrement),
//!             _ => None,
//!         })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     frond::run::<MyApp>().await
//! }
//! ```

mod application;
mod cmd;
mod error;
mod runtime;
mod sub;
mod terminal;

pub mod prelude;

pub use application::Application;
pub use cmd::Cmd;
pub use error::{Error, Result};
pub use runtime::run;
pub use sub::Sub;

// Re-export ratatui types that users will need
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
pub use ratatui::Frame;
