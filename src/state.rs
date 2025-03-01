use std::fmt;
// state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentState {
    Idle,
    Thinking,
    Speaking,
    Listening,
    Resting,
}


impl fmt::Display for AgentState {
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