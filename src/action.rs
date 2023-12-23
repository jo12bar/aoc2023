use serde::{Deserialize, Serialize};

/// Actions sent between TUI components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Tick the TUI
    Tick,
    /// Re-render the TUI
    Render,
    /// Resize the TUI to `(width, height)` characters.
    Resize(u16, u16),
    /// Suspend the TUI.
    Suspend,
    /// Resume the TUI from a suspended state.
    Resume,
    /// Quit the application.
    Quit,
    /// Refresh the application.
    Refresh,
    /// Send an error to be handled (or potentially to crash the program and printed)
    Error(String),
    /// Get help.
    Help,
    /// Toggle help display.
    ToggleShowHelp,
    /// Schedule increment for the future.
    ScheduleIncrement,
    /// Schedule decrement for the future.
    ScheduleDecrement,
    /// Increment the counter.
    Increment(usize),
    /// Decrement the counter.
    Decrement(usize),
    /// Complete inputting something.
    CompleteInput(String),
    /// Enter normal mode.
    EnterNormal,
    /// Enter insert mode.
    EnterInsert,
    /// Enter processing mode.
    EnterProcessing,
    /// Exit processing mode.
    ExitProcessing,
    /// Update the input.
    Update,
}
