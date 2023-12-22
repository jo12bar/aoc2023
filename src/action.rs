use std::fmt;

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};

/// Actions sent between TUI components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    Help,
}

/// We have to perform some custom handling while deserializing actions to
/// reconstitute [`Action::Resize`], [`Action::Error`], etc. This does that
/// by using a custom [`serde::de::Visitor`].
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ActionVisitor)
    }
}

struct ActionVisitor;

impl<'de> Visitor<'de> for ActionVisitor {
    type Value = Action;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid string representation of Action")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            "Tick" => Ok(Action::Tick),
            "Render" => Ok(Action::Render),
            "Suspend" => Ok(Action::Suspend),
            "Resume" => Ok(Action::Resume),
            "Quit" => Ok(Action::Quit),
            "Refresh" => Ok(Action::Refresh),
            "Help" => Ok(Action::Help),

            // Reconstitute error messages
            data if data.starts_with("Error(") => {
                let error_msg = data.trim_start_matches("Error(").trim_end_matches(')');
                Ok(Action::Error(error_msg.to_string()))
            },

            // Reconstitute resize action, and validate its serialized format
            data if data.starts_with("Resize(") => {
                let parts: Vec<&str> = data
                    .trim_start_matches("Resize(")
                    .trim_end_matches(')')
                    .split(',')
                    .collect();
                if parts.len() == 2 {
                    let width: u16 = parts[0].trim().parse().map_err(E::custom)?;
                    let height: u16 = parts[1].trim().parse().map_err(E::custom)?;
                    Ok(Action::Resize(width, height))
                } else {
                    Err(E::custom(format!("invalid Resize(w, h) format: {v}")))
                }
            },

            _ => Err(E::custom(format!("unknown Action variant: {v}"))),
        }
    }
}
