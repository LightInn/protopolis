// conversation_manager.rs

use crate::message::Message;
use std::collections::HashMap;

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
}
