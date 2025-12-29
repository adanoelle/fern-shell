//! The Elm runtime - the event loop that drives the application.
//!
//! The runtime:
//! 1. Initializes the application with `init()`
//! 2. Renders the initial view
//! 3. Listens for messages from commands, subscriptions, and input
//! 4. Calls `update()` for each message
//! 5. Re-renders the view
//! 6. Reconciles subscriptions
//! 7. Repeats until `should_quit()` returns true

use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::terminal::TerminalGuard;
use crate::{Application, Cmd, Result, Sub};

/// Run an Elm-style application.
///
/// This is the main entry point for frond applications. It:
/// 1. Sets up the terminal
/// 2. Initializes your application
/// 3. Runs the event loop
/// 4. Cleans up on exit
///
/// # Example
/// ```ignore
/// #[tokio::main]
/// async fn main() -> frond::Result<()> {
///     frond::run::<MyApp>().await
/// }
/// ```
pub async fn run<App: Application>() -> Result<()> {
    // Set up terminal
    let mut term = TerminalGuard::new()?;

    // Initialize application
    let (mut model, init_cmd) = App::init();

    // Channel for messages from commands and subscriptions
    let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<App::Msg>();

    // Spawn initial commands
    spawn_commands(init_cmd, msg_tx.clone());

    // Track active subscription tasks
    let mut sub_handles: Vec<JoinHandle<()>> = Vec::new();

    // Initial subscriptions
    let subs = App::subscriptions(&model);
    spawn_subscriptions(subs, msg_tx.clone(), &mut sub_handles);

    // Spawn a dedicated thread for terminal event reading
    // Terminal I/O must happen on a dedicated OS thread (not tokio) because
    // crossterm's poll/read are blocking operations.
    // We use a tokio channel so we can use select! in the main loop.
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Event>();
    let event_thread = thread::spawn(move || {
        loop {
            // Use a longer poll timeout (1 second) to reduce CPU usage
            // This still allows reasonable responsiveness while being idle-friendly
            match event::poll(Duration::from_secs(1)) {
                Ok(true) => {
                    if let Ok(evt) = event::read() {
                        if event_tx.send(evt).is_err() {
                            // Receiver dropped, exit thread
                            break;
                        }
                    }
                }
                Ok(false) => {
                    // Timeout - check if channel is closed
                    if event_tx.is_closed() {
                        break;
                    }
                }
                Err(_) => {
                    // Poll error, exit thread
                    break;
                }
            }
        }
    });

    // Main loop
    loop {
        // Render
        term.draw(|frame| App::view(&model, frame))?;

        // Check if we should quit
        if App::should_quit(&model) {
            break;
        }

        // Wait for next event or message using select! (true event-driven, zero CPU when idle)
        let msg = loop {
            tokio::select! {
                // Terminal events from dedicated thread
                Some(event) = event_rx.recv() => {
                    if let Some(msg) = handle_terminal_event::<App>(&model, event) {
                        break msg;
                    }
                    // Event didn't produce a message, continue waiting
                }
                // Async messages from commands and subscriptions
                Some(msg) = msg_rx.recv() => {
                    break msg;
                }
                // Both channels closed
                else => {
                    return Ok(());
                }
            }
        };

        // Update
        let (new_model, cmd) = App::update(model, msg);
        model = new_model;

        // Spawn commands from update
        spawn_commands(cmd, msg_tx.clone());

        // Reconcile subscriptions
        // For now, we just cancel old ones and start new ones
        // A more sophisticated implementation would diff them
        cancel_subscriptions(&mut sub_handles);
        let new_subs = App::subscriptions(&model);
        spawn_subscriptions(new_subs, msg_tx.clone(), &mut sub_handles);
    }

    // Cleanup
    App::on_shutdown(&model);
    cancel_subscriptions(&mut sub_handles);

    // The event thread will exit when event_rx is dropped
    drop(event_rx);
    let _ = event_thread.join();

    Ok(())
}

/// Spawn commands as async tasks.
fn spawn_commands<Msg: Send + 'static>(cmd: Cmd<Msg>, tx: mpsc::UnboundedSender<Msg>) {
    match cmd {
        Cmd::None => {}
        Cmd::Batch(cmds) => {
            for cmd in cmds {
                spawn_commands(cmd, tx.clone());
            }
        }
        Cmd::Perform(future) => {
            let tx = tx.clone();
            tokio::spawn(async move {
                if let Some(msg) = future.await {
                    let _ = tx.send(msg);
                }
            });
        }
    }
}

/// Spawn subscription tasks.
fn spawn_subscriptions<Msg: Clone + Send + 'static>(
    sub: Sub<Msg>,
    tx: mpsc::UnboundedSender<Msg>,
    handles: &mut Vec<JoinHandle<()>>,
) {
    match sub {
        Sub::None => {}
        Sub::Batch(subs) => {
            for sub in subs {
                spawn_subscriptions(sub, tx.clone(), handles);
            }
        }
        Sub::Keyboard(_) => {
            // Keyboard is handled specially in the main loop via poll_terminal_event
            // We don't spawn a separate task for it
        }
        Sub::Interval { duration, msg } => {
            let tx = tx.clone();
            let handle = tokio::spawn(async move {
                // Use interval_at to start at a future time, avoiding the immediate first tick
                // that tokio::time::interval() produces. This prevents a feedback loop when
                // subscriptions are reconciled on every message.
                let start = tokio::time::Instant::now() + duration;
                let mut interval = tokio::time::interval_at(start, duration);
                loop {
                    interval.tick().await;
                    if tx.send(msg.clone()).is_err() {
                        break;
                    }
                }
            });
            handles.push(handle);
        }
        Sub::FileWatch { path, on_change } => {
            let tx = tx.clone();
            let handle = tokio::spawn(async move {
                // Simple polling implementation for now
                // A real implementation would use notify crate
                let mut last_modified = std::fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .ok();

                loop {
                    tokio::time::sleep(Duration::from_millis(500)).await;

                    let current_modified = std::fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .ok();

                    if current_modified != last_modified {
                        last_modified = current_modified;
                        let msg = on_change();
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                }
            });
            handles.push(handle);
        }
    }
}

/// Cancel all subscription tasks.
fn cancel_subscriptions(handles: &mut Vec<JoinHandle<()>>) {
    for handle in handles.drain(..) {
        handle.abort();
    }
}

/// Handle terminal events, converting them to messages via subscriptions.
fn handle_terminal_event<App: Application>(model: &App::Model, event: Event) -> Option<App::Msg> {
    match event {
        Event::Key(key_event) => {
            // Only handle key press events, not release
            if key_event.kind != KeyEventKind::Press {
                return None;
            }

            // Check keyboard subscriptions
            let subs = App::subscriptions(model);
            find_keyboard_handler(&subs, key_event)
        }
        Event::Resize(_, _) => {
            // Could add a resize subscription type in the future
            None
        }
        _ => None,
    }
}

/// Find a keyboard handler in the subscriptions and apply it.
fn find_keyboard_handler<Msg: Clone>(
    sub: &Sub<Msg>,
    key_event: crossterm::event::KeyEvent,
) -> Option<Msg> {
    match sub {
        Sub::None => None,
        Sub::Keyboard(handler) => handler(key_event),
        Sub::Batch(subs) => {
            for sub in subs {
                if let Some(msg) = find_keyboard_handler(sub, key_event) {
                    return Some(msg);
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // Runtime tests would require mocking terminal/tokio
    // Keeping simple for now
}
