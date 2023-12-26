use std::pin::pin;

use color_eyre::eyre::{Result, WrapErr};
use futures::{prelude::*, stream_select};
use ratatui::prelude::Rect;
use tokio::sync::{broadcast, mpsc::channel};

use crate::{
    command::{self, process_cmd},
    message::Message,
    model::{self, RunningState},
    subscriptions::{subscriptions, tui_event_subscription},
    termination::{Interrupted, Terminator},
    tui::{self},
    view::view,
};

pub struct App {
    /// How fast to tick the TUI at.
    tick_rate: f64,
    /// Rendering frame per second cap.
    frame_rate: f64,
    /// Allows for terminating background threads.
    terminator: Terminator,
    /// Receiver for termination messages from the main thread.
    #[allow(unused)]
    termination_rx: broadcast::Receiver<Interrupted>,
}

impl App {
    pub fn new(
        tick_rate: f64,
        frame_rate: f64,
        terminator: Terminator,
        termination_rx: broadcast::Receiver<Interrupted>,
    ) -> Result<Self> {
        Ok(Self {
            tick_rate,
            frame_rate,
            terminator,
            termination_rx,
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

        let (msg_tx, msg_rx) = channel::<Message>(1);

        let (init_model, init_cmd) = model::init(&tui);

        command::process_cmd(init_cmd, msg_tx.clone());

        let (mut model, subs) = subscriptions(init_model);
        let tui_event_sub = tui_event_subscription(
            tui.take_event_rx()
                .expect("TUI event receiver should not already be taken, but it is"),
        );

        let msgs = tokio_stream::wrappers::ReceiverStream::new(msg_rx);
        let mut combined_msgs_stream = pin!(stream_select!(tui_event_sub, subs, msgs).fuse());

        while let Some(msg) = combined_msgs_stream.next().await {
            let should_render = msg == Message::Render;
            let resize_params = if let Message::Resize(w, h) = msg {
                Some((w, h))
            } else {
                None
            };

            let (new_model, cmd) = model::update(model, msg);
            model = new_model;

            process_cmd(cmd, msg_tx.clone());

            if let Some((w, h)) = resize_params {
                tui.resize(Rect::new(0, 0, w, h))
                    .wrap_err("Error resizing TUI")?;
            }

            if should_render || resize_params.is_some() {
                tui.draw(|f| view(&mut model, f))
                    .wrap_err("Error rendering TUI")?;
            }

            if model.running_state == RunningState::ShouldQuit {
                tui.stop().wrap_err("Error stopping TUI")?;
                tui.exit().wrap_err("Error exiting TUI mode")?;
                self.terminator.terminate(Interrupted::UserInt)?;
                break;
            }
        }

        // // Process messages to update the model. Loop until the update function stops
        // // returning new messages.
        // while current_message.is_some() {
        //     current_message = model::update(&mut model, current_message.unwrap())
        // }

        // if model.running_state == RunningState::ShouldSuspend {
        //     // TODO(jo12bar): Implement suspension

        //     // // Suspend the TUI
        //     // tui.suspend().wrap_err("Error suspending TUI")?;
        //     // // Queue a resume action for as soon as the app is unsuspended
        //     // action_tx.send(Action::Resume)?;
        //     // tui = tui::Tui::new()
        //     //     .wrap_err("Error re-initializing TUI after suspend")?
        //     //     .tick_rate(self.tick_rate)
        //     //     .frame_rate(self.frame_rate);
        //     // // tui.mouse(true)
        //     // tui.enter()
        //     //     .wrap_err("Error entering TUI mode after suspend")?;
        // } else if model.running_state == RunningState::ShouldQuit {
        //     tui.stop().wrap_err("Error stopping TUI")?;
        //     self.terminator.terminate(Interrupted::UserInt)?;
        //     tui.exit().wrap_err("Error exiting TUI mode")?;
        //     break;
        // }

        Ok(())
    }
}

// async fn flatten<T, E: Send + Sync + std::error::Error + 'static>(
//     handle: JoinHandle<Result<T, E>>,
// ) -> Result<T> {
//     match handle.await {
//         Ok(Ok(res)) => Ok(res),
//         Ok(Err(e)) => Err(e).wrap_err("Error in task"),
//         Err(e) => Err(e).wrap_err("Error joining task"),
//     }
// }
