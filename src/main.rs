// main.rs
use ollama_rs::Ollama;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

mod agent;
// mod comms;
mod utils;
mod world;

#[tokio::main]
async fn main() {
    // Initialisation
    let ollama = Ollama::default();
    let mut world = world::World::new(ollama);

    // Configuration initiale
    world.add_agent(agent::Agent::new(1, "Alice", "optimiste"));
    // world.add_agent(agent::Agent::new(2, "Bob", "sceptique"));

    // Entrée utilisateur pour le thème
    let theme = utils::get_user_input("Entrez le thème de la discussion:");
    world.set_initial_topic(&theme);

    // Lancement de la simulation
    world.run().await;

    // Sauvegarde des conversations
    utils::save_conversations(&world.agents());
}