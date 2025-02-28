// simulation.rs
use crate::agent::Agent;
use crate::config::Config;
use crate::message::{Message, MessageContent};
use crate::personality::get_personality_template;
use crate::state::AgentState;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use uuid::Uuid;

pub enum UIToSimulation {
    Start,
    Pause,
    Resume,
    Stop,
    SetDiscussionTopic(String),
}

pub enum SimulationToUI {
    TickUpdate(u64),
    AgentUpdate(String, AgentState, f32),
    MessageUpdate(Message),
    StateUpdate(String),
}

pub struct Simulation {
    config: Config,
    agents: HashMap<String, Agent>,
    messages: Vec<Message>,
    current_tick: u64,
    running: bool,
    ui_tx: Sender<SimulationToUI>,
    sim_rx: Receiver<UIToSimulation>,
    discussion_topic: Option<String>,
    runtime: Runtime,
}

impl Simulation {
    pub fn new(
        config: Config,
        ui_tx: Sender<SimulationToUI>,
        sim_rx: Receiver<UIToSimulation>,
    ) -> Self {
        // Créer un runtime Tokio pour les appels asynchrones à Ollama
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        // Initialiser les agents à partir de la configuration
        let mut agents = HashMap::new();
        for agent_config in &config.agents {
            let id = Uuid::new_v4().to_string();
            let personality = get_personality_template(&agent_config.personality_template);

            let mut agent = Agent::new(
                id.clone(),
                agent_config.name.clone(),
                personality,
                agent_config.initial_energy,
                agent_config.initial_position,
            );

            // Définir le modèle Ollama (on pourrait l'ajouter dans la config)
            agent.set_model("llama3.2:latest".to_string());

            agents.insert(id, agent);
        }

        Self {
            config,
            agents,
            messages: Vec::new(),
            current_tick: 0,
            running: false,
            ui_tx,
            sim_rx,
            discussion_topic: None,
            runtime,
        }
    }

    pub fn run(&mut self) {
        // Attendre le signal de démarrage
        while let Ok(command) = self.sim_rx.recv() {
            match command {
                UIToSimulation::Start => {
                    self.running = true;
                    break;
                }
                UIToSimulation::SetDiscussionTopic(topic) => {
                    self.discussion_topic = Some(topic.clone());
                    // Envoyer un message de mise à jour à l'UI
                    let _ = self.ui_tx.send(SimulationToUI::StateUpdate(
                        format!("Sujet de discussion défini: {}", topic)
                    ));
                    // Démarrer la conversation immédiatement si le sujet est défini
                    self.start_conversation(&topic);
                }
                _ => continue,
            }
        }

        // Boucle principale de simulation
        let mut last_tick_time = Instant::now();
        let tick_duration = Duration::from_millis(1000 / 10); // 10 ticks par seconde

        while self.running {
            // Vérifier les commandes de l'UI
            if let Ok(command) = self.sim_rx.try_recv() {
                match command {
                    UIToSimulation::Pause => self.running = false,
                    UIToSimulation::Resume => self.running = true,
                    UIToSimulation::Stop => break,
                    UIToSimulation::SetDiscussionTopic(topic) => {
                        self.discussion_topic = Some(topic.clone());
                        self.start_conversation(&topic);
                    }
                    _ => {}
                }
            }

            // Si en pause, attendre
            if !self.running {
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            // Vérifier si c'est le moment de faire un tick
            let now = Instant::now();
            if now.duration_since(last_tick_time) >= tick_duration {
                self.tick();
                last_tick_time = now;
            } else {
                // Attendre un peu pour ne pas surcharger le CPU
                thread::sleep(Duration::from_millis(10));
            }
        }

        // Envoyer un message final à l'UI
        let _ = self.ui_tx.send(SimulationToUI::StateUpdate("Simulation arrêtée".to_string()));
    }

    fn tick(&mut self) {
        self.current_tick += 1;

        // Envoyer une mise à jour du tick à l'UI
        let _ = self.ui_tx.send(SimulationToUI::TickUpdate(self.current_tick));

        // Traiter les messages en attente
        let mut new_messages = Vec::new();

        // Collecter les messages à traiter
        let messages_to_process: Vec<(String, Message)> = self.messages.iter()
            .filter(|msg| {
                // Filtrer les messages qui n'ont pas encore été traités
                let recipient = &msg.recipient;
                self.agents.values().any(|agent| agent.name == *recipient)
            })
            .map(|msg| {
                // Trouver l'ID de l'agent destinataire
                let recipient_id = self.agents.iter()
                    .find(|(_, agent)| agent.name == msg.recipient)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_default();

                (recipient_id, msg.clone())
            })
            .collect();

        // Traiter chaque message
        for (agent_id, message) in messages_to_process {
            if let Some(agent) = self.agents.get_mut(&agent_id) {
                if let Some(response) = agent.process_message(&message, &self.runtime) {
                    new_messages.push(response);

                    // Mettre à jour l'état de l'agent dans l'UI
                    let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                        agent.name.clone(),
                        agent.state.clone(),
                        agent.energy
                    ));
                }
            }
        }

        // Ajouter les nouveaux messages
        for message in new_messages {
            // Envoyer le message à l'UI
            let _ = self.ui_tx.send(SimulationToUI::MessageUpdate(message.clone()));
            self.messages.push(message);
        }

        // Mettre à jour l'énergie des agents
        for (id, agent) in self.agents.iter_mut() {
            // Régénération passive d'énergie
            agent.energy += 0.1;
            if agent.energy > 100.0 {
                agent.energy = 100.0;
            }

            // Envoyer une mise à jour à l'UI
            let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                agent.name.clone(),
                agent.state.clone(),
                agent.energy
            ));
        }
    }

    fn start_conversation(&mut self, topic: &str) {
        // Choisir un agent pour commencer la conversation
        if let Some((starter_id, starter)) = self.agents.iter().next() {
            // Choisir un destinataire aléatoire différent de l'expéditeur
            let recipient = self.agents.iter()
                .find(|(id, _)| *id != starter_id)
                .map(|(_, agent)| agent.name.clone())
                .unwrap_or_else(|| "everyone".to_string());

            // Créer un message initial
            let initial_message = Message {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                sender: "Système".to_string(),
                recipient: starter.name.clone(),
                content: json!(format!("Parlons de {}. Qu'en penses-tu?", topic)),
            };

            // Ajouter le message à la liste
            self.messages.push(initial_message.clone());

            // Envoyer le message à l'UI
            let _ = self.ui_tx.send(SimulationToUI::MessageUpdate(initial_message));
            let _ = self.ui_tx.send(SimulationToUI::StateUpdate(
                format!("Conversation démarrée sur le sujet: {}", topic)
            ));
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn setup_simulation() -> (Simulation, Sender<UIToSimulation>, Receiver<SimulationToUI>) {
        let config = Config::default(); // Assurez-vous d'avoir une implémentation par défaut pour les tests
        let (ui_tx, ui_rx) = mpsc::channel();
        let (sim_tx, sim_rx) = mpsc::channel();
        let simulation = Simulation::new(config, ui_tx, sim_rx);
        (simulation, sim_tx, ui_rx)
    }

    #[test]
    fn test_tick_updates() {
        let (mut simulation, sim_tx, ui_rx) = setup_simulation();
        sim_tx.send(UIToSimulation::Start).unwrap();

        std::thread::spawn(move || {
            simulation.run();
        });

        let response = ui_rx.recv_timeout(Duration::from_secs(1));
        assert!(matches!(response, Ok(SimulationToUI::TickUpdate(_))));
    }
}
