// messages.rs
use crate::agent::Agent;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub sender: String,
    pub recipient: String, // chaîne vide pour un broadcast à tous
    pub content: Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct MessageBus {
    subscribers: RwLock<HashMap<String, Arc<RwLock<Agent>>>>,
    recent_messages: RwLock<VecDeque<Message>>, // Nouveau
    debug: bool,
}

impl MessageBus {
    pub fn new(debug: bool) -> Arc<Self> {
        Arc::new(Self {
            subscribers: RwLock::new(HashMap::new()),
            recent_messages: RwLock::new(VecDeque::with_capacity(100)), // Stocke les 100 derniers messages
            debug,
        })
    }

    /// Enregistre un agent dans le bus
    pub async fn register_agent(&self, agent: Arc<RwLock<Agent>>) {
        self.subscribers
            .write()
            .await
            .insert(agent.read().await.name.clone(), agent.clone());
    }

    /// Diffuse un message à tous les agents, ou seulement à ceux spécifiés si recipient n'est pas vide
    // Modifier broadcast_message pour capturer les messages
    pub async fn broadcast_message(&self, message: Message, radius: i32) {
        // Ajouter le message à l'historique
        self.recent_messages.write().await.push_back(message.clone());
        if self.recent_messages.read().await.len() > 100 {
            self.recent_messages.write().await.pop_front();
        }
        let sender_position = (0, 0);
        // On verrouille une seule fois la map des abonnés
        let subscribers = self.subscribers.read().await;
        for agent in subscribers.values() {
            // Évite de se renvoyer un message à soi-même
            if agent.try_read().is_err() {
                continue;
            }

            let mut agent = agent
                .write()
                .await;
            // Tente de verrouiller la file de messages de l'agent

            // Si recipient est vide, on diffuse à tous ceux dans le rayon

            if agent.distance_square(sender_position) <= radius {
                agent.message_queue.push_back(message.clone());
            }

            // eprintln!(
            //     "Impossible d'acquérir le verrou pour la file de messages de {}",
            //     agent.name
            // );
        }
        if self.debug {
            println!("Message broadcasted");
        }
    }

    /// Diffuse un message système à tous les agents (sans rayon)
    pub async fn broadcast_system_message(&self, message: Message) {
        for agent in self.subscribers.read().await.values() {
            let mut agent = agent.write().await;
            agent.message_queue.push_back(message.clone());
        }
    }


    // Ajouter cette méthode
    pub async fn get_recent_messages(&self) -> Vec<Message> {
        self.recent_messages.read()
            .await
            .iter()
            .cloned()
            .collect()
    }
}
