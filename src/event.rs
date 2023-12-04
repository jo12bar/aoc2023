//! Terminal events handler.

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use color_eyre::eyre::Context;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};

/// Terminal events.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    Mouse(MouseEvent),
    /// Terminal resize to (`width`, `height`).
    Resize(u16, u16),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    #[allow(dead_code)]
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
    /// Event handler thread.
    #[allow(dead_code)]
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Start an event handler thread that polls for events every `tick_rate` milliseconds.
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();

        let handler = {
            let sender = sender.clone();
            thread::spawn(move || event_poller(sender, tick_rate))
        };

        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Recieve the next event from the handler thread.
    ///
    /// This function will always block the current thread if there is no data
    /// available and it's possible for more data to be sent.
    pub fn next(&self) -> color_eyre::eyre::Result<Event> {
        self.receiver
            .recv()
            .wrap_err("Error while receiving event from event handler thread.")
    }
}

/// Polls for events and sends them to a channel. Should be run in a background thread.
fn event_poller(sender: mpsc::Sender<Event>, tick_rate: Duration) {
    let mut last_tick = Instant::now();
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(tick_rate);

        if event::poll(timeout).expect("unable to poll for event") {
            match event::read().expect("unable to read event") {
                CrosstermEvent::Key(e) => {
                    if e.kind == event::KeyEventKind::Press {
                        sender.send(Event::Key(e))
                    } else {
                        Ok(()) // ignore KeyEventKind::Release
                    }
                }

                CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),

                CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),

                _ => Ok(()), // ignore all other events
            }
            .expect("failed to send terminal event")
        }

        if last_tick.elapsed() >= tick_rate {
            sender.send(Event::Tick).expect("Failed to send tick event");
            last_tick = Instant::now();
        }
    }
}
