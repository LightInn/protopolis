// simulation.rs
use crate::agent::Agent;
use crate::config::Config;
use crate::conversation_manager::ConversationManager;
use crate::message::Message;
use crate::personality::get_personality_template;
use crate::state::AgentState;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Enum representing commands from the UI to the simulation
pub enum UIToSimulation {
    Start,                       // Start the simulation
    Pause,                       // Pause the simulation
    Resume,                      // Resume the simulation
    Stop,                        // Stop the simulation
    SetDiscussionTopic(String),  // Set the discussion topic
    UserMessage(String, String), // User sends a message to a specific agent
}

/// Enum representing updates from the simulation to the UI
pub enum SimulationToUI {
    TickUpdate(u64),                      // Update with the current tick
    AgentUpdate(String, AgentState, f32), // Update agent's status and energy
    MessageUpdate(Message),               // New message update
    StateUpdate(String),                  // Update the simulation's state
}

/// Main simulation struct
pub struct Simulation {
    config: Config,
    agents: HashMap<String, Agent>,
    messages: Vec<Message>,
    current_tick: u64,
    running: bool,
    paused: bool,
    ui_tx: Sender<SimulationToUI>,
    sim_rx: Receiver<UIToSimulation>,
    discussion_topic: Option<String>,
    runtime: Runtime,
    conversation_manager: ConversationManager,
}

impl Simulation {
    /// Initializes a new simulation with the given configuration and channels.
    pub fn new(
        config: Config,
        ui_tx: Sender<SimulationToUI>,
        sim_rx: Receiver<UIToSimulation>,
    ) -> Self {
        // Create a Tokio runtime for async calls to Ollama
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        // Initialize agents based on configuration
        let mut agents = HashMap::new();
        for agent_config in &config.agents {
            let id = Uuid::new_v4().to_string();
            let personality = get_personality_template(&agent_config.personality_template);

            let mut agent = Agent::new(
                agent_config.name.clone(),
                personality,
                agent_config.initial_energy,
                agent_config.initial_position,
            );

            // Set the Ollama model (this could be added to the config later)
            agent.set_model("llama3.2:latest".to_string());

            agents.insert(id, agent);
        }

        Self {
            config,
            agents,
            messages: Vec::new(),
            current_tick: 0,
            running: false,
            paused: false,
            ui_tx,
            sim_rx,
            discussion_topic: None,
            runtime,
            conversation_manager: ConversationManager::new(),
        }
    }

    /// Starts the simulation loop, listening for commands and processing the simulation.
    pub fn run(&mut self) {
        self.running = true;
        // Wait for the start signal
        while let Ok(command) = self.sim_rx.recv() {
            match command {
                UIToSimulation::Start => {
                    self.running = true;
                    break;
                }
                UIToSimulation::SetDiscussionTopic(topic) => {
                    self.discussion_topic = Some(topic.clone());
                    // Send a topic update to the UI
                    let _ = self.ui_tx.send(SimulationToUI::StateUpdate(format!(
                        "Discussion topic set: {}",
                        topic
                    )));
                    // Start conversation immediately if the topic is set
                    self.start_conversation(&topic);
                }
                UIToSimulation::UserMessage(recipient, content) => {
                    self.handle_user_message(&recipient, &content);
                }
                UIToSimulation::Stop => {
                    self.running = false;
                    break;
                }
                _ => continue,
            }
        }

        // Main simulation loop
        let mut last_tick_time = Instant::now();
        let tick_duration = Duration::from_millis(1000 / 10); // 10 ticks per second

        while self.running {
            // Check UI commands
            if let Ok(command) = self.sim_rx.try_recv() {
                match command {
                    UIToSimulation::Pause => self.paused = true,
                    UIToSimulation::Resume => self.paused = false,
                    UIToSimulation::Stop => self.running = false,
                    UIToSimulation::SetDiscussionTopic(topic) => {
                        self.discussion_topic = Some(topic.clone());
                        self.start_conversation(&topic);
                    }
                    _ => {}
                }
            }

            // If paused, wait
            if self.paused {
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            // Check if it's time for a tick
            let now = Instant::now();
            if now.duration_since(last_tick_time) >= tick_duration {
                self.tick();
                last_tick_time = now;
            } else {
                // Wait a bit to avoid overloading the CPU
                thread::sleep(Duration::from_millis(10));
            }
        }

        // Send a final state update to the UI
        let _ = self.ui_tx.send(SimulationToUI::StateUpdate(
            "Simulation stopped".to_string(),
        ));
    }

    /// Executes a tick in the simulation, updating agent states, messages, and energy levels.
    fn tick(&mut self) {
        self.current_tick += 1;
        let _ = self
            .ui_tx
            .send(SimulationToUI::TickUpdate(self.current_tick));

        // 1. Collect all received messages during this tick
        for message in &self.messages {
            // Add to global conversation history
            self.conversation_manager.add_message(message.clone());

            // For each agent (except the sender), collect what it "hears"
            for (_, agent) in self.agents.iter_mut() {
                if agent.name != message.sender {
                    // The agent hears this message
                    agent.next_prompt.push_str(&format!(
                        "[{}→{}]: {}\n",
                        message.sender,
                        message.recipient,
                        message.content.to_string().trim_matches('"')
                    ));
                }
            }

            // Notify the UI about the new message
            let _ = self
                .ui_tx
                .send(SimulationToUI::MessageUpdate(message.clone()));
        }

        // 2. Make agents respond to the messages they heard
        let mut new_messages = Vec::new();

        for (_, agent) in self.agents.iter_mut() {
            if !agent.next_prompt.is_empty() {
                // The agent has heard messages and will respond
                agent.state = AgentState::Thinking;

                // Notify the UI about the state change
                let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                    agent.name.clone(),
                    agent.state.clone(),
                    agent.energy,
                ));

                // Determine the recipient (for now, we respond to the last message)
                let recipient = if agent.next_prompt.contains("→") {
                    agent
                        .next_prompt
                        .lines()
                        .last()
                        .and_then(|line| line.split('→').next())
                        .unwrap_or("everyone")
                        .trim_start_matches('[')
                        .to_string()
                } else {
                    "everyone".to_string()
                };

                // Generate a response
                if let Ok(response_text) = self
                    .runtime
                    .block_on(async { agent.generate_response_from_prompt().await })
                {
                    // Create a response message
                    let response_message = Message {
                        id: Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        sender: agent.name.clone(),
                        recipient,
                        content: json!(response_text),
                    };

                    // Add to the list of new messages
                    new_messages.push(response_message.clone());

                    // Notify the UI about the response
                    let _ = self
                        .ui_tx
                        .send(SimulationToUI::MessageUpdate(response_message));

                    // Update agent state
                    agent.state = AgentState::Speaking;
                    agent.energy -= 1.0;
                }

                // Reset the prompt for the next tick
                agent.next_prompt.clear();
            }
        }

