// world.rs
use colored::*;
use crate::agent::{Agent, Message};
use ollama_rs::Ollama;
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use crate::utils;

/// Représente le monde dans lequel les agents évoluent
pub struct World {
    /// Liste des agents actifs dans le monde
    agents: Vec<Agent>,
    /// File d'attente des messages à traiter
    message_queue: Vec<Message>,
    /// Compteur d'itérations (représente le temps dans le monde)
    iteration: u32,
    /// Instance d'Ollama pour les interactions IA
    ollama: Ollama,
    /// Thème actuel de la discussion
    current_topic: String,
    /// Mémoire globale du monde (pour partager des informations entre agents)
    global_memory: HashMap<String, Value>,
}

impl World {
    /// Crée une nouvelle instance du monde
    pub fn new(ollama: Ollama) -> Self {
        Self {
            agents: vec![],
            message_queue: vec![],
            iteration: 0,
            ollama,
            current_topic: String::new(),
            global_memory: HashMap::new(),
        }
    }

    /// Ajoute un agent au monde
    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
    }

    /// Définit le thème initial de la discussion
    pub fn set_initial_topic(&mut self) {
        // Entrée utilisateur pour le thème
        let theme = utils::get_user_input("World Goal:");

        self.current_topic = theme.to_string();
        self.broadcast_system_message(Message {
            sender: "System".to_string(),
            recipient: "Everybody".to_string(),
            content: format!("World goal: {}", theme),
            timestamp: self.iteration,
        });
    }

    /// Retourne une référence aux agents (pour sauvegarde, etc.)
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Boucle principale de simulation
    pub async fn run(&mut self) {
        println!("Simulation Starting...");
        println!("Topic: {}", self.current_topic);

        while self.iteration < 20 {
            self.iteration += 1;
            println!("\n=== Cycle {} ===", self.iteration);

            // Mise à jour de l'état du monde
            self.update_world_state();

            // Traitement des messages
            self.process_messages().await;

            // Mise à jour des agents
            for agent in &mut self.agents {
                agent.update_world_state(self.iteration);
                println!("debug his memory: {:?}", agent.memory);
                println!("debug his conversation: {:?}", agent.conversation);
            }

            // Pause entre les cycles
            sleep(Duration::from_secs(1)).await;
        }

        println!("{}", "[FIN DE SIMULATION]".bright_red().bold());
    }

    /// Met à jour l'état global du monde
    fn update_world_state(&mut self) {
        // Exemple : synthétiser la mémoire globale tous les 5 cycles
        if self.iteration % 5 == 0 {
            self.synthesize_global_memory();
        }
    }

    /// Traite les messages en attente
    async fn process_messages(&mut self) {
        // Distribuer les messages actuels
        let messages = std::mem::take(&mut self.message_queue);

        // Collecter les nouvelles réponses
        let mut new_responses = Vec::new();

        for agent in &mut self.agents {
            if let Some(response) = agent.process_messages(&messages, &self.ollama).await {
                println!("[{}] {}", response.sender, response.content);
                new_responses.push(response);
            }
        }

        // Ajouter les nouvelles réponses à la file d'attente
        self.message_queue = new_responses;
    }

    /// Diffuse un message à tous les agents
    fn broadcast_message(&mut self, message: Message, radius: i32, sender: &Agent) {

        // todo : la message_queue devraits etre par agent, pour que chaque agent puisse avoir sa propre file de message
        // boradcast message to all agents inside the radius
        let sender_position = &sender.position;

        for agent in &mut self.agents {
            if agent.position.distance_square(sender_position) <= radius {
                agent.message_queue.push_back(message.clone());
            }
        }

    }

    fn broadcast_system_message(&mut self, message: Message) {
        for agent in &mut self.agents {
            agent.message_queue.push_back(message.clone());
        }
    }

    /// Synthétise la mémoire globale
    fn synthesize_global_memory(&mut self) {
        println!(
            "{} Synthèse de la mémoire globale...",
            "[MÉMOIRE]".bright_yellow().bold()
        );

        // Exemple : regrouper les informations importantes
        let mut summary = HashMap::new();
        summary.insert("iteration".to_string(), Value::from(self.iteration));

        // Correction ici : utiliser `self.current_topic.as_str()` pour convertir en &str
        summary.insert("topic".to_string(), Value::from(self.current_topic.as_str()));

        // Ajouter des informations des agents
        for agent in &self.agents {
            summary.insert(
                format!("agent_{}_summary", agent.id),
                Value::from(agent.summarize_memory()),
            );
        }

        self.global_memory = summary;

    }

    /// Sauvegarde l'état actuel du monde
    pub async fn save_state(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(file_path)?;
        let state = serde_json::to_string_pretty(&self.global_memory)?;
        file.write_all(state.as_bytes())?;

        Ok(())
    }

    /// Charge un état précédent du monde
    pub async fn load_state(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        let data = fs::read_to_string(file_path)?;
        self.global_memory = serde_json::from_str(&data)?;

        Ok(())
    }
}

// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;
    use ollama_rs::Ollama;

    #[tokio::test]
    async fn test_world_initialization() {
        let ollama = Ollama::default();
        let world = World::new(ollama);

        assert_eq!(world.iteration, 0);
        assert!(world.agents.is_empty());
        assert!(world.message_queue.is_empty());
    }

    #[tokio::test]
    async fn test_adding_agents() {
        let ollama = Ollama::default();
        let mut world = World::new(ollama);

        world.add_agent(Agent::new(1, "Alice", "optimiste"));
        world.add_agent(Agent::new(2, "Bob", "sceptique"));

        assert_eq!(world.agents.len(), 2);
    }

    #[tokio::test]
    async fn test_topic_setting() {
        let ollama = Ollama::default();
        let mut world = World::new(ollama);

        world.set_initial_topic("Philosophie");
        assert_eq!(world.current_topic, "Philosophie");
        assert_eq!(world.message_queue.len(), 1);
    }
}