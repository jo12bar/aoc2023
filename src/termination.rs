//! Utilities for terminating the application.
//!
//! Heavily based on code from [`rust-chat-server`][rust-chat-server].
//!
//! [rust-chat-server]: https://github.com/Yengas/rust-chat-server/blob/main/tui/src/termination.rs

#[cfg(unix)]
use tokio::signal::unix::signal;

use color_eyre::eyre;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Interrupted {
    OsSigInt,
    UserInt,
}

#[derive(Debug, Clone)]
pub struct Terminator {
    interrupt_tx: broadcast::Sender<Interrupted>,
}

impl Terminator {
    pub fn new(interrupt_tx: broadcast::Sender<Interrupted>) -> Self {
        Self { interrupt_tx }
    }

    pub fn terminate(&mut self, interrupted: Interrupted) -> eyre::Result<()> {
        self.interrupt_tx.send(interrupted)?;

        Ok(())
    }
}

#[cfg(unix)]
async fn terminate_by_unix_signal(mut terminator: Terminator) {
    let mut interrupt_signal = signal(tokio::signal::unix::SignalKind::interrupt())
        .expect("failed to create interrupt signal stream");

    interrupt_signal.recv().await;

    terminator
        .terminate(Interrupted::OsSigInt)
        .expect("failed to send interrupt signal");
}

/// Create a broadcast channel for retrieving the application kill signal
pub fn create_termination() -> (Terminator, broadcast::Receiver<Interrupted>) {
    let (tx, rx) = broadcast::channel(1);
    let terminator = Terminator::new(tx);

    #[cfg(unix)]
    tokio::spawn(terminate_by_unix_signal(terminator.clone()));

    (terminator, rx)
}
