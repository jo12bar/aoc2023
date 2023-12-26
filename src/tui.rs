use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

use color_eyre::eyre::Result;
use crossterm::{
    cursor,
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CrosstermEvent, KeyEvent, KeyEventKind, MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

/// The chosen IO channel to output the TUI on.
pub type IO = std::io::Stderr;
/// Get a handle to the [`IO`] stream of this process.
pub fn io() -> IO {
    std::io::stderr()
}
/// A type alias for [`ratatui::Frame`], for convenience.
pub type Frame<'a> = ratatui::Frame<'a>;

/// Events to/from the TUI.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TuiEvent {
    /// TUI is initialized.
    Init,
    /// TUI is quitting.
    Quit,
    /// TUI is in an error state.
    Error,
    /// TUI has closed.
    Closed,
    /// TUI has ticked.
    Tick,
    /// TUI has been rendered.
    Render,
    /// TUI has gained focus.
    FocusGained,
    /// TUI has lost focus.
    FocusLost,
    /// The user has pasted text into the TUI.
    Paste(String),
    /// The user has pressed a key.
    Key(KeyEvent),
    /// The user moved or clicked the mouse.
    Mouse(MouseEvent),
    /// TUI has been resized.
    Resize(u16, u16),
}

/// The Text User Interface.
pub struct Tui {
    /// Handle to the terminal
    pub terminal: ratatui::Terminal<Backend<IO>>,
    /// Handle to the task that handles input events
    pub task: JoinHandle<()>,
    /// Cancellation token to shut down the event-handling task.
    pub cancellation_token: CancellationToken,
    /// Unbounded event receiver.
    pub event_rx: Option<UnboundedReceiver<TuiEvent>>,
    /// Unbounded event sender.
    pub event_tx: UnboundedSender<TuiEvent>,
    /// Rendering frame rate.
    pub frame_rate: f64,
    /// Event-handling tick rate.
    pub tick_rate: f64,
    /// If true, mouse interaction is enabled. False by default.
    pub mouse: bool,
    /// If true, pasting from the clipboard is enabled. False by default.
    pub paste: bool,
}

impl Tui {
    /// Initialize a new TUI.
    pub fn new() -> Result<Self> {
        let tick_rate = 4.0;
        let frame_rate = 60.0;
        let terminal = ratatui::Terminal::new(Backend::new(io()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async {});
        let mouse = false;
        let paste = false;
        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx: Some(event_rx),
            event_tx,
            frame_rate,
            tick_rate,
            mouse,
            paste,
        })
    }

    /// Set the TUI's tick rate.
    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    /// Set the TUI's frame rate.
    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    /// Enable or disable mouse interaction.
    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    /// Enable or disable pasting from the clipboard.
    pub fn paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }

    /// Start the background Tui task.
    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);

        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let task_cancellation_token = self.cancellation_token.clone();

        let event_tx = self.event_tx.clone();

        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            let mut render_interval = tokio::time::interval(render_delay);

            event_tx.send(TuiEvent::Init).unwrap();

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = task_cancellation_token.cancelled() => { break; }

                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Err(_)) => {
                                event_tx.send(TuiEvent::Error).unwrap();
                            }
                            None => {}

                            Some(Ok(evt)) => match evt {
                                CrosstermEvent::Key(key) => {
                                    if key.kind == KeyEventKind::Press {
                                        event_tx.send(TuiEvent::Key(key)).unwrap();
                                    }
                                }
                                CrosstermEvent::Mouse(mouse) => {
                                    event_tx.send(TuiEvent::Mouse(mouse)).unwrap();
                                }
                                CrosstermEvent::Resize(x, y) => {
                                    event_tx.send(TuiEvent::Resize(x, y)).unwrap();
                                },
                                CrosstermEvent::FocusLost => {
                                    event_tx.send(TuiEvent::FocusLost).unwrap();
                                },
                                CrosstermEvent::FocusGained => {
                                    event_tx.send(TuiEvent::FocusGained).unwrap();
                                },
                                CrosstermEvent::Paste(s) => {
                                    event_tx.send(TuiEvent::Paste(s)).unwrap();
                                },
                            }
                        }
                    }

                    _ = tick_delay => {
                        event_tx.send(TuiEvent::Tick).unwrap();
                    }

                    _ = render_delay => {
                        event_tx.send(TuiEvent::Render).unwrap();
                    }
                }
            }
        })
    }

    /// Stop the background Tui task.
    ///
    /// Will try to gracefully wait for the task to shut down. If it doesn't
    /// shut down, it will be hard aborted.
    pub fn stop(&self) -> Result<()> {
        self.cancel();

        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                tracing::error!(
                    "Failed to abort background TUI task in 100 milliseconds for unknown reason"
                );
                break;
            }
        }

        Ok(())
    }

    /// Use terminal escapes to enter Tui mode, and start the Tui background task.
    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(io(), EnterAlternateScreen, cursor::Hide)?;
        if self.mouse {
            crossterm::execute!(io(), EnableMouseCapture)?;
        }
        if self.paste {
            crossterm::execute!(io(), EnableBracketedPaste)?;
        }
        self.start();

        Ok(())
    }

    /// Use terminal escapes to exit Tui mode, and stop the Tui background task.
    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            if self.paste {
                crossterm::execute!(io(), DisableBracketedPaste)?;
            }
            if self.mouse {
                crossterm::execute!(io(), DisableMouseCapture)?;
            }
            crossterm::execute!(io(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }

        Ok(())
    }

    /// Cancel execution of the background TUI task, forcing it to start shutting down.
    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    /// Destroy the Tui on suspend. The Tui will have to be reinitialized with
    /// [`Tui::resume()`] when the app is resumed.
    ///
    /// On non-Windows platforms, this raises a `SIGTSTP` signal, which will
    /// cause the kernel to properly suspend the process.
    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;

        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;

        Ok(())
    }

    /// Resume the TUI after the app resumes.
    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    /// Take ownership of the TUI's event receiver.
    ///
    /// Returns `None` if the event receiver has already been taken.
    pub fn take_event_rx(&mut self) -> Option<UnboundedReceiver<TuiEvent>> {
        self.event_rx.take()
    }
}

/// Allows immutable access to the underlying ratatui terminal handle.
impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<IO>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

/// Allows mutable access to the underlying ratatui terminal handle.
impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

/// Properly exits the TUI mode on drop.
impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
