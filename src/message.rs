// message.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub sender: String,
    pub recipient: String,
    pub content: Value,
    pub timestamp: DateTime<Utc>,
}
