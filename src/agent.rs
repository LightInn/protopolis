// agent.rs

use crate::message::Message;
use crate::personality::Personality;
use crate::state::AgentState;
use chrono::Utc;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use serde_json::json;
use tokio::runtime::Runtime;

/// Represents an autonomous agent in the simulation.
#[derive(Debug, Clone)]
pub struct Agent {
    /// Unique identifier for the agent.
    pub id: String,

    /// Agent's display name.
    pub name: String,

    /// Current state of the agent (Idle, Thinking, Speaking, etc.).
    pub state: AgentState,

    /// Current energy level of the agent.
    pub energy: f32,

    /// Position of the agent in the world (x, y coordinates).
    pub position: (i32, i32),

    /// Agent's personality traits influencing its behavior.
    pub personality: Personality,

    /// Memory storage for important past events.
    pub memory: Vec<String>,

    /// Conversation history (last 10 messages).
    pub conversation_history: Vec<String>,

    /// Name of the AI model used for generating responses.
    pub ollama_model: String,

    /// Stores messages heard during the current tick.
    pub next_prompt: String,
}

impl Agent {
    /// Creates a new agent with the given parameters.
    ///
    /// # Arguments
    /// * `id` - Unique agent ID.
    /// * `name` - Agent's name.
    /// * `personality` - Personality traits of the agent.
    /// * `initial_energy` - Starting energy level.
    /// * `initial_position` - Initial (x, y) coordinates.
    ///
    /// # Returns
    /// * A new `Agent` instance.
    pub fn new(
        id: String,
        name: String,
        personality: Personality,
        initial_energy: f32,
        initial_position: (i32, i32),
    ) -> Self {
        Self {
            id,
            name,
            state: AgentState::Idle,
            energy: initial_energy,
            position: initial_position,
            personality,
            memory: Vec::new(),
            conversation_history: Vec::new(),
            ollama_model: "llama3.2:latest".to_string(), // Default model
            next_prompt: String::new(),
        }
    }

    /// Sets the AI model used for generating responses.
    pub fn set_model(&mut self, model: String) {
        self.ollama_model = model;
    }

    /// Processes an incoming message and generates a response.
    ///
    /// # Arguments
    /// * `message` - The incoming message.
    /// * `runtime` - The Tokio runtime for async execution.
    ///
    /// # Returns
    /// * An optional `Message` containing the agent's response.
    ///
    /// # Behavior:
    /// - Updates the agent's state to `Thinking`.
    /// - Adds the message to the conversation history.
    /// - Uses an AI model (Ollama) to generate a response.
    /// - Consumes energy when speaking.
    pub fn process_message(&mut self, message: &Message, runtime: &Runtime) -> Option<Message> {
        self.state = AgentState::Thinking;

        // Append the message to conversation history
        let msg_entry = format!("{}: {}", message.sender, message.content);
        self.conversation_history.push(msg_entry);

        // Limit conversation history to 10 messages to keep prompts short
        if self.conversation_history.len() > 10 {
            self.conversation_history.remove(0);
        }

        // Generate a response using Ollama
        let response =
            runtime.block_on(async { self.generate_response(&message.content.to_string()).await });

        if let Ok(response_text) = response {
            // Add response to conversation history
            self.conversation_history
                .push(format!("{}: {}", self.name, response_text));

            // Create a new message
            let response_message = Message {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                sender: self.name.clone(),
                recipient: message.sender.clone(),
                content: json!(response_text),
            };

            // Update agent state and reduce energy
            self.state = AgentState::Speaking;
            self.energy -= 1.0;

            Some(response_message)
        } else {
            // If response generation fails, return to Idle state
            self.state = AgentState::Idle;
            None
        }
    }

    /// Generates a response based on the given input message.
    ///
    /// # Arguments
    /// * `input` - The message to respond to.
    ///
    /// # Returns
    /// * `Ok(String)` containing the response text.
    /// * `Err(String)` if the response could not be generated.
    ///
    /// # TODO:
    /// - Improve the response generation logic by integrating more personality-driven choices.
    /// - Consider using a weighted system where personality traits influence responses.
    async fn generate_response(&self, input: &str) -> Result<String, String> {
        let ollama = Ollama::default();

        // Construct personality description
        let personality_desc = format!(
            "You are {}, an AI agent with the following personality traits:\n\
            - Openness: {}/10\n\
            - Conscientiousness: {}/10\n\
            - Extraversion: {}/10\n\
            - Agreeableness: {}/10\n\
            - Neuroticism: {}/10\n\
            Respond concisely (max 2-3 sentences) while staying in character.",
            self.name,
            (self.personality.openness * 10.0) as i32,
            (self.personality.conscientiousness * 10.0) as i32,
            (self.personality.extraversion * 10.0) as i32,
            (self.personality.agreeableness * 10.0) as i32,
            (self.personality.neuroticism * 10.0) as i32
        );

        // Build conversation history
        let history = self.conversation_history.join("\n");

        // Final prompt
        let prompt = format!(
            "{}\n\nConversation history:\n{}\n\nRespond to: {}",
            personality_desc, history, input
        );

        // Send request to the AI model
        let request = GenerationRequest::new(self.ollama_model.clone(), prompt);
        match ollama.generate(request).await {
            Ok(response) => Ok(response.response),
            Err(e) => Err(format!("Generation error: {}", e)),
        }
    }

    /// Generates a response based on the agent's stored prompt.
    ///
    /// # Returns
    /// * `Ok(String)` containing the response text.
    /// * `Err(String)` if the response could not be generated.
    ///
    /// # TODO:
    /// - Improve contextual awareness by prioritizing recent inputs.
    /// - Introduce energy-based behavior (e.g., tired agents respond differently).
    pub(crate) async fn generate_response_from_prompt(&self) -> Result<String, String> {
        let ollama = Ollama::default();

        // Construct personality description
        let personality_desc = format!(
            "You are {}, an AI agent with the following personality traits:\n\
            - Openness: {}/10\n\
            - Conscientiousness: {}/10\n\
            - Extraversion: {}/10\n\
            - Agreeableness: {}/10\n\
            - Neuroticism: {}/10\n\
            Respond concisely (max 2-3 sentences) while staying in character.",
            self.name,
            (self.personality.openness * 10.0) as i32,
            (self.personality.conscientiousness * 10.0) as i32,
            (self.personality.extraversion * 10.0) as i32,
            (self.personality.agreeableness * 10.0) as i32,
            (self.personality.neuroticism * 10.0) as i32
        );

        // Conversation history
        let history = self.conversation_history.join("\n");

        // Final prompt including recent messages
        let prompt = format!(
            "{}\n\nConversation history:\n{}\n\nRecent messages:\n{}\n\nHow would you respond?",
            personality_desc, history, self.next_prompt
        );

        // Send request to the AI model
        let request = GenerationRequest::new(self.ollama_model.clone(), prompt);
        match ollama.generate(request).await {
            Ok(response) => Ok(response.response),
            Err(e) => Err(format!("Generation error: {}", e)),
        }
    }
}
