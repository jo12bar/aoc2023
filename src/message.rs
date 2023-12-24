#[derive(Debug, PartialEq)]
pub enum Message {
    Increment,
    Decrement,
    Render,
    Tick,
    Resize(u16, u16),
    Reset,
    Quit,
}
