// config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// World configuration settings
#[derive(Debug, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Ticks per hour in simulation
    pub ticks_per_hour: u32,
    /// World size dimensions
    pub world_size: (i32, i32),
    /// Maximum message broadcast range
    pub max_broadcast_range: i32,
    /// Base energy consumption rate
    pub base_energy_consumption: f32,
}

/// Agent configuration settings
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent's name
    pub name: String,
    /// Personality template to use
    pub personality_template: String,
    /// Initial position
    pub initial_position: (i32, i32),
    /// Initial energy level
    pub initial_energy: f32,
    /// System prompt for the agent
    pub system_prompt: String,
}

/// Global configuration container
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub world: WorldConfig,
    pub agents: Vec<AgentConfig>,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            ticks_per_hour: 60,
            world_size: (1000, 1000),
            max_broadcast_range: 100,
            base_energy_consumption: 0.1,
        }
    }
}

impl Config {
    /// Loads configuration from a JSON file
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Creates a new default configuration
    pub fn default() -> Self {
        Self {
            world: WorldConfig::default(),
            agents: vec![
                AgentConfig {
                    name: "Alice".to_string(),
                    personality_template: "optimistic".to_string(),
                    initial_position: (0, 0),
                    initial_energy: 100.0,
                    system_prompt: "Bob have a terrible secret".to_string(),
                },
                AgentConfig {
                    name: "Bob".to_string(),
                    personality_template: "analytical".to_string(),
                    initial_position: (10, 10),
                    initial_energy: 100.0,
                    system_prompt: "Alice is lying".to_string(),
                },
            ],
        }
    }

    /// Saves configuration to a JSON file
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
