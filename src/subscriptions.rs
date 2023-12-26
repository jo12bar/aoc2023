use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use futures::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{message::Message, model::Model, tui::TuiEvent};

pub type Subscription<'a, Msg> = stream::BoxStream<'a, Msg>;

fn on_tui_event(tui_event_rx: UnboundedReceiver<TuiEvent>) -> impl Stream<Item = TuiEvent> {
    tokio_stream::wrappers::UnboundedReceiverStream::new(tui_event_rx)
}

/// Handle events from the TUI and map to a message
pub fn tui_event_subscription(
    tui_event_rx: UnboundedReceiver<TuiEvent>,
) -> Subscription<'static, Message> {
    Box::pin(
        on_tui_event(tui_event_rx).filter_map(|tui_event| async move {
            match tui_event {
                TuiEvent::Quit => Some(Message::Quit),

                TuiEvent::Tick => Some(Message::Tick),

                // Render if the TUI says we should.
                TuiEvent::Render => {
                    // tui.draw(|f| view(&mut model, f))
                    //     .wrap_err("Error rendering TUI")?;
                    Some(Message::Render)
                },

                // Re-render if the TUI has been resized
                TuiEvent::Resize(w, h) => {
                    // tui.resize(Rect::new(0, 0, w, h))
                    //     .wrap_err("Error resizing TUI")?;

                    // tui.draw(|f| view(&mut model, f))
                    //     .wrap_err("Error re-rendering TUI after resize")?;

                    Some(Message::Resize(w, h))
                },

                TuiEvent::Key(key) => handle_key_event(key),

                _ => None,
            }
        }),
    )
}

/// Update the list of subscriptions.
///
/// Currently only called once on startup, and never again.
pub fn subscriptions(model: Model) -> (Model, Subscription<'static, Message>) {
    (model, Box::pin(tokio_stream::empty()))
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
