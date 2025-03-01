// message.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type alias for message content, allowing flexible JSON structures.
pub type MessageContent = Value;

/// Represents a message exchanged between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for the message.
    pub id: String,

    /// Timestamp indicating when the message was sent (in UTC).
    pub timestamp: DateTime<Utc>,

    /// Identifier of the sender (could be an agent name or system).
    pub sender: String,

    /// Identifier of the recipient (could be an agent name or broadcast).
    pub recipient: String,

    /// The actual message content, stored as a flexible JSON value.
    pub content: MessageContent,
}
