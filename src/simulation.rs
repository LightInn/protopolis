// simulation.rs
use crate::config::Config;
use crate::message::MessageBus;
use crate::{agent};
use std::path::Path;
use std::time::Duration;
use tokio::time;


/// World time management
struct WorldTime {
    current_tick: u64,
    ticks_per_hour: u32,
}

impl WorldTime {
    fn new(ticks_per_hour: u32) -> Self {
        Self {
            current_tick: 0,
            ticks_per_hour,
        }
    }

    fn increment(&mut self) {
        self.current_tick += 1;
    }

    fn get_hour(&self) -> u64 {
        self.current_tick / self.ticks_per_hour as u64
    }
}


pub struct Simulation {}

impl Simulation {
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        // Load configuration
        let config = match Config::load(Path::new("config.json")) {
            Ok(config) => config,
            Err(e) => {
                println!("Error loading config: {}", e);
                println!("No config cannot be loaded, using default configuration");
                let conf = Config::default();
                conf.save(Path::new("config.json"))?;
                conf
            }
        };

        if config.debug {
            println!("Debug is {}", config.debug);
        }
        // Initialize message bus
        let msg_bus = MessageBus::new(config.debug);

        // Initialize world time
        let mut world_time = WorldTime::new(config.world.ticks_per_hour);

        // Create agents from configuration
        let mut agents = Vec::new();
        for agent_config in &config.agents {
            let agent = agent::Agent::new(agent_config, msg_bus.clone().into(), config.debug);

            msg_bus.register_agent(agent.clone());
            agents.push(agent);
        }

        // Main simulation loop
        let mut interval = time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            world_time.increment();

            if config.debug {
                println!("World time: Hour {}", world_time.get_hour());
            }
            // Update all agents
            for agent in &agents {
                let mut agent = agent.write().unwrap();
                agent.update(world_time.current_tick).await?;

                if config.debug {
                    println!("{}: {:?}", agent.name, agent.get_state());
                    println!("{}: Energy: {}", agent.name, agent.get_energy());
                }
            }

            // Optional: Break condition (e.g., after 24 simulation hours)
            if world_time.get_hour() >= 1 {
                break;
            }
        }

        Ok(())
    }
}
