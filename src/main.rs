// main.rs
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
// main.rs (suite)
use std::thread;

fn main() {
    // Charger la configuration
    let config_path = Path::new("config.json");
    let config = match Config::load(config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Erreur lors du chargement de la configuration: {}", e);
            return;
        }
    };

    // Créer les canaux de communication
    let (ui_tx, sim_rx) = mpsc::channel();
    let (sim_tx, ui_rx) = mpsc::channel();

    // Créer et démarrer la simulation dans un thread séparé
    let simulation_thread = thread::spawn(move || {
        let mut simulation = Simulation::new(config, sim_tx, sim_rx);
        simulation.run();
    });

    // Créer et démarrer l'interface utilisateur
    let mut ui = UI::new(ui_tx, ui_rx);
    ui.run();

    // Attendre que le thread de simulation se termine
    if let Err(e) = simulation_thread.join() {
        eprintln!("Erreur lors de la jointure du thread de simulation: {:?}", e);
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_ui_to_simulation_channel() {
        let (ui_tx, sim_rx) = mpsc::channel::<UIToSimulation>();

        ui_tx.send(UIToSimulation::Start).unwrap();

        match sim_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(UIToSimulation::Start) => assert!(true),
            _ => panic!("Le message UIToSimulation::Start n'a pas été reçu"),
        }
    }
}
