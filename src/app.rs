//! Application.

#[derive(Debug, Default)]
pub struct App {
    /// Counter
    pub counter: u8,
    /// Should the application exit?
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    /// Tick the app state.
    pub fn tick(&self) {}

    /// Set the application to quit.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_increment_counter() {
        let mut app = App::default();
        app.increment_counter();
        assert_eq!(app.counter, 1);
    }

    #[test]
    fn test_app_increment_saturated_counter() {
        let mut app = App {
            counter: u8::MAX,
            ..Default::default()
        };
        app.increment_counter();
        assert_eq!(app.counter, u8::MAX);
    }

    #[test]
    fn test_app_decrement_counter() {
        let mut app = App {
            counter: 42,
            ..Default::default()
        };
        app.decrement_counter();
        assert_eq!(app.counter, 41);
    }

    #[test]
    fn test_app_decrement_saturated_counter() {
        let mut app = App {
            counter: u8::MIN,
            ..Default::default()
        };
        app.decrement_counter();
        assert_eq!(app.counter, u8::MIN);
    }
}
