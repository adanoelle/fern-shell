//! Subscriptions - declarations of external event sources.
//!
//! Subscriptions tell the runtime what external events your application cares about.
//! They're recalculated after every update, allowing you to conditionally subscribe
//! based on your model's state.
//!
//! Unlike commands (which are one-shot), subscriptions are ongoing event sources
//! like keyboard input, timers, or file watchers.

use crossterm::event::KeyEvent;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

/// A subscription to external events.
///
/// Subscriptions are declarative - you describe what events you want,
/// and the runtime manages the actual event sources.
pub enum Sub<Msg> {
    /// No subscription - don't listen for any events.
    None,

    /// Combine multiple subscriptions.
    Batch(Vec<Sub<Msg>>),

    /// Subscribe to keyboard events.
    ///
    /// The handler function receives key events and returns `Some(msg)` for
    /// events it wants to handle, `None` for events to ignore.
    Keyboard(Arc<dyn Fn(KeyEvent) -> Option<Msg> + Send + Sync>),

    /// Subscribe to timer ticks at a fixed interval.
    ///
    /// Produces the given message every `duration`.
    Interval {
        /// How often to tick.
        duration: Duration,
        /// The message to produce on each tick.
        msg: Msg,
    },

    /// Subscribe to file system changes.
    ///
    /// Produces the message when the file at `path` changes.
    FileWatch {
        /// The file to watch.
        path: PathBuf,
        /// Function to produce a message when the file changes.
        on_change: Arc<dyn Fn() -> Msg + Send + Sync>,
    },
}

impl<Msg> Default for Sub<Msg> {
    fn default() -> Self {
        Self::None
    }
}

impl<Msg: Clone + Send + 'static> Sub<Msg> {
    /// Create a subscription that doesn't listen to anything.
    ///
    /// # Example
    /// ```ignore
    /// fn subscriptions(_model: &Model) -> Sub<Msg> {
    ///     Sub::none()
    /// }
    /// ```
    #[must_use]
    pub fn none() -> Self {
        Self::None
    }

    /// Combine multiple subscriptions.
    ///
    /// # Example
    /// ```ignore
    /// fn subscriptions(model: &Model) -> Sub<Msg> {
    ///     Sub::batch([
    ///         Sub::on_key(handle_keys),
    ///         Sub::every(Duration::from_secs(1), Msg::Tick),
    ///     ])
    /// }
    /// ```
    #[must_use]
    pub fn batch(subs: impl IntoIterator<Item = Sub<Msg>>) -> Self {
        let subs: Vec<_> = subs.into_iter().collect();
        if subs.is_empty() {
            Self::None
        } else if subs.len() == 1 {
            subs.into_iter().next().unwrap_or(Self::None)
        } else {
            Self::Batch(subs)
        }
    }

    /// Subscribe to keyboard events.
    ///
    /// The handler receives every key event and returns `Some(msg)` for keys
    /// it wants to handle, `None` otherwise.
    ///
    /// # Example
    /// ```ignore
    /// Sub::on_key(|key| match key.code {
    ///     KeyCode::Char('q') => Some(Msg::Quit),
    ///     KeyCode::Up => Some(Msg::Up),
    ///     KeyCode::Down => Some(Msg::Down),
    ///     _ => None,
    /// })
    /// ```
    #[must_use]
    pub fn on_key<F>(handler: F) -> Self
    where
        F: Fn(KeyEvent) -> Option<Msg> + Send + Sync + 'static,
    {
        Self::Keyboard(Arc::new(handler))
    }

    /// Subscribe to timer ticks at a fixed interval.
    ///
    /// # Example
    /// ```ignore
    /// // Update clock every second
    /// Sub::every(Duration::from_secs(1), Msg::Tick)
    /// ```
    #[must_use]
    pub fn every(duration: Duration, msg: Msg) -> Self {
        Self::Interval { duration, msg }
    }

    /// Subscribe to file system changes.
    ///
    /// The message is produced whenever the file at `path` is modified.
    ///
    /// # Example
    /// ```ignore
    /// Sub::on_file_change("config.toml", || Msg::ConfigChanged)
    /// ```
    #[must_use]
    pub fn on_file_change<F>(path: impl Into<PathBuf>, on_change: F) -> Self
    where
        F: Fn() -> Msg + Send + Sync + 'static,
    {
        Self::FileWatch {
            path: path.into(),
            on_change: Arc::new(on_change),
        }
    }

    /// Conditionally include a subscription.
    ///
    /// Returns `sub` if `condition` is true, otherwise returns `Sub::none()`.
    ///
    /// # Example
    /// ```ignore
    /// fn subscriptions(model: &Model) -> Sub<Msg> {
    ///     Sub::batch([
    ///         Sub::on_key(handle_keys),
    ///         // Only watch config when settings panel is open
    ///         Sub::when(
    ///             model.settings_open,
    ///             Sub::on_file_change("config.toml", || Msg::ConfigChanged),
    ///         ),
    ///     ])
    /// }
    /// ```
    #[must_use]
    pub fn when(condition: bool, sub: Self) -> Self {
        if condition {
            sub
        } else {
            Self::None
        }
    }

    /// Check if this subscription is `None`.
    #[must_use]
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Flatten nested batches into a single level.
    #[must_use]
    pub fn flatten(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Keyboard(h) => Self::Keyboard(h),
            Self::Interval { duration, msg } => Self::Interval { duration, msg },
            Self::FileWatch { path, on_change } => Self::FileWatch { path, on_change },
            Self::Batch(subs) => {
                let mut flattened = Vec::new();
                for sub in subs {
                    match sub.flatten() {
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

// Manual Debug implementation since we can't derive it for the closures
impl<Msg> std::fmt::Debug for Sub<Msg> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Sub::None"),
            Self::Batch(subs) => f.debug_tuple("Sub::Batch").field(&subs.len()).finish(),
            Self::Keyboard(_) => write!(f, "Sub::Keyboard(<handler>)"),
            Self::Interval { duration, .. } => {
                f.debug_struct("Sub::Interval").field("duration", duration).finish()
            }
            Self::FileWatch { path, .. } => {
                f.debug_struct("Sub::FileWatch").field("path", path).finish()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_default() {
        let sub: Sub<()> = Sub::default();
        assert!(sub.is_none());
    }

    #[test]
    fn batch_of_empty_is_none() {
        let sub: Sub<()> = Sub::batch([]);
        assert!(sub.is_none());
    }

    #[test]
    fn when_false_returns_none() {
        let sub: Sub<i32> = Sub::when(false, Sub::every(Duration::from_secs(1), 42));
        assert!(sub.is_none());
    }

    #[test]
    fn when_true_returns_sub() {
        let sub: Sub<i32> = Sub::when(true, Sub::every(Duration::from_secs(1), 42));
        assert!(matches!(sub, Sub::Interval { .. }));
    }
}
