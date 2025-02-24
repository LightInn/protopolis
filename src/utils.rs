// utils.rs
use crate::agent::Agent;
use serde_json::Value;
use std::io;

pub fn validate_json(response: &str) -> bool {
    serde_json::from_str::<Value>(response).is_ok()
}

pub fn get_user_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn save_conversations(agents: &[Agent]) {
    // Implémentation de sauvegarde...
}
