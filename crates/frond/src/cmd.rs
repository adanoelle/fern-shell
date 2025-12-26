//! Commands - descriptions of side effects to be executed by the runtime.
//!
//! Commands are how Elm-style applications handle side effects. Instead of
//! performing I/O directly in `update()`, you return `Cmd` values that describe
//! what should happen. The runtime executes them and feeds results back as messages.
//!
//! This keeps `update()` pure and testable.

use std::future::Future;
use std::pin::Pin;

/// A command representing a side effect to be executed.
///
/// Commands are executed asynchronously by the runtime. When they complete,
/// they may produce a message that gets fed back into `update()`.
pub enum Cmd<Msg> {
    /// No operation - do nothing.
    None,

    /// Execute multiple commands.
    Batch(Vec<Cmd<Msg>>),

    /// Perform an async operation that may produce a message.
    Perform(Pin<Box<dyn Future<Output = Option<Msg>> + Send + 'static>>),
}

impl<Msg> Default for Cmd<Msg> {
    fn default() -> Self {
        Self::None
    }
}

impl<Msg: Send + 'static> Cmd<Msg> {
    /// Create a command that does nothing.
    ///
    /// Use this when an update doesn't need to trigger any side effects.
    ///
    /// # Example
    /// ```ignore
    /// fn update(model: Model, msg: Msg) -> (Model, Cmd<Msg>) {
    ///     match msg {
    ///         Msg::Increment => (model.increment(), Cmd::none()),
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn none() -> Self {
        Self::None
    }

    /// Batch multiple commands together.
    ///
    /// All commands in the batch will be executed concurrently.
    ///
    /// # Example
    /// ```ignore
    /// Cmd::batch([
    ///     Cmd::perform(save_to_disk(data), Msg::Saved),
    ///     Cmd::perform(notify_user("Saving..."), |_| Msg::Noop),
    /// ])
    /// ```
    #[must_use]
    pub fn batch(cmds: impl IntoIterator<Item = Cmd<Msg>>) -> Self {
        let cmds: Vec<_> = cmds.into_iter().collect();
        if cmds.is_empty() {
            Self::None
        } else if cmds.len() == 1 {
            cmds.into_iter().next().unwrap_or(Self::None)
        } else {
            Self::Batch(cmds)
        }
    }

    /// Perform an async operation and map the successful result to a message.
    ///
    /// If the future returns `Ok(value)`, `to_msg(value)` is called to produce
    /// a message. If it returns `Err(_)`, no message is produced.
    ///
    /// # Example
    /// ```ignore
    /// // Load events from database
    /// Cmd::perform(
    ///     db.get_events(date),
    ///     Msg::EventsLoaded  // EventsLoaded(Vec<Event>)
    /// )
    /// ```
    #[must_use]
    pub fn perform<F, T, E>(future: F, to_msg: impl Fn(T) -> Msg + Send + 'static) -> Self
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
    {
        Self::Perform(Box::pin(async move {
            match future.await {
                Ok(value) => Some(to_msg(value)),
                Err(_) => None,
            }
        }))
    }

    /// Perform an async operation with separate handlers for success and failure.
    ///
    /// # Example
    /// ```ignore
    /// Cmd::perform_fallible(
    ///     db.save_event(event),
    ///     |saved| Msg::EventSaved(Ok(saved)),
    ///     |err| Msg::EventSaved(Err(err.to_string())),
    /// )
    /// ```
    #[must_use]
    pub fn perform_fallible<F, T, E>(
        future: F,
        on_ok: impl Fn(T) -> Msg + Send + 'static,
        on_err: impl Fn(E) -> Msg + Send + 'static,
    ) -> Self
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Send + 'static,
    {
        Self::Perform(Box::pin(async move {
            match future.await {
                Ok(value) => Some(on_ok(value)),
                Err(err) => Some(on_err(err)),
            }
        }))
    }

    /// Perform an async operation that always succeeds (infallible).
    ///
    /// # Example
    /// ```ignore
    /// Cmd::perform_infallible(
    ///     async { SystemTime::now() },
    ///     Msg::TimeUpdated,
    /// )
    /// ```
    #[must_use]
    pub fn perform_infallible<F, T>(
        future: F,
        to_msg: impl Fn(T) -> Msg + Send + Sync + 'static,
    ) -> Self
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Self::Perform(Box::pin(async move { Some(to_msg(future.await)) }))
    }

    /// Create a command that immediately produces a message.
    ///
    /// Useful for triggering follow-up actions without async work.
    ///
    /// # Example
    /// ```ignore
    /// // After saving, trigger a refresh
    /// Cmd::msg(Msg::RefreshList)
    /// ```
    #[must_use]
    pub fn msg(msg: Msg) -> Self {
        Self::Perform(Box::pin(async move { Some(msg) }))
    }

    /// Check if this command is `None`.
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Flatten nested batches into a single level.
    #[must_use]
    pub fn flatten(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Perform(f) => Self::Perform(f),
            Self::Batch(cmds) => {
                let mut flattened = Vec::new();
                for cmd in cmds {
                    match cmd.flatten() {
                        Self::None => {}
                        Self::Batch(inner) => flattened.extend(inner),
                        other => flattened.push(other),
                    }
                }
                Self::batch(flattened)
            }
        }
    }
}

// Manual Debug implementation since we can't derive it for the Future
impl<Msg> std::fmt::Debug for Cmd<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Cmd::None"),
            Self::Batch(cmds) => f.debug_tuple("Cmd::Batch").field(&cmds.len()).finish(),
            Self::Perform(_) => write!(f, "Cmd::Perform(<future>)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_default() {
        let cmd: Cmd<()> = Cmd::default();
        assert!(cmd.is_none());
    }

    #[test]
    fn batch_of_empty_is_none() {
        let cmd: Cmd<()> = Cmd::batch([]);
        assert!(cmd.is_none());
    }

    #[test]
    fn batch_of_one_unwraps() {
        let cmd: Cmd<i32> = Cmd::batch([Cmd::msg(42)]);
        assert!(matches!(cmd, Cmd::Perform(_)));
    }

    #[test]
    fn flatten_removes_nones() {
        let cmd: Cmd<i32> = Cmd::batch([Cmd::none(), Cmd::msg(1), Cmd::none(), Cmd::msg(2)]);
        let flattened = cmd.flatten();
        if let Cmd::Batch(cmds) = flattened {
            assert_eq!(cmds.len(), 2);
        } else {
            panic!("Expected Batch");
        }
    }
}
