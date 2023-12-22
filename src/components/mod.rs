use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    config::Config,
    tui::{Event, Frame},
};

pub mod fps;
pub mod home;

/// [`Component`] is a trait that represents a visual and interactive element of the user interface.
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    /// Register an action handler that can send actions for processing if necessary.
    ///
    /// # Arguments
    ///
    /// - `tx` - An [unbounded sender][tokio::sync::mpsc::UnboundedSender] that can send [`Action`]s.
    ///
    /// # Returns
    ///
    /// - An Ok result or an error.
    #[allow(unused_variables)]
    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Register a configuration handler that provides configuration settings if necessary.
    ///
    /// # Arguments
    ///
    /// - `config` - [Configuration][crate::config::Config] settings.
    ///
    /// # Returns
    ///
    /// - An Ok result or an error.
    #[allow(unused_variables)]
    fn register_config_handler(&mut self, config: Config) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Initialize the component with a specified area if necessary.
    ///
    /// # Arguments
    ///
    /// - `area` - [Rectangular area][ratatui::layout::Rect] to initialize the component within.
    ///
    /// # Returns
    ///
    /// - An Ok result or an error.
    #[allow(unused_variables)]
    fn init(&mut self, area: Rect) -> Result<(), ComponentError> {
        Ok(())
    }

    /// Handle incoming events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// - `event` - An optional [event][Event] to be processed.
    ///
    /// # Returns
    ///
    /// - An action to be processed or none.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>, ComponentError> {
        let r = match event {
            Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }

    /// Handle key events and produce actions if necessaru.
    ///
    /// # Arguments
    ///
    /// - `key_event` - A [key event][KeyEvent] to be processed.
    ///
    /// # Returns
    ///
    /// - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<Option<Action>, ComponentError> {
        Ok(None)
    }

    /// Handle mouse events and produce actions if necessaru.
    ///
    /// # Arguments
    ///
    /// - `mouse_event` - A [mouse event][MouseEvent] to be processed.
    ///
    /// # Returns
    ///
    /// - An action to be processed or none.
    #[allow(unused_variables)]
    fn handle_mouse_events(
        &mut self,
        mouse_event: MouseEvent,
    ) -> Result<Option<Action>, ComponentError> {
        Ok(None)
    }

    /// Update the state of the component based on a received action.
    ///
    /// # Arguments
    ///
    /// - `action` - An [`Action`] that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// - An action to be processed or none.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>, ComponentError> {
        Ok(None)
    }

    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// - `f` - A [frame][Frame] used for rendering.
    /// - `area` - The [area][ratatui::layout::Rect] in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// - An Ok result or an error.
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<(), ComponentError>;
}

#[derive(thiserror::Error, Debug)]
pub enum ComponentError {
    #[error(transparent)]
    BoxedError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    EyreError(#[from] color_eyre::eyre::Report),
}
