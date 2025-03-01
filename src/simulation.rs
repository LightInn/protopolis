// simulation.rs
use crate::agent::Agent;
use crate::config::Config;
use crate::conversation_manager::ConversationManager;
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
    UserMessage(String, String),
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
    conversation_manager: ConversationManager,
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
            conversation_manager: ConversationManager::new(),
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
                    let _ = self.ui_tx.send(SimulationToUI::StateUpdate(format!(
                        "Sujet de discussion défini: {}",
                        topic
                    )));
                    // Démarrer la conversation immédiatement si le sujet est défini
                    self.start_conversation(&topic);
                }
                UIToSimulation::UserMessage(recipient, content) => {
                    self.handle_user_message(&recipient, &content);
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
        let _ = self.ui_tx.send(SimulationToUI::StateUpdate(
            "Simulation arrêtée".to_string(),
        ));
    }

    fn tick(&mut self) {
        self.current_tick += 1;
        let _ = self
            .ui_tx
            .send(SimulationToUI::TickUpdate(self.current_tick));

        // 1. Collecter tous les messages reçus dans ce tick
        for message in &self.messages {
            // Ajouter à l'historique global
            self.conversation_manager.add_message(message.clone());

            // Pour chaque agent (sauf l'expéditeur), collecter ce qu'il "entend"
            for (_, agent) in self.agents.iter_mut() {
                if agent.name != message.sender {
                    // L'agent entend ce message
                    agent.next_prompt.push_str(&format!(
                        "[{}→{}]: {}\n",
                        message.sender,
                        message.recipient,
                        message.content.to_string().trim_matches('"')
                    ));
                }
            }

            // Notifier l'UI
            let _ = self
                .ui_tx
                .send(SimulationToUI::MessageUpdate(message.clone()));
        }

        // 2. Faire répondre les agents qui ont entendu quelque chose
        let mut new_messages = Vec::new();

        for (_, agent) in self.agents.iter_mut() {
            if !agent.next_prompt.is_empty() {
                // L'agent a entendu des messages et va répondre
                agent.state = AgentState::Thinking;

                // Notifier l'UI du changement d'état
                let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                    agent.name.clone(),
                    agent.state.clone(),
                    agent.energy,
                ));

                // Déterminer le destinataire (pour l'instant basique, on répond au dernier message)
                let recipient = if agent.next_prompt.contains("→") {
                    agent
                        .next_prompt
                        .lines()
                        .last()
                        .and_then(|line| line.split('→').next())
                        .unwrap_or("everyone")
                        .trim_start_matches('[')
                        .to_string()
                } else {
                    "everyone".to_string()
                };

                // Générer une réponse
                if let Ok(response_text) = self
                    .runtime
                    .block_on(async { agent.generate_response_from_prompt().await })
                {
                    // Créer un message de réponse
                    let response_message = Message {
                        id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        sender: agent.name.clone(),
                        recipient,
                        content: json!(response_text),
                    };

                    // Ajouter à la liste des nouveaux messages
                    new_messages.push(response_message.clone());

                    // Notifier l'UI
                    let _ = self
                        .ui_tx
                        .send(SimulationToUI::MessageUpdate(response_message));

                    // Mettre à jour l'état de l'agent
                    agent.state = AgentState::Speaking;
                    agent.energy -= 1.0;
                }

                // Réinitialiser le prompt pour le prochain tick
                agent.next_prompt.clear();
            }
        }

        // Vider les messages actuels et ajouter les nouveaux
        self.messages.clear();
        self.messages.extend(new_messages);

        // Mise à jour de l'énergie des agents (code existant)
        for (_, agent) in self.agents.iter_mut() {
            agent.energy += 0.1;
            if agent.energy > 100.0 {
                agent.energy = 100.0;
            }

            let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                agent.name.clone(),
                agent.state.clone(),
                agent.energy,
            ));
        }
    }

    fn start_conversation(&mut self, topic: &str) {
        // Choisir un agent pour commencer la conversation
        if let Some((starter_id, starter)) = self.agents.iter().next() {
            // Choisir un destinataire aléatoire différent de l'expéditeur
            let recipient = self
                .agents
                .iter()
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
            let _ = self
                .ui_tx
                .send(SimulationToUI::MessageUpdate(initial_message));
            let _ = self.ui_tx.send(SimulationToUI::StateUpdate(format!(
                "Conversation démarrée sur le sujet: {}",
                topic
            )));
        }
    }

    fn handle_user_message(&mut self, recipient: &str, content: &str) {
        // Créer un message utilisateur
        let user_message = Message {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            sender: "Utilisateur".to_string(),
            recipient: recipient.to_string(),
            content: json!(content),
        };

        // Notifier l'UI
        let _ = self
            .ui_tx
            .send(SimulationToUI::MessageUpdate(user_message.clone()));

        // Ajouter à l'historique
        self.conversation_manager.add_message(user_message.clone());

        // Ajouter au next_prompt de l'agent destinataire pour traitement immédiat
        if let Some(agent) = self.agents.values_mut().find(|a| a.name == recipient) {
            agent
                .next_prompt
                .push_str(&format!("[Utilisateur→{}]: {}\n", recipient, content));

            // Traiter immédiatement la réponse
            agent.state = AgentState::Thinking;
            let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                agent.name.clone(),
                agent.state.clone(),
                agent.energy,
            ));

            // Stocker le nom de l'agent pour l'utiliser après la libération de l'emprunt
            let agent_name = agent.name.clone();

            // Générer une réponse
            let response_result = self
                .runtime
                .block_on(async { agent.generate_response_from_prompt().await });

            // Libérer l'emprunt de agent en sortant du if let
            if let Ok(response_text) = response_result {
                let response_message = Message {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    sender: agent_name.clone(),
                    recipient: "Utilisateur".to_string(),
                    content: json!(response_text),
                };

                // Notifier l'UI
                let _ = self
                    .ui_tx
                    .send(SimulationToUI::MessageUpdate(response_message));

                // Maintenant on peut emprunter self.agents à nouveau
                for (_, other_agent) in self.agents.iter_mut() {
                    if other_agent.name != agent_name {
                        other_agent.next_prompt.push_str(&format!(
                            "[{}→Utilisateur]: {}\n",
                            agent_name, response_text
                        ));
                    }
                }

                // Mise à jour de l'état de l'agent avec un nouvel emprunt
                if let Some(agent) = self.agents.values_mut().find(|a| a.name == agent_name) {
                    agent.state = AgentState::Speaking;
                    agent.energy -= 1.0;
                    let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                        agent.name.clone(),
                        agent.state.clone(),
                        agent.energy,
                    ));
                }
            }

            // Réinitialiser le prompt (besoin d'un nouvel emprunt)
            if let Some(agent) = self.agents.values_mut().find(|a| a.name == agent_name) {
                agent.next_prompt.clear();
            }
        } else {
            let _ = self.ui_tx.send(SimulationToUI::StateUpdate(format!(
                "Agent '{}' non trouvé.",
                recipient
            )));
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
