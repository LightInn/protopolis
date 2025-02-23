// main.rs
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

mod agent;
// mod comms;
mod utils;
mod world;

#[tokio::main]
// add debug argument to the main function that can be passed to the program
async fn main() {
    let debug: bool = true;

    // Initialisation
    let ollama = Ollama::default();
    let mut world = world::World::new(ollama, debug);

    // Configuration initiale
    world.add_agent(agent::Agent::new(1, "Alice", "optimistic"));
    world.add_agent(agent::Agent::new(2, "Bob", "sceptique"));

    world.set_initial_topic();

    // Lancement de la simulation
    world.run().await;

    // Sauvegarde des conversations
    utils::save_conversations(&world.agents());
}
