// agent.rs
use crate::messages::{Message, MessageBus};
use colored::Colorize;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::Ollama;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Agent {
    pub id: u32,
    pub name: String,
    pub personality: String,
    pub memory: Vec<ChatMessage>,
    pub history: Mutex<Vec<String>>,
    pub current_goal: Mutex<String>,
    pub conversation: Mutex<Vec<String>>,
    pub last_action: Mutex<String>,
    pub position: Coord,
    pub message_queue: Mutex<VecDeque<Message>>,
    pub msg_bus: Arc<MessageBus>, // Nouveau champ
}

#[derive(Debug)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn distance_square(&self, other: &Coord) -> i32 {
        (self.x - other.x).pow(2) + (self.y - other.y).pow(2)
    }
}

impl Agent {
    pub fn new(id: u32, name: &str, personality: &str, msg_bus: Arc<MessageBus>) -> Arc<Self> {
        Arc::new(Self {
            id,
            name: name.to_string(),
            personality: personality.to_string(),
            memory: vec![],
            history: Mutex::new(Vec::new()),
            current_goal: Mutex::new("Engager une conversation intéressante".to_string()),
            conversation: Mutex::new(Vec::new()),
            last_action: Mutex::new("Initialisé".to_string()),
            position: Coord { x: 0, y: 0 },
            message_queue: Mutex::new(VecDeque::new()),
            msg_bus,
        })
    }

    /// Traite les messages reçus (en simplifiant)
    pub fn process_messages(&self) {
        let mut queue = self.message_queue.lock().unwrap();
        while let Some(message) = queue.pop_front() {
            println!("{} reçoit : {}", self.name, message.content);
            // Ajoute à la mémoire ou à la conversation
            self.history
                .lock()
                .unwrap()
                .push(format!("[{}] {}", message.timestamp, message.content));
        }

        let response = Message {
            sender: self.name.clone(),
            recipient: "".to_string(),
            content: "My answers".to_string(),
            timestamp: 0,
        };

        self.msg_bus.broadcast_message(response, 100, self);
    }

    /// Génère une réponse en utilisant Ollama
    async fn generate_response(
        &mut self,
        ollama: &mut Ollama,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let prompt = "test prompt";
        let mut retries = 3;

        // Log coloré pour la génération de réponse
        println!(
            "{} {}: Génération d'une réponse...",
            "[GÉNÉRATION]".bright_cyan().bold(),
            self.name.bright_green()
        );

        loop {
            let response = ollama
                .send_chat_messages_with_history(
                    &mut self.memory,
                    ChatMessageRequest::new(
                        "llama3.2:latest".to_string(),
                        vec![ChatMessage::user(prompt.parse().unwrap())], // <- You should provide only one message
                    ),
                )
                .await;

            if let Ok(response) = response {
                let parsed = response.message.content;
                // self.history(format!("Réponse générée: {}", parsed));
                self.last_action = Mutex::from("Speaking".to_string());

                // Log coloré pour la réponse générée
                println!(
                    "{} {}: {}",
                    "[RÉPONSE]".bright_green().bold(),
                    self.name.bright_green(),
                    parsed.bright_white()
                );

                let message = Message {
                    sender: self.name.clone(),
                    recipient: "".to_string(),
                    content: parsed,
                    timestamp: 0,
                };

                return Ok(message);
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
}
