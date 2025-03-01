// conversation_manager.rs
use std::collections::HashMap;
use crate::message::Message;

pub struct ConversationManager {
    // Stocke l'historique des conversations par paire d'agents
    conversations: HashMap<(String, String), Vec<Message>>,
    // Stocke les conversations actives
    active_conversations: HashMap<String, Vec<String>>,
}

impl ConversationManager {
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
            active_conversations: HashMap::new(),
        }
    }

    // Ajoute un message à l'historique de conversation
    pub fn add_message(&mut self, message: Message) {
        let conversation_key = if message.sender < message.recipient {
            (message.sender.clone(), message.recipient.clone())
        } else {
            (message.recipient.clone(), message.sender.clone())
        };

        self.conversations
            .entry(conversation_key)
            .or_insert_with(Vec::new)
            .push(message.clone());

        // Mettre à jour les conversations actives
        self.active_conversations
            .entry(message.sender.clone())
            .or_insert_with(Vec::new)
            .push(message.recipient.clone());

        self.active_conversations
            .entry(message.recipient.clone())
            .or_insert_with(Vec::new)
            .push(message.sender.clone());
    }

    // Récupère l'historique de conversation entre deux agents
    pub fn get_conversation(&self, agent1: &str, agent2: &str) -> Vec<Message> {
        let key = if agent1 < agent2 {
            (agent1.to_string(), agent2.to_string())
        } else {
            (agent2.to_string(), agent1.to_string())
        };

        self.conversations
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }

    // Vérifie si un agent est en conversation active
    pub fn is_in_conversation(&self, agent: &str) -> bool {
        self.active_conversations.contains_key(agent) &&
            !self.active_conversations[agent].is_empty()
    }

    // Récupère les partenaires de conversation d'un agent
    pub fn get_conversation_partners(&self, agent: &str) -> Vec<String> {
        self.active_conversations
            .get(agent)
            .cloned()
            .unwrap_or_default()
    }
}
