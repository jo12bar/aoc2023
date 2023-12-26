use ratatui::layout::Rect;

use crate::{command::Cmd, fps_counter, message::Message, tui::Tui};

#[derive(Debug, Default)]
pub struct Model {
    pub counter: i32,
    pub running_state: RunningState,
    pub tui_size: Rect,
    pub fps_counter: fps_counter::FpsCounterModel,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    ShouldQuit,
    ShouldSuspend,
}

pub fn init(tui: &Tui) -> (Model, Cmd<Message>) {
    (
        Model {
            tui_size: tui.size().unwrap(),
            ..Default::default()
        },
        Cmd::None,
    )
}

pub fn update(mut model: Model, msg: Message) -> (Model, Cmd<Message>) {
    match msg {
        Message::Increment => {
            model.counter += 1;
            if model.counter > 50 {
                return update(model, Message::Reset);
            }
        },
        Message::Decrement => {
            model.counter -= 1;
            if model.counter < -50 {
                return update(model, Message::Reset);
            }
        },
        Message::Reset => {
            model.counter = 0;
        },
        Message::Quit => {
            model.running_state = RunningState::ShouldQuit;
        },
        Message::Render => {
            return update(
                model,
                Message::FpsCounterMessage(fps_counter::FpsCounterMessage::Render),
            );
        },
        Message::Tick => {
            return update(
                model,
                Message::FpsCounterMessage(fps_counter::FpsCounterMessage::Tick),
            );
        },
        Message::Resize(w, h) => {
            model.tui_size.width = w;
            model.tui_size.height = h;
        },
        Message::FpsCounterMessage(m) => {
            return fps_counter::update(model, m);
        },
    }
    (model, Cmd::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn model_setup() -> Model {
        Model::default()
    }

    #[test]
    fn test_increment() {
        let mut model = model_setup();
        model.counter = -4;
        let (model, cmd) = update(model, Message::Increment);
        assert_eq!(model.counter, -3);
        assert!(matches!(cmd, Cmd::None));
    }

    #[test]
    fn test_decrement() {
        let mut model = model_setup();
        model.counter = 35;
        let (model, cmd) = update(model, Message::Decrement);
        assert_eq!(model.counter, 34);
        assert!(matches!(cmd, Cmd::None));
    }

    #[test]
    fn test_increment_saturating() {
        let mut model = model_setup();
        model.counter = 50;
        let (model, cmd) = update(model, Message::Increment);
        assert_eq!(model.counter, 0);
        assert!(matches!(cmd, Cmd::None));
    }

    #[test]
    fn test_decrement_saturating() {
        let mut model = model_setup();
        model.counter = -50;
        let (model, cmd) = update(model, Message::Decrement);
        assert_eq!(model.counter, 0);
        assert!(matches!(cmd, Cmd::None));
    }
}
