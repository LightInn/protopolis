// message.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type MessageContent = Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub sender: String,
    pub recipient: String,
    pub content: MessageContent,
}
