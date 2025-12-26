//! Terminal setup and teardown utilities.

use std::io::{self, stdout, Stdout};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::Result;

/// A wrapper around the terminal that ensures proper cleanup.
pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    /// Initialize the terminal for TUI rendering.
    ///
    /// This:
    /// - Enables raw mode (no line buffering, direct key input)
    /// - Switches to the alternate screen buffer
    /// - Hides the cursor
    ///
    /// The terminal is automatically restored when this guard is dropped.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    /// Get a mutable reference to the underlying terminal.
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    /// Draw to the terminal using a closure.
    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best effort cleanup - ignore errors since we're in drop
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

/// Restore the terminal to its original state.
///
/// This is called automatically when `TerminalGuard` is dropped,
/// but can be called manually if needed.
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
