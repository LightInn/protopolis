// agent.rs

use crate::personality::Personality;
use crate::state::AgentState;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;

/// Represents an autonomous agent in the simulation.
#[derive(Debug, Clone)]
pub struct Agent {
    /// Agent's display name.
    pub name: String,

    /// Current state of the agent (Idle, Thinking, Speaking, etc.).
    pub state: AgentState,

    /// Current energy level of the agent.
    pub energy: f32,

    /// Agent's personality traits influencing its behavior.
    pub personality: Personality,

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
    /// * `ollama_model` - The Ollama model to be used by the agent.
    ///
    /// # Returns
    /// * A new `Agent` instance.
    pub fn new(name: String, personality: Personality, initial_energy: f32, ollama_model: String) -> Self {
        Self {
            name,
            state: AgentState::Idle,
            energy: initial_energy,
            personality,
            conversation_history: Vec::new(),
            ollama_model, // Use the provided model
            next_prompt: String::new(),
        }
    }

    /// Sets the AI model used for generating responses.
    pub fn set_model(&mut self, model: String) {
        self.ollama_model = model;
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
