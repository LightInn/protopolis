// config.rs

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use ollama_rs::Ollama;

/// Represents the full configuration of the simulation.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Configuration for the world settings.
    pub world: WorldConfig,

    /// List of agent configurations.
    pub agents: Vec<AgentConfig>,

    /// Debug mode flag (enables additional logging and debugging features).
    pub debug: bool,

    /// The selected model for the simulation.
    pub model: String,
}

/// Defines the world parameters for the simulation.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Width of the simulated world.
    pub width: u32,

    /// Height of the simulated world.
    pub height: u32,

    /// Number of simulation ticks per in-game hour.
    pub ticks_per_hour: u32,

    /// Number of hours in an in-game day.
    pub hours_per_day: u32,
}

/// Defines the configuration of an individual agent.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent's name.
    pub name: String,

    /// The template defining the agent's personality (e.g., "friendly", "curious").
    pub personality_template: String,

    /// Initial energy level of the agent.
    pub initial_energy: f64,

    /// Starting position of the agent in the world (x, y).
    pub initial_position: [u32; 2],
}

impl Config {
    /// Returns a default configuration for the simulation.
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
                    initial_position: [10, 10],
                },
                AgentConfig {
                    name: "Bob".to_string(),
                    personality_template: "curious".to_string(),
                    initial_energy: 100.0,
                    initial_position: [20, 20],
                },
                AgentConfig {
                    name: "Charlie".to_string(),
                    personality_template: "cautious".to_string(),
                    initial_energy: 100.0,
                    initial_position: [30, 30],
                },
            ],
            debug: true,
            model: "llama2".to_string(),
        }
    }

    /// Loads a configuration from a JSON file.
    ///
    /// # Arguments
    /// * `path` - The file path to load the configuration from.
    ///
    /// # Returns
    /// * `Ok(Config)` if the file is successfully read and parsed.
    /// * `Err(Box<dyn std::error::Error>)` if an error occurs.
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Saves the current configuration to a JSON file.
    ///
    /// # Arguments
    /// * `path` - The file path to save the configuration to.
    ///
    /// # Returns
    /// * `Ok(())` if the file is successfully written.
    /// * `Err(Box<dyn std::error::Error>)` if an error occurs.
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    /// Selects a model from the available Ollama models.
    ///
    /// # Returns
    /// * `Ok(String)` if a valid model is selected.
    /// * `Err(Box<dyn std::error::Error>)` if an error occurs.
    pub async fn select_model() -> Result<String, Box<dyn std::error::Error>> {
        let ollama = Ollama::default();
        let models = ollama.list_local_models().await?;
        
        println!("Modèles Ollama disponibles:");
        for (i, model) in models.iter().enumerate() {
            println!("{}. {}", i + 1, model.name);
        }

        let mut input = String::new();
        println!("\nSélectionnez un modèle (entrez le numéro):");
        std::io::stdin().read_line(&mut input)?;

        let selection: usize = input.trim().parse()?;
        if selection > 0 && selection <= models.len() {
            Ok(models[selection - 1].name.clone())
        } else {
            Err("Sélection invalide".into())
        }
    }
}
