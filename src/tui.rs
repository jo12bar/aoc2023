//! Terminal user interface.

use std::{io, panic};

use color_eyre::eyre::{self, Context};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{app::App, event::EventHandler, ui};

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

/// The terminal user interface.
///
/// Responsible for setting up the terminal, initializing the interface and
/// handling the draw events.
pub struct Tui {
    /// Handle to the [`ratatui::Terminal`].
    terminal: CrosstermTerminal,
    /// Terminal event handler.
    pub events: EventHandler,
}

impl Tui {
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Intialize the terminal interface.
    ///
    /// This enables raw mode and sets various terminal properties. It also sets
    /// up a panic handler to reset the terminal on crash.
    pub fn enter(&mut self) -> eyre::Result<()> {
        terminal::enable_raw_mode().wrap_err("Error enabling terminal's raw mode")?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)
            .wrap_err("Error setting terminal properties")?;

        // Setup a custom panic hook to reset the terminal on crash.
        let orig_panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            Self::reset().expect("Failed to reset the terminal.");
            orig_panic_hook(panic_info);
        }));

        self.terminal
            .hide_cursor()
            .wrap_err("Couldn't hide cursor.")?;
        self.terminal
            .clear()
            .wrap_err("Error performing initial terminal clear.")?;

        Ok(())
    }

    /// Reset the terminal interface.
    ///
    /// This function is also used by a custom panic hook set up in [`Tui::enter`]
    /// to revert the terminal properties if unexpected errors occur.
    fn reset() -> eyre::Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exit the terminal interface.
    ///
    /// Disables raw mode and reverts the terminal properties set in [`Tui::enter`].
    pub fn exit(&mut self) -> eyre::Result<()> {
        Self::reset().wrap_err("Error restting terminal")?;
        self.terminal
            .show_cursor()
            .wrap_err("Error showing cursor")?;
        Ok(())
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui::render
    pub fn draw(&mut self, app: &mut App) -> eyre::Result<()> {
        self.terminal
            .draw(|frame| ui::render(app, frame))
            .wrap_err("Error drawing terminal interface")?;
        Ok(())
    }
}
