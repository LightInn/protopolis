// agent.rs
use crate::action::{Action, ActionHandler, ActionResult};
use crate::config::AgentConfig;
use crate::message::{Message, MessageBus};
use crate::personality::Personality;
use crate::prompt;
use crate::prompt::Prompt;
use crate::state::AgentState;
use chrono::Utc;
use colored::Colorize;
use ollama_rs::generation::chat::request::ChatMessageRequest;
use ollama_rs::generation::chat::ChatMessage;
use ollama_rs::Ollama;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Represents an autonomous agent in the system
#[derive(Debug)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub personality: Personality,
    pub state: AgentState,
    pub action_handler: ActionHandler,
    pub message_bus: Arc<MessageBus>,
    pub conversation_history: Vec<ChatMessage>,
    pub memory: Vec<Message>,
    pub system_prompt: String,
    pub next_prompt: String,
    pub message_queue: VecDeque<Message>,
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
    /// Creates a new agent from configuration
    pub fn new(config: &AgentConfig, message_bus: Arc<MessageBus>) -> Arc<RwLock<Agent>> {
        Arc::new(RwLock::new(Self {
            id: Uuid::new_v4(),
            name: config.name.clone(),
            personality: crate::personality::get_personality_template(&config.personality_template),
            state: AgentState::Idle,
            action_handler: ActionHandler::new(),
            message_bus,
            conversation_history: Vec::new(),
            memory: vec![],
            system_prompt: config.system_prompt.clone(),
            next_prompt: "".to_string(),
            message_queue: Default::default(),
        }))
    }

    /// Processes incoming messages for the agent
    pub fn process_messages(&mut self) {
        // Récupérer et vider la file de messages sous forme de Vec
        let messages = { self.message_queue.drain(..).collect::<Vec<_>>() };

        // Récupérer les informations nécessaires AVANT l'appel à broadcast_message
        let agent_name = self.name.clone();
        let message_bus = self.message_bus.clone(); // Assurez-vous que c'est un Arc<MessageBus>

        // Maintenant, on peut traiter les messages sans verrou
        for message in messages {
            println!(
                "{} reçoit de {} pour {} : {}",
                agent_name, message.sender, message.recipient, message.content
            );
            self.memory.push(message.clone());
            self.handle_message(message);
        }

        let response = Message {
            sender: agent_name,
            recipient: "".to_string(),
            content: Value::String("My answers".to_string()),
            timestamp: Utc::now(),
        };

        // Appel en dehors du contexte d'emprunt mutable de self
        message_bus.broadcast_message(response, 100);
    }

    /// Génère une réponse en utilisant Ollama
    async fn generate_response(
        &mut self,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let mut retries = 3;
        let mut ollama = Ollama::default();

        // Log coloré pour la génération de réponse
        println!(
            "{} {}: Génération d'une réponse...",
            "[GÉNÉRATION]".bright_cyan().bold(),
            self.name.bright_green()
        );

        loop {
            let response = ollama
                .send_chat_messages_with_history(
                    &mut self.conversation_history,
                    ChatMessageRequest::new(
                        "llama3.2:latest".to_string(),
                        vec![ChatMessage::user(self.next_prompt.clone())], // <- You should provide only one message
                    ),
                )
                .await;

            if let Ok(response) = response {
                let parsed = response.message.content;
                // self.history(format!("Réponse générée: {}", parsed));
                // self.last_action = Mutex::from("Speaking".to_string());

                // Log coloré pour la réponse générée
                println!(
                    "{} {}: {}",
                    "[RÉPONSE]".bright_green().bold(),
                    self.name.bright_green(),
                    parsed.bright_white()
                );

                let message = Message {
                    sender: self.name.clone(),
                    recipient: "everyone".to_string(),
                    content: Value::String(parsed),
                    timestamp: Utc::now(),
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

    /// Updates the agent's state and performs actions
    pub async fn update(&mut self, current_tick: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Consume energy over time
        self.consume_base_energy();

        // Process incoming messages
        self.process_messages();

        // Decide and perform next action
        self.decide_next_action(current_tick).await?;

        Ok(())
    }

    /// Handles a single message
    fn handle_message(&mut self, message: Message) {
        self.next_prompt += &Prompt::get_message(message);
    }

    /// Decides and performs the next action based on current state and personality
    async fn decide_next_action(
        &mut self,
        current_tick: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get available actions based on current state and energy
        let available_actions = self.get_available_actions();

        // Use personality to influence decision
        let action_strings: Vec<String> = available_actions
            .iter()
            .map(|a| format!("{:?}", a))
            .collect();

        let chosen_index = self.personality.influence_decision(&action_strings);
        let chosen_action = &available_actions[chosen_index];

        // Perform the chosen action
        if self.action_handler.can_perform(chosen_action, 100.0) {
            let result = self.action_handler.execute(chosen_action)?;
            self.apply_action_result(result);
        }


        // Generate a response using Ollama
       let generated = self.generate_response().await?;

        // Send the generated response
        self.send_message(generated);

        Ok(())
    }

    /// Gets list of available actions based on current state
    fn get_available_actions(&self) -> Vec<Action> {
        let mut actions = vec![
            Action::Think {
                topic: "next move".to_string(),
            },
            Action::CheckTime,
        ];

        let energy = 100.0;

        // Add conditional actions based on state and energy
        if energy < 30.0 {
            actions.push(Action::Sleep { duration: 10 });
        }

        // if self.state != AgentState::Sleeping {
        //     actions.push(Action::Move {
        //         x: self.positon.0 + 1,
        //         y: self.position.1,
        //     });
        //     actions.push(Action::Listen { duration: 5 });
        //     actions.push(Action::Speak {
        //         message: "Hello!".to_string(),
        //         target: None,
        //     });
        // }

        actions
    }

    /// Applies the result of an action
    fn apply_action_result(&mut self, result: ActionResult) {
        let mut energy = 100.0;
        self.state = result.new_state;
        energy += result.energy_delta;

        if let Some(message) = result.message {
            println!("{}: {}", self.name, message);
        }
    }

    /// Consumes base energy over time
    fn consume_base_energy(&self) {
        let mut energy = 100.0;

        energy -= 0.1; // Base energy consumption rate
    }

    /// Saves the agent's conversation history
    pub fn save_conversation_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("conversations/{}-{}.json", self.name, self.id);
        let content = serde_json::to_string_pretty(&self.conversation_history)?;
        std::fs::write(filename, content)?;
        Ok(())
    }

    /// Performs memory synthesis during sleep
    fn synthesize_memories(&mut self) {
        if self.state == AgentState::Sleeping {
            // Analyze conversation history and create memory summaries
            // This could involve NLP or other analysis techniques
            println!("{} is synthesizing memories while sleeping", self.name);
        }
    }

    /// Gets the agent's current position
    pub fn get_position(&self) -> (i32, i32) {
        // self.position
        (0, 0)
    }

    /// Gets the agent's current energy level
    pub fn get_energy(&self) -> f32 {
        // self.energy
        100.0
    }

    /// Gets the agent's current state
    pub fn get_state(&self) -> AgentState {
        self.state.clone()
    }

    /// Sends a message to another agent
    pub fn send_message(&self, message: Message) {
        self.message_bus.broadcast_message(message, 100);
    }

    pub fn distance_square(&self, sender_position: (i32, i32)) -> i32 {
        return 0;
    }
}
