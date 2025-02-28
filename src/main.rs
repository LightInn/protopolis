use std::path::Path;
use crate::config::Config;
use crate::simulation::Simulation;
use crate::ui::UI;

// main.rs
mod action;
mod agent;
mod config;
mod message;
mod personality;
mod simulation;
mod state;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger la configuration
    let config = match Config::load(Path::new("config.json")) {
        Ok(config) => config,
        Err(_) => {
            let default_config = Config::default();
            default_config.save(Path::new("config.json"))?;
            default_config
        }
    };

    // Créer les canaux de communication entre UI et simulation
    let (ui_tx, ui_rx) = std::sync::mpsc::channel();
    let (sim_tx, sim_rx) = std::sync::mpsc::channel();

    // Démarrer la simulation dans un thread séparé
    let sim_config = config.clone();
    let simulation_handle = std::thread::spawn(move || {
        let mut simulation = Simulation::new(sim_config, ui_tx);
        simulation.run(sim_rx);
    });

    // Démarrer l'interface utilisateur dans le thread principal
    let mut ui = UI::new(sim_tx, ui_rx)?;
    ui.run()?;

    // Attendre que le thread de simulation se termine
    simulation_handle.join().unwrap();

    Ok(())
}
