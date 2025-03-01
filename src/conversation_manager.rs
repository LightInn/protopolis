// conversation_manager.rs

use std::collections::HashMap;
use crate::message::Message;

/// Manages conversations between agents by storing message history and active conversations.
pub struct ConversationManager {
    /// Stores the conversation history between pairs of agents.
    conversations: HashMap<(String, String), Vec<Message>>,

    /// Tracks active conversations by storing ongoing communication partners.
    active_conversations: HashMap<String, Vec<String>>,
}

impl ConversationManager {
    /// Creates a new, empty conversation manager.
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
            active_conversations: HashMap::new(),
        }
    }

    /// Adds a message to the conversation history and updates active conversations.
    ///
    /// # Arguments
    /// * `message` - The message to be stored.
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

        // Update active conversations
        self.active_conversations
            .entry(message.sender.clone())
            .or_insert_with(Vec::new)
            .push(message.recipient.clone());

        self.active_conversations
            .entry(message.recipient.clone())
            .or_insert_with(Vec::new)
            .push(message.sender.clone());
    }

    /// Retrieves the message history between two agents.
    ///
    /// # Arguments
    /// * `agent1` - The first agent in the conversation.
    /// * `agent2` - The second agent in the conversation.
    ///
    /// # Returns
    /// * A vector of `Message` representing the conversation history.
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

    /// Checks if an agent is currently engaged in a conversation.
    ///
    /// # Arguments
    /// * `agent` - The agent to check.
    ///
    /// # Returns
    /// * `true` if the agent has active conversations, otherwise `false`.
    pub fn is_in_conversation(&self, agent: &str) -> bool {
        self.active_conversations.contains_key(agent) &&
            !self.active_conversations[agent].is_empty()
    }

    /// Retrieves the list of active conversation partners for a given agent.
    ///
    /// # Arguments
    /// * `agent` - The agent whose conversation partners are requested.
    ///
    /// # Returns
    /// * A vector of `String` representing the names of active conversation partners.
    pub fn get_conversation_partners(&self, agent: &str) -> Vec<String> {
        self.active_conversations
            .get(agent)
            .cloned()
            .unwrap_or_default()
    }
}
