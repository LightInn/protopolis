// main.rs

// Module declarations
mod agent;
mod config;
mod message;
mod personality;
mod simulation;
mod state;
mod ui;
mod conversation_manager;

use crate::config::Config;
use crate::simulation::Simulation;
use crate::ui::UI;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

fn main() {
    // Load configuration file
    let config_path = Path::new("config.json");
    let config = match Config::load(config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            return;
        }
    };

    // Create communication channels
    let (ui_tx, sim_rx) = mpsc::channel(); // Channel for UI -> Simulation messages
    let (sim_tx, ui_rx) = mpsc::channel(); // Channel for Simulation -> UI messages

    // Spawn the simulation thread
    let simulation_thread = thread::spawn(move || {
        let mut simulation = Simulation::new(config, sim_tx, sim_rx);
        simulation.run();
    });

    // Initialize and start the user interface
    let mut ui = UI::new(ui_tx, ui_rx);
    ui.run();

    // Wait for the simulation thread to finish
    if let Err(e) = simulation_thread.join() {
        eprintln!("Error joining the simulation thread: {:?}", e);
    }
}
