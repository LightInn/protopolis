// main.rs

// Module declarations
mod agent;
mod config;
mod conversation_manager;
mod message;
mod personality;
mod simulation;
mod state;
mod ui;

use crate::config::Config;
use crate::simulation::Simulation;
use crate::ui::UI;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::io::{self, Write};

fn main() {
    // Load configuration file
    let config_path = Path::new("config.json");
    let mut config = match Config::load(config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            let config = config::Config::default();
            let _ = config.save(Path::new("config.json"));
            config
        }
    };

    if config.ollama_model.is_none() {
        println!("No Ollama model configured. Please choose a model from the list below:");
        let output = std::process::Command::new("ollama")
            .arg("list")
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let models: Vec<String> = stdout
                        .lines()
                        .skip(1) // Skip header line
                        .filter_map(|line| line.split_whitespace().next().map(String::from))
                        .collect();

                    if models.is_empty() {
                        eprintln!("No Ollama models found. Please ensure Ollama is running and models are installed.");
                        // Optionally, set a default or exit
                        config.ollama_model = Some("default".to_string()); // Or handle error appropriately
                    } else {
                        for (i, model_name) in models.iter().enumerate() {
                            println!("{}: {}", i + 1, model_name);
                        }
                        loop {
                            print!("Select model number: ");
                            io::stdout().flush().unwrap();
                            let mut selection = String::new();
                            io::stdin().read_line(&mut selection).unwrap();
                            match selection.trim().parse::<usize>() {
                                Ok(n) if n > 0 && n <= models.len() => {
                                    config.ollama_model = Some(models[n - 1].clone());
                                    if let Err(e) = config.save(config_path) {
                                        eprintln!("Error saving configuration: {}", e);
                                    }
                                    println!("Selected model: {}", models[n - 1]);
                                    break;
                                }
                                _ => {
                                    println!("Invalid selection. Please try again.");
                                }
                            }
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Error listing Ollama models: {}", stderr);
                    // Optionally, set a default or exit
                    config.ollama_model = Some("default".to_string()); // Or handle error appropriately
                }
            }
            Err(e) => {
                eprintln!("Failed to execute 'ollama list': {}. Please ensure Ollama is installed and in your PATH.", e);
                // Optionally, set a default or exit
                config.ollama_model = Some("default".to_string()); // Or handle error appropriately
            }
        }
    }

    // Create communication channels
    let (ui_tx, sim_rx) = mpsc::channel();
    let (sim_tx, ui_rx) = mpsc::channel();

    // Spawn the simulation thread
    let simulation_thread = thread::spawn(move || {
        let mut simulation = Simulation::new(config, sim_tx, sim_rx);
        simulation.run();
    });

    // Initialize and start the user interface
    let mut ui = UI::new(ui_tx, ui_rx);
    if let Err(err) = ui.run() {
        eprintln!("Error running UI: {}", err);
    }

    // Wait for the simulation thread to finish
    if let Err(e) = simulation_thread.join() {
        eprintln!("Error joining the simulation thread: {:?}", e);
    }
}
