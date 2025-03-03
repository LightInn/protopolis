use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the different states an agent can be in.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentState {
    /// The agent is inactive and waiting.
    Idle,

    /// The agent is processing information.
    Thinking,

    /// The agent is actively speaking.
    Speaking,

    /// The agent is listening for input.
    Listening,

    /// The agent is in a resting state (e.g., cooldown or inactivity).
    Resting,
}

impl fmt::Display for AgentState {
    /// Converts an `AgentState` into a human-readable string.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state_str = match self {
            AgentState::Idle => "Idle",
            AgentState::Thinking => "Thinking",
            AgentState::Speaking => "Speaking",
            AgentState::Listening => "Listening",
            AgentState::Resting => "Resting",
        };
        write!(f, "{}", state_str)
    }
}
