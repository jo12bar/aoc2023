use color_eyre::eyre::{Result, WrapErr};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::Rect;
use tokio::sync::broadcast;

use crate::{
    message::Message,
    model::{self, RunningState},
    termination::{Interrupted, Terminator},
    tui::{self, TuiEvent},
    view::view,
};

pub struct App {
    /// How fast to tick the TUI at.
    tick_rate: f64,
    /// Rendering frame per second cap.
    frame_rate: f64,
    /// Allows for terminating background threads.
    terminator: Terminator,
    // /// Receiver for termination messages from the main thread.
    // termination_rx: broadcast::Receiver<Interrupted>,
}

impl App {
    pub fn new(
        tick_rate: f64,
        frame_rate: f64,
        terminator: Terminator,
        _termination_rx: broadcast::Receiver<Interrupted>,
    ) -> Result<Self> {
        Ok(Self {
            tick_rate,
            frame_rate,
            terminator,
            // termination_rx,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = tui::Tui::new()
            .wrap_err("Error initializing text user interface (TUI)")?
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()
            .wrap_err("Error entering text user interface (TUI) mode")?;

        let mut model = model::init(&tui);

        loop {
            // Wait for the TUI to offer up an event.
            if let Some(e) = tui.next().await {
                // Handle events from the TUI and map to a message
                let mut current_message = match e {
                    TuiEvent::Quit => Some(Message::Quit),

                    TuiEvent::Tick => Some(Message::Tick),

                    // Render if the TUI says we should.
                    TuiEvent::Render => {
                        tui.draw(|f| view(&mut model, f))
                            .wrap_err("Error rendering TUI")?;
                        Some(Message::Render)
                    },

                    // Re-render if the TUI has been resized
                    TuiEvent::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))
                            .wrap_err("Error resizing TUI")?;

                        tui.draw(|f| view(&mut model, f))
                            .wrap_err("Error re-rendering TUI after resize")?;

                        Some(Message::Resize(w, h))
                    },

                    TuiEvent::Key(key) => handle_key_event(key),

                    _ => None,
                };

                // Process messages to update the model. Loop until the update function stops
                // returning new messages.
                while current_message.is_some() {
                    current_message = model::update(&mut model, current_message.unwrap())
                }
            }

            if model.running_state == RunningState::ShouldSuspend {
                // TODO(jo12bar): Implement suspension

                // // Suspend the TUI
                // tui.suspend().wrap_err("Error suspending TUI")?;
                // // Queue a resume action for as soon as the app is unsuspended
                // action_tx.send(Action::Resume)?;
                // tui = tui::Tui::new()
                //     .wrap_err("Error re-initializing TUI after suspend")?
                //     .tick_rate(self.tick_rate)
                //     .frame_rate(self.frame_rate);
                // // tui.mouse(true)
                // tui.enter()
                //     .wrap_err("Error entering TUI mode after suspend")?;
            } else if model.running_state == RunningState::ShouldQuit {
                tui.stop().wrap_err("Error stopping TUI")?;
                self.terminator.terminate(Interrupted::UserInt)?;
                break;
            }
        }

        tui.exit().wrap_err("Error exiting TUI mode")?;
        Ok(())
    }
}

fn handle_key_event(key: KeyEvent) -> Option<Message> {
    if key.kind == KeyEventKind::Press {
        return match key.code {
            KeyCode::Char('j') => Some(Message::Increment),
            KeyCode::Char('k') => Some(Message::Decrement),
            KeyCode::Char('q') => Some(Message::Quit),
            _ => None,
        };
    }

    None
}
