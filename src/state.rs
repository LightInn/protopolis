// state.rs

use serde::{Deserialize, Serialize};

/// Represents the different states an agent can be in
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum AgentState {
    Thinking,     // Agent is processing information
    Listening,    // Agent is waiting for messages
    Speaking,     // Agent is sending a message
    Moving,       // Agent is changing position
    Sleeping,     // Agent is in rest mode
    TimeChecking, // Agent is checking world time
    #[default]
    Idle,
}

/// Holds the current state and related data for an agent
#[derive(Debug)]
pub struct AgentStateManager {
    /// Current state of the agent
    current_state: AgentState,
    /// Time spent in current state
    state_duration: u32,
    /// Energy level (0-100)
    energy: f32,
    /// Position in the world
    position: (i32, i32),
}

impl AgentStateManager {
    /// Creates a new state manager with default values
    pub fn new() -> Self {
        Self {
            current_state: AgentState::Listening,
            state_duration: 0,
            energy: 100.0,
            position: (0, 0),
        }
    }

    /// Updates the current state
    pub fn set_state(&mut self, new_state: AgentState) {
        self.current_state = new_state;
        self.state_duration = 0;
    }

    /// Returns the current state
    pub fn get_state(&self) -> &AgentState {
        &self.current_state
    }

    /// Updates energy level
    pub fn update_energy(&mut self, delta: f32) {
        self.energy = (self.energy + delta).clamp(0.0, 100.0);
    }

    /// Returns current energy level
    pub fn get_energy(&self) -> f32 {
        self.energy
    }

    /// Updates position
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = (x, y);
    }

    /// Returns current position
    pub fn get_position(&self) -> (i32, i32) {
        self.position
    }

    /// Increments state duration
    pub fn tick(&mut self) {
        self.state_duration += 1;
    }
}
