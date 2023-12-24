use crate::message::Message;

#[derive(Debug, Default)]
pub struct Model {
    pub counter: i32,
    pub running_state: RunningState,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    ShouldQuit,
    ShouldSuspend,
}

pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Increment => {
            model.counter += 1;
            if model.counter > 50 {
                return Some(Message::Reset);
            }
        },
        Message::Decrement => {
            model.counter -= 1;
            if model.counter < -50 {
                return Some(Message::Reset);
            }
        },
        Message::Reset => {
            model.counter = 0;
        },
        Message::Quit => {
            model.running_state = RunningState::ShouldQuit;
        },
        Message::Render => {
            // TODO(jo12bar): Update frame render counter.
        },
        Message::Tick => {
            // TODO(jo12bar): Update app tick counter.
        },
        Message::Resize(_, _) => {
            // TODO(jo12bar): Update app size tracker.
        },
    }
    None
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
        let res = update(&mut model, Message::Increment);
        assert_eq!(res, None);
        assert_eq!(model.counter, -3);
    }

    #[test]
    fn test_decrement() {
        let mut model = model_setup();
        model.counter = 35;
        let res = update(&mut model, Message::Decrement);
        assert_eq!(res, None);
        assert_eq!(model.counter, 34);
    }

    #[test]
    fn test_increment_saturating() {
        let mut model = model_setup();
        model.counter = 50;
        let res = update(&mut model, Message::Increment);
        assert_eq!(res, Some(Message::Reset));
        assert_eq!(model.counter, 51);
        let res = update(&mut model, res.unwrap());
        assert_eq!(res, None);
        assert_eq!(model.counter, 0);
    }

    #[test]
    fn test_decrement_saturating() {
        let mut model = model_setup();
        model.counter = -50;
        let res = update(&mut model, Message::Decrement);
        assert_eq!(res, Some(Message::Reset));
        assert_eq!(model.counter, -51);
        let res = update(&mut model, res.unwrap());
        assert_eq!(res, None);
        assert_eq!(model.counter, 0);
    }
}
