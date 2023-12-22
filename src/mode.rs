use serde::{Deserialize, Serialize};

/// Mode that the application is in. Allows for, e.g., easy switching between screens.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}
