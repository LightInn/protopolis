mod action;
mod agent;
mod app;
mod config;
mod message;
mod personality;
mod prompt;
mod simulation;
mod state;

use crate::app::App;
use crate::simulation::{Simulation, SimulationEvent};
use cli_log::*;
use color_eyre::eyre::Result;
use std::sync::{Arc, Mutex, RwLock};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

fn main() -> Result<()> {
    color_eyre::install()?;
    init_cli_log!();

    time!(Debug, "main");
    info!("count is {}", 423);
    debug!("data: {:#?}", (423, "forty-two"));
    warn!("this application does nothing");
    log_mem(Level::Info);
    info!("bye");

    // Créer un runtime tokio
    let rt = Runtime::new().unwrap();

    // Initialiser le terminal
    let mut terminal = ratatui::init();

    // Créer les canaux pour communiquer entre la simulation et l'UI
    let (event_sender, mut event_receiver) = mpsc::channel::<SimulationEvent>(100);

    // Créer l'app avec le récepteur d'événements
    let app = Arc::new(Mutex::new(App::default()));

    // Créer une référence à l'app pour le thread de la simulation
    let app_clone = app.clone();

    // Démarrer le thread qui va traiter les événements
    rt.spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            let mut app = app_clone.lock().unwrap();
            match event {
                SimulationEvent::Message(msg) => {
                    app.add_message(msg);
                }
                SimulationEvent::StateChange(msg) => {
                    app.add_message(format!("État: {}", msg));
                }
                SimulationEvent::Finished => {
                    app.add_message("Simulation terminée".to_string());
                }
            }
        }
    });

    // Exécuter l'application
    let app_result = {
        let mut app = app.lock().unwrap();
        app.set_event_sender(event_sender);
        app.run(&mut terminal)
    };

    // Restaurer le terminal
    ratatui::restore();

    // Retourner le résultat
    // app_result

    Ok(())
}
