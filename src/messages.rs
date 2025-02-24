// messages.rs
use crate::agent::Agent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub recipient: String, // chaîne vide pour un broadcast à tous
    pub content: String,
    pub timestamp: u32,
}

#[derive(Debug)]
pub struct MessageBus {
    subscribers: Mutex<HashMap<String, Arc<Agent>>>,
}

impl MessageBus {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            subscribers: Mutex::new(HashMap::new()),
        })
    }

    /// Enregistre un agent dans le bus
    pub fn register_agent(&self, agent: &Arc<Agent>) {
        self.subscribers
            .lock()
            .unwrap()
            .insert(agent.name.clone(), agent.clone());
    }

    /// Diffuse un message à tous les agents, ou seulement à ceux spécifiés si recipient n'est pas vide
    pub fn broadcast_message(&self, message: Message, radius: i32, sender: &Agent) {
        let sender_position = &sender.position;
        // On verrouille une seule fois la map des abonnés
        let subscribers = self.subscribers.lock().unwrap();
        for agent in subscribers.values() {
            // Évite de se renvoyer un message à soi-même
            if agent.name == sender.name {
                continue;
            }
            // Tente de verrouiller la file de messages de l'agent
            if let Ok(mut queue) = agent.message_queue.try_lock() {
                // Si recipient est vide, on diffuse à tous ceux dans le rayon
                if message.recipient.is_empty() {
                    if agent.position.distance_square(sender_position) <= radius {
                        queue.push_back(message.clone());
                    }
                }
                // Sinon, on envoie uniquement à l'agent ciblé
                else if agent.name == message.recipient {
                    queue.push_back(message.clone());
                }
            } else {
                eprintln!(
                    "Impossible d'acquérir le verrou pour la file de messages de {}",
                    agent.name
                );
            }
        }
    }

    /// Diffuse un message système à tous les agents (sans rayon)
    pub fn broadcast_system_message(&self, message: Message) {
        for agent in self.subscribers.lock().unwrap().values() {
            agent
                .message_queue
                .lock()
                .unwrap()
                .push_back(message.clone());
        }
    }
}
