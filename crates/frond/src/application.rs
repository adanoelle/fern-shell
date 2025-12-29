//! The Application trait - the core abstraction for Elm-style applications.

use ratatui::Frame;

use crate::{Cmd, Sub};

/// The core trait for building Elm-style applications.
///
/// Implement this trait to define your application's behavior:
/// - `Model`: Your application state
/// - `Msg`: All possible messages/events
/// - `init()`: Initial state and startup commands
/// - `update()`: Pure state transitions
/// - `view()`: Pure rendering
/// - `subscriptions()`: External event sources
pub trait Application: Sized + 'static {
    /// The application's state type.
    ///
    /// This should contain all data your application needs to render and update.
    /// It should be `Send` to allow async command execution.
    type Model: Send + 'static;

    /// The message type representing all possible events.
    ///
    /// Use an enum to represent every action that can occur in your app.
    /// Must be `Clone` for subscriptions that can fire multiple times.
    type Msg: Clone + Send + 'static;

    /// Initialize the application.
    ///
    /// Returns the initial model and any commands to run at startup
    /// (e.g., loading data from disk, fetching from network).
    fn init() -> (Self::Model, Cmd<Self::Msg>);

    /// Update the model in response to a message.
    ///
    /// This is the heart of your application logic. It should be a pure function:
    /// - No side effects (I/O, network, etc.)
    /// - Same input always produces same output
    /// - Side effects are described as `Cmd` values, executed by the runtime
    ///
    /// # Arguments
    /// * `model` - The current application state (owned for modification)
    /// * `msg` - The message/event to handle
    ///
    /// # Returns
    /// A tuple of (new_model, commands_to_execute)
    fn update(model: Self::Model, msg: Self::Msg) -> (Self::Model, Cmd<Self::Msg>);

    /// Render the current state to the terminal.
    ///
    /// This should be a pure function that only reads from the model
    /// and writes to the frame. No side effects.
    ///
    /// # Arguments
    /// * `model` - The current application state (borrowed)
    /// * `frame` - The ratatui frame to render to
    fn view(model: &Self::Model, frame: &mut Frame);

    /// Declare what external events the application cares about.
    ///
    /// Subscriptions are recalculated after every update. Return `Sub::none()`
    /// if no external events are needed, or use `Sub::batch()` to combine multiple.
    ///
    /// Common subscriptions:
    /// - `Sub::on_key()` - Keyboard input
    /// - `Sub::every()` - Timer/interval
    /// - `Sub::on_file_change()` - File system watching
    ///
    /// # Arguments
    /// * `model` - The current application state (used to conditionally subscribe)
    fn subscriptions(model: &Self::Model) -> Sub<Self::Msg>;

    /// Called when the application is about to shut down.
    ///
    /// Override this to perform cleanup, save state, etc.
    /// The default implementation does nothing.
    fn on_shutdown(_model: &Self::Model) {}

    /// Check if the application should quit.
    ///
    /// Override this to provide custom quit logic. The default always returns false,
    /// meaning the app runs until explicitly terminated.
    ///
    /// Typically you'd add a `should_quit: bool` field to your model and check it here.
    fn should_quit(_model: &Self::Model) -> bool {
        false
    }
}
