// config.rs
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub world: WorldConfig,
    pub agents: Vec<AgentConfig>,
    pub debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub width: i32,
    pub height: i32,
    pub ticks_per_hour: u32,
    pub hours_per_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub personality_template: String,
    pub initial_energy: f32,
    pub initial_position: (i32, i32),
}

impl Config {
    pub fn default() -> Self {
        Self {
            world: WorldConfig {
                width: 100,
                height: 100,
                ticks_per_hour: 60,
                hours_per_day: 24,
            },
            agents: vec![
                AgentConfig {
                    name: "Alice".to_string(),
                    personality_template: "friendly".to_string(),
                    initial_energy: 100.0,
                    initial_position: (10, 10),
                },
                AgentConfig {
                    name: "Bob".to_string(),
                    personality_template: "curious".to_string(),
                    initial_energy: 100.0,
                    initial_position: (20, 20),
                },
                AgentConfig {
                    name: "Charlie".to_string(),
                    personality_template: "cautious".to_string(),
                    initial_energy: 100.0,
                    initial_position: (30, 30),
                },
            ],
            debug: true,
        }
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
