use crate::config::Config;
use crate::message::MessageBus;
use crate::{agent};
use std::path::Path;
use std::sync::{Arc, RwLock, Mutex};
use std::time::Duration;
use cli_log::{debug, info};
use tokio::task;
use tokio::time;
use tokio::sync::mpsc;

/// World time management
#[derive(Debug, Clone)]
pub struct WorldTime {
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

    pub fn get_hour(&self) -> u64 {
        self.current_tick / self.ticks_per_hour as u64
    }

    pub fn get_tick(&self) -> u64 {
        self.current_tick
    }
}

/// Événements de simulation
#[derive(Debug, Clone)]
pub enum SimulationEvent {
    Message(String),
    StateChange(String),
    Finished,
}

/// État de la simulation
#[derive(Debug, Clone, PartialEq)]
pub enum SimulationState {
    Initializing,
    Running,
    Paused,
    Finished,
}

#[derive(Debug)]
pub struct Simulation {
    state: Arc<RwLock<SimulationState>>,
    world_time: Arc<RwLock<WorldTime>>,
    agents: Vec<Arc<RwLock<agent::Agent>>>,
    topic: String,
    debug: bool,
    event_sender: Option<mpsc::Sender<SimulationEvent>>,
}

impl Simulation {
    pub fn new(topic: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(SimulationState::Initializing)),
            world_time: Arc::new(RwLock::new(WorldTime::new(4))), // Par défaut 4 ticks par heure
            agents: Vec::new(),
            topic,
            debug: false,
            event_sender: None,
        }
    }

    pub fn set_event_sender(&mut self, sender: mpsc::Sender<SimulationEvent>) {
        self.event_sender = Some(sender);
    }

    fn send_event(&self, event: SimulationEvent) {
        if let Some(sender) = &self.event_sender {
            let sender_clone = sender.clone();
            tokio::spawn(async move {
                if let Err(e) = sender_clone.send(event).await {
                    eprintln!("Erreur lors de l'envoi d'un événement: {}", e);
                }
            });
        }
    }

    pub fn get_state(&self) -> SimulationState {
        self.state.read().unwrap().clone()
    }

    pub fn pause(&self) {
        let mut state = self.state.write().unwrap();
        if *state == SimulationState::Running {
            info!("Simulation mise en pause");
            *state = SimulationState::Paused;
            self.send_event(SimulationEvent::StateChange("Simulation mise en pause".to_string()));
        }
    }

    pub fn resume(&self) {
        let mut state = self.state.write().unwrap();
        if *state == SimulationState::Paused {
            info!("Simulation reprise");
            *state = SimulationState::Running;
            self.send_event(SimulationEvent::StateChange("Simulation reprise".to_string()));
        }
    }

    pub fn toggle_pause(&self) {
        let state = self.state.read().unwrap().clone();
        match state {
            SimulationState::Running => self.pause(),
            SimulationState::Paused => self.resume(),
            _ => {}
        }
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Charger la configuration
        let config = match Config::load(Path::new("config.json")) {
            Ok(config) => config,
            Err(e) => {
                let message = format!("Erreur de chargement de la configuration: {}", e);
                self.send_event(SimulationEvent::Message(message));

                let conf = Config::default();
                conf.save(Path::new("config.json"))?;
                conf
            }
        };
        
        debug!("Configuration chargée: {:#?}", config);

        self.debug = config.debug;

        // Initialiser le message bus
        let msg_bus = MessageBus::new(config.debug);

        // Mettre à jour le world_time avec la config
        let mut world_time = self.world_time.write().unwrap();
        *world_time = WorldTime::new(config.world.ticks_per_hour);

        // Créer les agents à partir de la configuration
        for agent_config in &config.agents {
            let agent = agent::Agent::new(agent_config, msg_bus.clone().into(), config.debug);

            // Injecter le sujet dans les agents
            {
                let mut agent_write = agent.write().unwrap();
                agent_write.set_topic(&self.topic);
            }

            msg_bus.register_agent(agent.clone());
            self.agents.push(agent);
        }

        debug!("Agents initialisés: {:#?}", self.agents);
        
        let message = format!("Simulation initialisée avec {} agents sur le sujet: {}",
                              self.agents.len(), self.topic);
        self.send_event(SimulationEvent::Message(message));

        // Changer l'état à Running
        let mut state = self.state.write().unwrap();
        *state = SimulationState::Running;

        Ok(())
    }

    pub async fn run_async(simulation: Arc<RwLock<Self>>) -> Result<(), Box<dyn std::error::Error>> {
        // Initialisation
        {
            let mut sim = simulation.write().unwrap();
            sim.initialize().await?;
        }

        // Boucle principale
        let mut interval = time::interval(Duration::from_millis(100));

        info!("Simulation démarrée");
        loop {
            interval.tick().await;

            // Vérifier l'état
            let state;
            {
                let sim = simulation.read().unwrap();
                state = sim.state.read().unwrap().clone();
            }

            match state {
                SimulationState::Running => {
                    // Mise à jour du temps
                    {
                        let sim = simulation.read().unwrap();
                        let mut world_time = sim.world_time.write().unwrap();
                        world_time.increment();

                        if sim.debug {
                            let message = format!("Heure mondiale: {}", world_time.get_hour());
                            sim.send_event(SimulationEvent::Message(message));
                        }
                    }

                    // Mise à jour des agents
                    let current_tick;
                    let agents;
                    let debug;
                    {
                        let sim = simulation.read().unwrap();
                        current_tick = sim.world_time.read().unwrap().get_tick();
                        agents = sim.agents.clone();
                        debug = sim.debug;
                    }

                    for agent in &agents {
                        let mut agent = agent.write().unwrap();
                        if let Err(e) = agent.update(current_tick).await {
                            let sim = simulation.read().unwrap();
                            let message = format!("Erreur de mise à jour de l'agent {}: {}", agent.name, e);
                            sim.send_event(SimulationEvent::Message(message));
                        }

                        if debug {
                            let sim = simulation.read().unwrap();
                            let message = format!("{}: État: {:?}, Énergie: {}",
                                                  agent.name, agent.get_state(), agent.get_energy());
                            sim.send_event(SimulationEvent::Message(message));
                        }
                    }

                    // Vérifier la condition de sortie
                    {
                        let sim = simulation.read().unwrap();
                        let hour = sim.world_time.read().unwrap().get_hour();
                        if hour >= 1 {  // 24 heures simulées
                            let mut state = sim.state.write().unwrap();
                            *state = SimulationState::Finished;
                            sim.send_event(SimulationEvent::Finished);
                            break;
                        }
                    }
                },
                SimulationState::Paused => {
                    // En pause, on attend simplement
                    tokio::time::sleep(Duration::from_millis(100)).await;
                },
                SimulationState::Finished => {
                    break;
                },
                _ => {}
            }
        }

        Ok(())
    }

    // Pour compatibilité avec l'ancien code
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let simulation = Simulation::new("Discussion générale".to_string());
        let simulation_arc = Arc::new(RwLock::new(simulation));

        Self::run_async(simulation_arc).await
    }
}