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

#[tokio::main]
async fn main() {
    // Load configuration file
    let config_path = Path::new("config.json");
    let config = match Config::load(config_path) {
        Ok(config) => config,
        Err(_) => {
            println!("Configuration non trouvée. Création d'une nouvelle configuration...");
            let mut config = Config::default();
            
            // Sélection du modèle
            match Config::select_model().await {
                Ok(model) => {
                    config.model = model;
                    println!("Modèle sélectionné avec succès!");
                }
                Err(e) => {
                    eprintln!("Erreur lors de la sélection du modèle: {}", e);
                    println!("Utilisation du modèle par défaut (llama2)");
                }
            }

            // Sauvegarde de la configuration
            if let Err(e) = config.save(config_path) {
                eprintln!("Erreur lors de la sauvegarde de la configuration: {}", e);
            }
            config
        }
    };

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
        eprintln!("Error in simulation thread: {:?}", e);
    }
}
