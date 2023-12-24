use crate::fps_counter;

#[derive(Debug, PartialEq)]
pub enum Message {
    Increment,
    Decrement,
    Render,
    Tick,
    Resize(u16, u16),
    Reset,
    Quit,
    FpsCounterMessage(fps_counter::FpsCounterMessage),
}
