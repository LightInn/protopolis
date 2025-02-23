// agent.rs
use colored::Colorize;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;

/// Represent an intelligent agent with a personality and abilities
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    /// Unique identifier of the agent
    pub id: u32,
    /// Agent's name
    pub name: String,
    /// Agent's personality (used in prompts)
    pub personality: String,
    /// Agent's memory (history of events)
    pub memory: Vec<String>,
    /// Current goal of the agent
    pub current_goal: String,
    /// Discussions history
    pub conversation: Vec<ChatMessage>,
    /// Last action performed by the agent
    pub last_action: String,
    /// coordinates of the agent
    pub position: Coord,
    /// Queue of messages received by the agent (to be processed)
    pub message_queue: VecDeque<Message>,
}

/// Represent a message exchanged between agents
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    /// Sender's name
    pub sender: String,
    /// Recipient's name (empty for broadcast)
    pub recipient: String,
    /// Message content
    pub content: String,
    /// Timestamp of the message
    pub timestamp: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub(crate) fn distance_square(&self, p0: &Coord) -> i32 {
        ((self.x - p0.x).pow(2) + (self.y - p0.y).pow(2))
    }
}

impl Agent {
    /// Crée un nouvel agent
    pub fn new(id: u32, name: &str, personality: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            personality: personality.to_string(),
            memory: Vec::new(),
            current_goal: "Engage in meaningful conversation".to_string(),
            conversation: Vec::new(),
            last_action: "Initialized".to_string(),

            // todo : quel vitesse pour les deplacements ? ils ne penses pas assez vite pour se deplacer normalement, et les teleporté casserais l'intimité de la conversation
            position: Coord { x: 0, y: 0 },
            message_queue: Default::default(),
        }
    }

    /// Traite les messages reçus et génère une réponse
    pub async fn process_messages(&mut self, ollama: &Ollama) -> Option<Message> {
        // Récupération des messages
        let messages = &mut self.message_queue.drain(..).collect();

        // Écoute des messages
        self.listen(&messages);

        // Réflexion sur les messages reçus
        self.reflect();

        // Génération d'une réponse
        self.generate_response(ollama).await.ok()
    }

    /// Écoute les messages et les ajoute à la mémoire
    fn listen(&mut self, messages: &[Message]) {
        for message in messages {
            self.memory.push(format!(
                "[Cycle {}] Message de {}: {}",
                message.timestamp, message.sender, message.content
            ));
            self.conversation.push_back(message.clone());

            // Log coloré pour l'écoute
            println!(
                "{} {} {}: {}",
                "[ÉCOUTE]".bright_blue().bold(),
                self.name.bright_green(),
                "a reçu un message de".bright_white(),
                message.sender.bright_yellow()
            );
            println!("   Contenu: {}", message.content.bright_white());
        }
    }

    /// Simule la réflexion de l'agent
    fn reflect(&mut self) {
        let reflection = format!("Réflexion sur {} messages récents", self.conversation.len());
        self.memory.push(reflection.clone());
        self.last_action = "Reflecting".to_string();

        // Log coloré pour la réflexion
        println!(
            "{} {}: {}",
            "[RÉFLEXION]".bright_magenta().bold(),
            self.name.bright_green(),
            reflection.bright_white()
        );
    }

    /// Génère une réponse en utilisant Ollama
    async fn generate_response(
        &mut self,
        ollama: &Ollama,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let prompt = self.build_prompt();
        let mut retries = 3;

        // Log coloré pour la génération de réponse
        println!(
            "{} {}: Génération d'une réponse...",
            "[GÉNÉRATION]".bright_cyan().bold(),
            self.name.bright_green()
        );

        loop {
            let response = ollama
                .generate(GenerationRequest::new(
                    "llama3.2:latest".to_string(),
                    prompt.clone(),
                ))
                .await?;

            if let Ok(parsed) = self.parse_response(&response.response) {
                self.memory.push(format!("Réponse générée: {}", parsed));
                self.last_action = "Speaking".to_string();

                // Log coloré pour la réponse générée
                println!(
                    "{} {}: {}",
                    "[RÉPONSE]".bright_green().bold(),
                    self.name.bright_green(),
                    parsed.bright_white()
                );

                return Ok(Message {
                    sender: self.name.clone(),
                    recipient: "".to_string(),
                    content: parsed,
                    timestamp: 0, // Le monde mettra à jour le timestamp
                });
            }

            retries -= 1;
            if retries == 0 {
                println!(
                    "{} {}: Échec de la génération de réponse",
                    "[ERREUR]".bright_red().bold(),
                    self.name.bright_green()
                );
                return Err("Échec génération JSON valide".into());
            }
        }
    }

    // pub async fn test_message(&mut self, ollama: &mut Ollama, model: String, prompt: String) {
    //
    //
    //
    //     let res = ollama
    //         .send_chat_messages_with_history(
    //             &mut self.conv_channel, // <- messages will be saved here
    //             ChatMessageRequest::new(
    //                 model,
    //                 vec![ChatMessage::user(prompt)], // <- You should provide only one message
    //             ),
    //         )
    //         .await;
    //
    //     if let Ok(res) = res {
    //         println!("{}", res.message.content);
    //     }
    //
    // }

    /// Construit le prompt pour l'agent
    fn build_prompt(&self) -> String {
        format!(
            "You are {name} ({personality}).\n\
            Your goal: {goal}\n\
            Historique conversation:\n{history}\n\
            Formate ta réponse en JSON (uniquement le JSON en sortie brut, rien d'autre, il sera parsé) avec le structure suivante:\n\
            {{\n  \"response\": \"Ta réponse ici\"\n}}\n\
            try to keep your message short until absolutely necessity",
            name = self.name,
            personality = self.personality,
            goal = self.current_goal,
            history = self.conversation_history()
        )
    }

    /// Parse la réponse de l'IA
    fn parse_response(&self, response: &str) -> Result<String, Box<dyn std::error::Error>> {
        // println!("Réponse brute: {}", response);
        let parsed: Value = serde_json::from_str(response)?;
        Ok(parsed["response"]
            .as_str()
            .ok_or("Champ 'response' manquant")?
            .to_string())
    }

    /// Retourne l'historique des conversations formaté
    fn conversation_history(&self) -> String {
        self.conversation
            .iter()
            .map(|msg| format!("{}: {}", msg.sender, msg.content))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Synthétise la mémoire de l'agent
    pub fn summarize_memory(&self) -> String {
        format!(
            "Agent {} ({}):\n\
            Dernière action: {}\n\
            Nombre de messages: {}\n\
            Mémoire récente:\n{}",
            self.name,
            self.personality,
            self.last_action,
            self.conversation.len(),
            self.memory
                .iter()
                .rev()
                .take(3)
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .join("\n")
        )
    }

    /// Met à jour l'état de l'agent en fonction du temps
    pub fn update_world_state(&mut self, current_time: u32) {
        if current_time % 5 == 0 {
            self.sleep();
        }
    }

    /// Simule le sommeil et la synthèse de la mémoire
    pub fn sleep(&mut self) {
        let summary = self.summarize_memory();
        self.memory
            .push("Sommeil et synthèse de la mémoire".to_string());
        self.memory.push(summary.clone());
        self.last_action = "Sleeping".to_string();

        // Log coloré pour le sommeil
        println!(
            "{} {}: {}",
            "[SOMMEIL]".bright_yellow().bold(),
            self.name.bright_green(),
            summary.bright_white()
        );
    }
}

// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;
    use ollama_rs::Ollama;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new(1, "Alice", "optimistic");
        assert_eq!(agent.name, "Alice");
        assert_eq!(agent.personality, "optimistic");
        assert_eq!(agent.memory.len(), 0);
    }

    #[test]
    fn test_listen() {
        let mut agent = Agent::new(1, "Alice", "optimistic");
        let messages = vec![Message {
            sender: "Bob".to_string(),
            recipient: "Alice".to_string(),
            content: "Bonjour".to_string(),
            timestamp: 1,
        }];

        agent.listen(&messages);
        assert_eq!(agent.memory.len(), 1);
        assert_eq!(agent.conversation.len(), 1);
    }

    #[tokio::test]
    async fn test_response_generation() {
        let ollama = Ollama::default();
        let mut agent = Agent::new(1, "Alice", "optimistic");

        // Test avec un mock serait préférable ici
        let result = agent.generate_response(&ollama).await;
        assert!(result.is_err()); // Échec attendu car pas de vrai serveur Ollama
    }
}
