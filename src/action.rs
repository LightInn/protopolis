// action.rs
use crate::state::AgentState;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Represents all possible actions an agent can take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Think {
        topic: String,
    },
    Listen {
        duration: u32,
    },
    Speak {
        message: String,
        target: Option<String>,
    },
    Move {
        x: i32,
        y: i32,
    },
    Sleep {
        duration: u32,
    },
    CheckTime,
}

/// Result of an action execution
#[derive(Debug)]
pub struct ActionResult {
    pub success: bool,
    pub new_state: AgentState,
    pub energy_delta: f32,
    pub message: Option<String>,
}

/// Handles the execution of actions
#[derive(Debug)]
pub struct ActionHandler {
    /// Energy cost for each action
    energy_costs: std::collections::HashMap<String, f32>,
}

impl ActionHandler {
    /// Creates a new action handler with default energy costs
    pub fn new() -> Self {
        let mut energy_costs = std::collections::HashMap::new();
        energy_costs.insert("Think".to_string(), -2.0);
        energy_costs.insert("Listen".to_string(), -1.0);
        energy_costs.insert("Speak".to_string(), -3.0);
        energy_costs.insert("Move".to_string(), -5.0);
        energy_costs.insert("Sleep".to_string(), 10.0);
        energy_costs.insert("CheckTime".to_string(), -0.5);

        Self { energy_costs }
    }

    /// Executes an action and returns the result
    pub fn execute(&self, action: &Action) -> Result<ActionResult, Box<dyn Error>> {
        match action {
            Action::Think { topic } => Ok(ActionResult {
                success: true,
                new_state: AgentState::Thinking,
                energy_delta: *self.energy_costs.get("Think").unwrap(),
                message: Some(format!("Thinking about: {}", topic)),
            }),
            Action::Listen { duration } => Ok(ActionResult {
                success: true,
                new_state: AgentState::Listening,
                energy_delta: *self.energy_costs.get("Listen").unwrap() * (*duration as f32),
                message: Some(format!("Listening for {} ticks", duration)),
            }),
            Action::Speak { message, target } => {
                let target_str = target
                    .as_ref()
                    .map_or("everyone".to_string(), |t| t.clone());
                Ok(ActionResult {
                    success: true,
                    new_state: AgentState::Speaking,
                    energy_delta: *self.energy_costs.get("Speak").unwrap(),
                    message: Some(format!("Speaking to {}: {}", target_str, message)),
                })
            }
            Action::Move { x, y } => Ok(ActionResult {
                success: true,
                new_state: AgentState::Moving,
                energy_delta: *self.energy_costs.get("Move").unwrap(),
                message: Some(format!("Moving to position ({}, {})", x, y)),
            }),
            Action::Sleep { duration } => Ok(ActionResult {
                success: true,
                new_state: AgentState::Sleeping,
                energy_delta: *self.energy_costs.get("Sleep").unwrap() * (*duration as f32),
                message: Some(format!("Sleeping for {} ticks", duration)),
            }),
            Action::CheckTime => Ok(ActionResult {
                success: true,
                new_state: AgentState::TimeChecking,
                energy_delta: *self.energy_costs.get("CheckTime").unwrap(),
                message: Some("Checking current time".to_string()),
            }),
        }
    }

    /// Validates if an action can be performed based on available energy
    pub fn can_perform(&self, action: &Action, current_energy: f32) -> bool {
        let cost = match action {
            Action::Think { .. } => *self.energy_costs.get("Think").unwrap(),
            Action::Listen { duration } => {
                *self.energy_costs.get("Listen").unwrap() * (*duration as f32)
            }
            Action::Speak { .. } => *self.energy_costs.get("Speak").unwrap(),
            Action::Move { .. } => *self.energy_costs.get("Move").unwrap(),
            Action::Sleep { .. } => 0.0, // Always allow sleep
            Action::CheckTime => *self.energy_costs.get("CheckTime").unwrap(),
        };

        current_energy + cost >= 0.0
    }
}