        // Clear current messages and add new ones
        self.messages.clear();
        self.messages.extend(new_messages);

        // Update agents' energy levels
        for (_, agent) in self.agents.iter_mut() {
            agent.energy += 0.1;
            if agent.energy > 100.0 {
                agent.energy = 100.0;
            }

            let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                agent.name.clone(),
                agent.state.clone(),
                agent.energy,
            ));
        }
    }

    /// Starts the conversation with a given topic.
    fn start_conversation(&mut self, topic: &str) {
        // Choose an agent to start the conversation
        if let Some((_, starter)) = self.agents.iter().next() {
            // Create an initial message
            let initial_message = Message {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                sender: "System".to_string(),
                recipient: starter.name.clone(),
                content: json!(format!("Let's talk about {}. What do you think?", topic)),
            };

            // Add the message to the list
            self.messages.push(initial_message.clone());

            // Send the message to the UI
            let _ = self
                .ui_tx
                .send(SimulationToUI::MessageUpdate(initial_message));
            let _ = self.ui_tx.send(SimulationToUI::StateUpdate(format!(
                "Conversation started on topic: {}",
                topic
            )));
        }
    }

    /// Handles user messages and passes them to the relevant agent.
    fn handle_user_message(&mut self, recipient: &str, content: &str) {
        // Create a user message
        let user_message = Message {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            sender: "User".to_string(),
            recipient: recipient.to_string(),
            content: json!(content),
        };

        // Notify the UI about the user message
        let _ = self
            .ui_tx
            .send(SimulationToUI::MessageUpdate(user_message.clone()));

        // Add to the conversation history
        self.conversation_manager.add_message(user_message.clone());

        // Add the message to the recipient agent's next prompt for immediate processing
        if let Some(agent) = self.agents.values_mut().find(|a| a.name == recipient) {
            agent
                .next_prompt
                .push_str(&format!("[User→{}]: {}\n", recipient, content));

            // Process the response immediately
            agent.state = AgentState::Thinking;
            let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                agent.name.clone(),
                agent.state.clone(),
                agent.energy,
            ));

            // Store the agent's name for later use
            let agent_name = agent.name.clone();

            // Generate a response
            let response_result = self
                .runtime
                .block_on(async { agent.generate_response_from_prompt().await });

            // Release the agent lock once we're done
            if let Ok(response_text) = response_result {
                let response_message = Message {
                    id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    sender: agent_name.clone(),
                    recipient: "User".to_string(),
                    content: json!(response_text),
                };

                // Notify the UI about the agent's response
                let _ = self
                    .ui_tx
                    .send(SimulationToUI::MessageUpdate(response_message));

                // Update the state of other agents
                for (_, other_agent) in self.agents.iter_mut() {
                    if other_agent.name != agent_name {
                        other_agent
                            .next_prompt
                            .push_str(&format!("[{}→User]: {}\n", agent_name, response_text));
                    }
                }

                // Update the agent's state with the new energy level
                if let Some(agent) = self.agents.values_mut().find(|a| a.name == agent_name) {
                    agent.state = AgentState::Speaking;
                    agent.energy -= 1.0;
                    let _ = self.ui_tx.send(SimulationToUI::AgentUpdate(
                        agent.name.clone(),
                        agent.state.clone(),
                        agent.energy,
                    ));
                }
            }

            // Clear the prompt for the next turn
            if let Some(agent) = self.agents.values_mut().find(|a| a.name == agent_name) {
                agent.next_prompt.clear();
            }
        } else {
            let _ = self.ui_tx.send(SimulationToUI::StateUpdate(format!(
                "Agent '{}' not found.",
                recipient
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn setup_simulation() -> (Simulation, Sender<UIToSimulation>, Receiver<SimulationToUI>) {
        let config = Config::default(); // Ensure you have a default implementation for testing
        let (ui_tx, ui_rx) = mpsc::channel();
        let (sim_tx, sim_rx) = mpsc::channel();
        let simulation = Simulation::new(config, ui_tx, sim_rx);
        (simulation, sim_tx, ui_rx)
    }

    #[test]
    fn test_tick_updates() {
        let (mut simulation, sim_tx, ui_rx) = setup_simulation();
        sim_tx.send(UIToSimulation::Start).unwrap();

        thread::spawn(move || {
            simulation.run();
        });

        let response = ui_rx.recv_timeout(Duration::from_secs(1));
        assert!(matches!(response, Ok(SimulationToUI::TickUpdate(_))));
    }
}
