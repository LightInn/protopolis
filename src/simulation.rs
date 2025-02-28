// simulation.rs
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use std::thread;

use crate::agent::Agent;
use crate::config::Config;
use crate::message::Message;

// Messages de l'UI vers la simulation
pub enum UIToSimulation {
    Start,
    Pause,
    Resume,
    Stop,
    SetConfig(String),
}

// Messages de la simulation vers l'UI
pub enum SimulationToUI {
    Status(String),
    AgentUpdate(String, String, u32), // nom, état, énergie
    Messages(Vec<Message>),
    Tick(u64),
}

pub struct Simulation {
    config: Config,
    agents: Vec<Agent>,
    ui_tx: Sender<SimulationToUI>,
    running: bool,
    paused: bool,
    current_tick: u64,
}

impl Simulation {
    pub fn new(config: Config, ui_tx: Sender<SimulationToUI>) -> Self {
        // Créer les agents à partir de la configuration
        let agents = config.agents
            .iter()
            .map(|agent_config| Agent::new(agent_config, config.debug))
            .collect();

        Self {
            config,
            agents,
            ui_tx,
            running: false,
            paused: false,
            current_tick: 0,
        }
    }

    pub fn run(&mut self, rx: Receiver<UIToSimulation>) {
        // Envoyer un statut initial
        let _ = self.ui_tx.send(SimulationToUI::Status("Simulation prête".to_string()));

        // Boucle principale
        loop {
            // Vérifier les messages de l'UI
            if let Ok(message) = rx.try_recv() {
                match message {
                    UIToSimulation::Start => {
                        self.running = true;
                        self.paused = false;
                        let _ = self.ui_tx.send(SimulationToUI::Status("Simulation démarrée".to_string()));
                    }
                    UIToSimulation::Pause => {
                        self.paused = true;
                        let _ = self.ui_tx.send(SimulationToUI::Status("Simulation en pause".to_string()));
                    }
                    UIToSimulation::Resume => {
                        self.paused = false;
                        let _ = self.ui_tx.send(SimulationToUI::Status("Simulation reprise".to_string()));
                    }
                    UIToSimulation::Stop => {
                        self.running = false;
                        let _ = self.ui_tx.send(SimulationToUI::Status("Simulation arrêtée".to_string()));
                        break;
                    }
                    UIToSimulation::SetConfig(config_str) => {
                        // Traiter la configuration
                        let _ = self.ui_tx.send(SimulationToUI::Status(format!("Configuration reçue: {}", config_str)));
                    }
                }
            }

            // Si la simulation est en cours et non en pause
            if self.running && !self.paused {
                // Incrémenter le tick
                self.current_tick += 1;

                // Mettre à jour les agents
                let mut all_messages = Vec::new();

                for agent in &mut self.agents {
                    agent.update(self.current_tick);

                    // Envoyer les mises à jour d'état
                    let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                        agent.name.clone(),
                        format!("{:?}", agent.get_state()),
                        agent.get_energy() as u32
                    ));

                    // Collecter les messages
                    all_messages.extend(agent.get_messages());
                }

                // Envoyer tous les messages à l'UI
                if !all_messages.is_empty() {
                    let _ = self.ui_tx.send(SimulationToUI::Messages(all_messages));
                }

                // Envoyer le tick actuel
                let _ = self.ui_tx.send(SimulationToUI::Tick(self.current_tick));

                // Attendre avant le prochain tick
                thread::sleep(Duration::from_millis(1000 / self.config.world.ticks_per_hour as u64));
            } else {
                // Si en pause ou non démarré, attendre un peu avant de vérifier à nouveau
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
