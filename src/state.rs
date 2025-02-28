// state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Thinking,
    Speaking,
    Listening,
    Sleeping,
    Moving,
    Eating,
}
