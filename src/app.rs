use color_eyre::eyre::{Result, WrapErr};
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use tokio::sync::mpsc;

use crate::{
    action::Action,
    components::{fps::FpsCounter, home::Home, Component},
    config::Config,
    mode::Mode,
    tui,
};

pub struct App {
    /// Application config.
    pub config: Config,
    /// How fast to tick the TUI at.
    pub tick_rate: f64,
    /// Rendering frame per second cap.
    pub frame_rate: f64,
    /// Components to render.
    pub components: Vec<Box<dyn Component>>,
    /// If true, the app will quit on the next render cycle (and shut down the simulation).
    pub should_quit: bool,
    /// If true, the app will suspend rendering and run in the background.
    pub should_suspend: bool,
    /// Current mode the app is in.
    pub mode: Mode,
    /// Key events that happened during the last TUI tick.
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let home = Home::new();
        let fps = FpsCounter::default();
        let config = Config::new().wrap_err("Error initializing app config")?;
        let mode = Mode::Home;

        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(home), Box::new(fps)],
            should_quit: false,
            should_suspend: false,
            config,
            mode,
            last_tick_key_events: Vec::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        let mut tui = tui::Tui::new()
            .wrap_err("Error initializing text user interface (TUI)")?
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()
            .wrap_err("Error entering text user interface (TUI) mode")?;

        for component in self.components.iter_mut() {
            component
                .register_action_handler(action_tx.clone())
                .wrap_err("Error registering action handler with a TUI component")?;
            component
                .register_config_handler(self.config.clone())
                .wrap_err("Error registering config handler with a TUI component")?;
            component
                .init(tui.size().wrap_err("Error getting TUI size")?)
                .wrap_err("Error initializing component")?;
        }

        loop {
            // Wait for the TUI to offer up an event.
            if let Some(e) = tui.next().await {
                // Send actions based on what the TUI coughed up.
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,

                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,

                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,

                    // Check if a key event matchs something from the keybindings, and send an action
                    // if it does.
                    tui::Event::Key(key) => {
                        if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                            if let Some(action) = keymap.get(&vec![key]) {
                                tracing::trace!(?action, "Sending single-key action from TUI");
                                action_tx.send(action.clone())?;
                            } else {
                                // If the key was not handled as a single-key action, then consider
                                // it for multi-key combinations
                                self.last_tick_key_events.push(key);

                                // Check for multi-key combinations
                                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                                    tracing::trace!(?action, "Sending multi-key action from TUI");
                                    action_tx.send(action.clone())?;
                                }
                            }
                        }
                    },

                    _ => {},
                }

                // Let the components handle their own events and send off actions.
                for component in self.components.iter_mut() {
                    if let Some(action) = component
                        .handle_events(Some(e.clone()))
                        .wrap_err("Error handling events in a TUI component")?
                    {
                        action_tx.send(action)?;
                    }
                }
            }

            // Receive all actions from the MPSC channel, and act on them.
            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    tracing::trace!(?action, "Received action");
                }

                match action {
                    // On TUI tick, drain all the key events that occurred during the tick.
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    },

                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,

                    // Redraw everything if the console resizes.
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))
                            .wrap_err("Error resizing TUI")?;
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!(
                                            "Failed to draw during resize: {e:?}"
                                        )))
                                        .unwrap();
                                }
                            }
                        })
                        .wrap_err("Error redrawing TUI while handling resize event")?;
                    },

                    // Render all the stuff.
                    Action::Render => {
                        tui.draw(|f| {
                            for component in self.components.iter_mut() {
                                let r = component.draw(f, f.size());
                                if let Err(e) = r {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {e:?}")))
                                        .unwrap();
                                }
                            }
                        })
                        .wrap_err("Error drawing TUI")?;
                    },

                    _ => {},
                }

                // Update the components, and send along their actions to be handled this frame
                // (since we're still in the action MPSC receiving loop).
                for component in self.components.iter_mut() {
                    if let Some(action) = component
                        .update(action.clone())
                        .wrap_err("Error updating a TUI component")?
                    {
                        action_tx.send(action)?;
                    }
                }
            }

            if self.should_suspend {
                // Suspend the TUI
                tui.suspend().wrap_err("Error suspending TUI")?;
                // Queue a resume action for as soon as the app is unsuspended
                action_tx.send(Action::Resume)?;
                tui = tui::Tui::new()
                    .wrap_err("Error re-initializing TUI after suspend")?
                    .tick_rate(self.tick_rate)
                    .frame_rate(self.frame_rate);
                // tui.mouse(true)
                tui.enter()
                    .wrap_err("Error entering TUI mode after suspend")?;
            } else if self.should_quit {
                tui.stop().wrap_err("Error stopping TUI")?;
                break;
            }
        }

        tui.exit().wrap_err("Error exiting TUI mode")?;
        Ok(())
    }
}
