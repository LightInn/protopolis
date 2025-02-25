//main.rs
mod action;
mod agent;
mod app;
mod config;
mod message;
mod personality;
mod prompt;
mod simulation;
mod state;
mod logging;

use crate::app::App;
use crate::simulation::{SimulationEvent, SimulationState};
use cli_log::*;
use color_eyre::eyre::Result;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

async fn cleanup(app: Arc<Mutex<App>>) {
    // Attendre que tous les agents terminent
    let mut app = app.lock().unwrap();
    if let Some(simulation) = &app.simulation {
        let mut sim = simulation.write().await;
        *sim.state.write().await = SimulationState::Finished;

        // Attendre que tous les agents terminent
        for agent in &sim.agents {
            let _ = agent.write().await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    init_cli_log!();
    
    

    info!("Démarrage de l'application");

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

    // tokio::signal::ctrl_c().await?;
    // cleanup(app.clone()).await;

    // Démarrer le thread qui va traiter les événements
    rt.spawn(async move {
        info!("Thread d'écoute des événements démarré");
        while let Some(event) = event_receiver.recv().await {
            let mut app = app_clone.lock().unwrap();
            match event {
                SimulationEvent::Message(msg) => {
                    app.add_message(msg.clone());
                    app.log(format!("Message: {}", msg));
                }
                SimulationEvent::StateChange(msg) => {
                    app.log(format!("Changement d'état: {}", msg));
                }
                SimulationEvent::Finished => {
                    app.add_message("Simulation terminée".to_string());
                    app.log("Simulation terminée".to_string());
                }
            }
        }
        info!("Thread d'écoute des événements terminé");
    });

    {
        let mut app = app.lock().unwrap();
        app.set_event_sender(event_sender);
        app.log("Application initialisée".to_string());
        let result = app.run(&mut terminal);
        ratatui::restore();
        result.await?
    }

 

    info!("Application terminée");
    Ok(())
}
