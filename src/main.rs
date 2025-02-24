use std::thread;
// main.rs
use crate::agent::Agent;
use crate::messages::{Message, MessageBus};
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

mod agent;
// mod comms;
mod messages;
mod utils;
mod world;

#[tokio::main]
// add debug argument to the main function that can be passed to the program
async fn main() {
    // Initialisation
    let msg_bus = MessageBus::new();

    // Création des agents
    let agent_alice = Agent::new(1, "Alice", "optimiste", msg_bus.clone());
    let agent_bob = Agent::new(2, "Bob", "sceptique", msg_bus.clone());

    // Enregistrement des agents dans le bus
    msg_bus.register_agent(&agent_alice);
    msg_bus.register_agent(&agent_bob);

    // Envoi d'un message broadcast dans un rayon défini (ex: rayon 100)
    let message = Message {
        sender: "System".to_string(),
        recipient: "".to_string(), // vide pour broadcast
        content: "Salut à tous, prêt pour l'aventure ?".to_string(),
        timestamp: 1,
    };

    msg_bus.broadcast_system_message(message);

    // Traitement des messages pour chaque agent
    agent_alice.process_messages();
    agent_bob.process_messages();

    // let debug: bool = true;
    //
    // // Initialisation
    // let ollama = Ollama::default();
    // let mut world = world::World::new(ollama, debug);
    // let msg_bus = messages::MessageBus::new();
    //
    // // Configuration initiale
    // world.add_agent(agent::Agent::new(1, "Alice", "optimistic"));
    // world.add_agent(agent::Agent::new(2, "Bob", "sceptique"));
    //
    // // Enregistrement des agents dans le bus de messages
    // for agent in world.agents() {
    //     msg_bus.register_agent(agent);
    // }
    //
    // world.set_initial_topic();
    //
    // // Lancement de la simulation
    // world.run().await;
    //
    // // Sauvegarde des conversations
    // utils::save_conversations(&world.agents());
}
