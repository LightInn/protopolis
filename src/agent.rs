// agent.rs
use crate::message::{Message, MessageContent};
use crate::personality::Personality;
use crate::state::AgentState;
use chrono::Utc;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::{request::GenerationRequest, GenerationContext};
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub state: AgentState,
    pub energy: f32,
    pub position: (i32, i32),
    pub personality: Personality,
    pub memory: Vec<String>,
    pub conversation_history: Vec<String>,
    pub ollama_model: String,
}

impl Agent {
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
            energy: 0.0,
            position: initial_position,
            personality,
            memory: Vec::new(),
            conversation_history: Vec::new(),
            ollama_model: "llama2".to_string(), // Modèle par défaut
        }
    }

    pub fn set_model(&mut self, model: String) {
        self.ollama_model = model;
    }

    pub fn process_message(&mut self, message: &Message, runtime: &Runtime) -> Option<Message> {
        // Mettre à jour l'état
        self.state = AgentState::Thinking;

        // Ajouter le message à l'historique de conversation
        let msg_entry = format!("{}: {}", message.sender, message.content);
        self.conversation_history.push(msg_entry);

        // Limiter l'historique à 10 messages pour éviter des prompts trop longs
        if self.conversation_history.len() > 10 {
            self.conversation_history.remove(0);
        }

        // Générer une réponse avec Ollama
        let response = runtime.block_on(async {
            self.generate_response(&message.content.to_string()).await
        });

        if let Ok(response_text) = response {
            // Ajouter notre réponse à l'historique
            self.conversation_history.push(format!("{}: {}", self.name, response_text));

            // Créer un nouveau message
            let response_message = Message {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                sender: self.name.clone(),
                recipient: message.sender.clone(),
                content: json!(response_text),
            };

            // Mettre à jour l'état
            self.state = AgentState::Speaking;
            self.energy -= 1.0; // Parler consomme de l'énergie

            Some(response_message)
        } else {
            // En cas d'erreur, on passe à l'état Idle
            self.state = AgentState::Idle;
            None
        }
    }

    async fn generate_response(&self, input: &str) -> Result<String, String> {
        let ollama = Ollama::default();

        // Construire le prompt avec la personnalité et l'historique de conversation
        let personality_desc = format!(
            "Tu es {}, un agent avec les traits de personnalité suivants:\n\
            - Ouverture d'esprit: {}/10\n\
            - Conscienciosité: {}/10\n\
            - Extraversion: {}/10\n\
            - Agréabilité: {}/10\n\
            - Névrosisme: {}/10\n\
            Réponds de manière concise (max 2-3 phrases) en respectant ta personnalité.",
            self.name,
            (self.personality.openness * 10.0) as i32,
            (self.personality.conscientiousness * 10.0) as i32,
            (self.personality.extraversion * 10.0) as i32,
            (self.personality.agreeableness * 10.0) as i32,
            (self.personality.neuroticism * 10.0) as i32
        );

        // Construire l'historique de conversation
        let history = self.conversation_history.join("\n");

        // Prompt final
        let prompt = format!(
            "{}\n\nHistorique de conversation:\n{}\n\nRéponds à: {}",
            personality_desc,
            history,
            input
        );

        // Créer la requête
        let request = GenerationRequest::new(self.ollama_model.clone(), prompt);

        // Envoyer la requête
        match ollama.generate(request).await {
            Ok(response) => {
                println!("{}", response.response);
                Ok(response.response)   
            },
            Err(e) => Err(format!("Erreur lors de la génération: {}", e))
        }
    }
}
