// agent.rs
use crate::messages::{Message, MessageBus};
use colored::Colorize;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Agent {
    pub id: u32,
    pub name: String,
    pub personality: String,
    pub memory: Mutex<Vec<String>>,
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
            memory: Mutex::new(Vec::new()),
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
            self.memory
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

    //
    // /// Traite les messages reçus et génère une réponse
    // pub async fn process_messages(&mut self, ollama: &Ollama) {
    //     // Écoute des messages incoming
    //     self.listen();
    //
    //     // Réflexion sur les messages reçus
    //     // todo : ne fait rien du tout pour l'instant
    //     self.reflect();
    //
    //     // Génération d'une réponse
    //     let message = self.generate_response(ollama).await.ok();
    //     // Appel de la callback si présente
    // }
    //
    // /// Écoute les messages et les ajoute à la mémoire
    // fn listen(&mut self) {
    //     // pop front from the queue
    //     for message in self.message_queue.drain(..) {
    //         self.memory.push(format!(
    //             "[Cycle {}] Message de {}: {}",
    //             message.timestamp, message.sender, message.content
    //         ));
    //         // self.conversation.push_back(message.clone());
    //
    //         // Log coloré pour l'écoute
    //         println!(
    //             "{} {} {}: {}",
    //             "[ÉCOUTE]".bright_blue().bold(),
    //             self.name.bright_green(),
    //             "a reçu un message de".bright_white(),
    //             message.sender.bright_yellow()
    //         );
    //         println!("   Contenu: {}", message.content.bright_white());
    //     }
    // }
    //
    // /// Simule la réflexion de l'agent
    // fn reflect(&mut self) {
    //     let reflection = format!("Réflexion sur {} messages récents", self.conversation.len());
    //     self.memory.push(reflection.clone());
    //     self.last_action = "Reflecting".to_string();
    //
    //     // Log coloré pour la réflexion
    //     println!(
    //         "{} {}: {}",
    //         "[RÉFLEXION]".bright_magenta().bold(),
    //         self.name.bright_green(),
    //         reflection.bright_white()
    //     );
    // }
    //
    // /// Génère une réponse en utilisant Ollama
    // async fn generate_response(
    //     &mut self,
    //     ollama: &Ollama,
    // ) -> Result<Message, Box<dyn std::error::Error>> {
    //     let prompt = self.build_prompt();
    //     let mut retries = 3;
    //
    //     // Log coloré pour la génération de réponse
    //     println!(
    //         "{} {}: Génération d'une réponse...",
    //         "[GÉNÉRATION]".bright_cyan().bold(),
    //         self.name.bright_green()
    //     );
    //
    //     loop {
    //         let response = ollama
    //             .generate(GenerationRequest::new(
    //                 "llama3.2:latest".to_string(),
    //                 prompt.clone(),
    //             ))
    //             .await?;
    //
    //         if let Ok(parsed) = self.parse_response(&response.response) {
    //             self.memory.push(format!("Réponse générée: {}", parsed));
    //             self.last_action = "Speaking".to_string();
    //
    //             // Log coloré pour la réponse générée
    //             println!(
    //                 "{} {}: {}",
    //                 "[RÉPONSE]".bright_green().bold(),
    //                 self.name.bright_green(),
    //                 parsed.bright_white()
    //             );
    //
    //             let message = Message {
    //                 sender: self.name.clone(),
    //                 recipient: "".to_string(),
    //                 content: parsed,
    //                 timestamp: 0,
    //             };
    //
    //             return Ok(message);
    //         }
    //
    //         retries -= 1;
    //         if retries == 0 {
    //             println!(
    //                 "{} {}: Échec de la génération de réponse",
    //                 "[ERREUR]".bright_red().bold(),
    //                 self.name.bright_green()
    //             );
    //             return Err("Échec génération JSON valide".into());
    //         }
    //     }
    // }
    //
    // // pub async fn test_message(&mut self, ollama: &mut Ollama, model: String, prompt: String) {
    // //
    // //
    // //
    // //     let res = ollama
    // //         .send_chat_messages_with_history(
    // //             &mut self.conv_channel, // <- messages will be saved here
    // //             ChatMessageRequest::new(
    // //                 model,
    // //                 vec![ChatMessage::user(prompt)], // <- You should provide only one message
    // //             ),
    // //         )
    // //         .await;
    // //
    // //     if let Ok(res) = res {
    // //         println!("{}", res.message.content);
    // //     }
    // //
    // // }
    //
    // /// Construit le prompt pour l'agent
    // fn build_prompt(&self) -> String {
    //     format!(
    //         "You are {name} ({personality}).\n\
    //         Your goal: {goal}\n\
    //         Historique conversation:\n{history}\n\
    //         Formate ta réponse en JSON (uniquement le JSON en sortie brut, rien d'autre, il sera parsé) avec le structure suivante:\n\
    //         {{\n  \"response\": \"Ta réponse ici\"\n}}\n\
    //         try to keep your message short until absolutely necessity",
    //         name = self.name,
    //         personality = self.personality,
    //         goal = self.current_goal,
    //         history = self.conversation_history()
    //     )
    // }
    //
    // /// Parse la réponse de l'IA
    // fn parse_response(&self, response: &str) -> Result<String, Box<dyn std::error::Error>> {
    //     // println!("Réponse brute: {}", response);
    //     let parsed: Value = serde_json::from_str(response)?;
    //     Ok(parsed["response"]
    //         .as_str()
    //         .ok_or("Champ 'response' manquant")?
    //         .to_string())
    // }
}
