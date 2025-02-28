// action.rs
use crate::state::AgentState;

#[derive(Debug, Clone)]
pub enum Action {
    Think { topic: String },
    Sleep { duration: u32 },
    Speak { message: String, target: Option<String> },
    Listen { duration: u32 },
    Move { direction: (i32, i32) },
}

pub struct ActionResult {
    pub new_state: AgentState,
    pub energy_delta: f32,
    pub message: Option<String>,
}

pub struct ActionHandler;

impl ActionHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, action: &Action) -> Result<ActionResult, String> {
        match action {
            Action::Think { topic } => {
                Ok(ActionResult {
                    new_state: AgentState::Thinking,
                    energy_delta: -0.5,
                    message: Some(format!("Thinking about {}", topic)),
                })
            }
            Action::Sleep { duration } => {
                Ok(ActionResult {
                    new_state: AgentState::Sleeping,
                    energy_delta: *duration as f32 * 0.2,
                    message: Some(format!("Sleeping for {} ticks", duration)),
                })
            }
            Action::Speak { message, target } => {
                let target_str = target.as_ref().map_or("everyone".to_string(), |t| t.clone());
                Ok(ActionResult {
                    new_state: AgentState::Speaking,
                    energy_delta: -1.0,
                    message: Some(format!("Speaking to {}: {}", target_str, message)),
                })
            }
            Action::Listen { duration } => {
                Ok(ActionResult {
                    new_state: AgentState::Listening,
                    energy_delta: -0.3,
                    message: Some(format!("Listening for {} ticks", duration)),
                })
            }
            Action::Move { direction } => {
                Ok(ActionResult {
                    new_state: AgentState::Moving,
                    energy_delta: -1.5,
                    message: Some(format!("Moving in direction ({}, {})", direction.0, direction.1)),
                })
            }
        }
    }
}
