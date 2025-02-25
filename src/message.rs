// messages.rs
use crate::agent::Agent;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct Message {
    pub sender: String,
    pub recipient: String, // chaîne vide pour un broadcast à tous
    pub content: Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct MessageBus {
    subscribers: RwLock<HashMap<String, Arc<RwLock<Agent>>>>,
}

impl MessageBus {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            subscribers: RwLock::new(HashMap::new()),
        })
    }

    /// Enregistre un agent dans le bus
    pub fn register_agent(&self, agent: Arc<RwLock<Agent>>) {
        self.subscribers
            .write()
            .unwrap()
            .insert(agent.read().unwrap().name.clone(), agent.clone());
    }

    /// Diffuse un message à tous les agents, ou seulement à ceux spécifiés si recipient n'est pas vide
    pub fn broadcast_message(&self, message: Message, radius: i32) {
        let sender_position = (0, 0);
        // On verrouille une seule fois la map des abonnés
        let subscribers = self.subscribers.read().unwrap();
        for agent in subscribers.values() {
            // Évite de se renvoyer un message à soi-même
            if agent.try_read().is_err() {
                continue;
            }

            let mut agent = agent.write().expect("Impossible d'obtenir le verrou en écriture sur l'agent");
            // Tente de verrouiller la file de messages de l'agent

            // Si recipient est vide, on diffuse à tous ceux dans le rayon
            if message.recipient.is_empty() {
                if agent.distance_square(sender_position) <= radius {
                    agent.message_queue.push_back(message.clone());
                }
            }
            // Sinon, on envoie uniquement à l'agent ciblé
            else if agent.name == message.recipient {
                agent.message_queue.push_back(message.clone());
            }

            // eprintln!(
            //     "Impossible d'acquérir le verrou pour la file de messages de {}",
            //     agent.name
            // );
        }
        println!("Message broadcasted");
    }

    /// Diffuse un message système à tous les agents (sans rayon)
    pub fn broadcast_system_message(&self, message: Message) {
        for agent in self.subscribers.read().unwrap().values() {
            let mut agent = agent.write().unwrap();
            agent.message_queue.push_back(message.clone());
        }
    }
}
