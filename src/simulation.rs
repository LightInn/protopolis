//simulations.rs
use crate::agent;
use crate::config::Config;
use crate::message::MessageBus;
use cli_log::{debug, info};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time;

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
    pub(crate) state: Arc<RwLock<SimulationState>>,
    world_time: Arc<RwLock<WorldTime>>,
    pub(crate) agents: Vec<Arc<RwLock<agent::Agent>>>,
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
            let event_clone = event.clone();

            // Utiliser spawn_blocking pour éviter de bloquer le thread courant
            tokio::spawn(async move {
                match sender_clone.send(event_clone).await {
                    Ok(_) => {}
                    Err(e) => eprintln!("Erreur lors de l'envoi d'un événement: {}", e),
                }
            });
        }
    }

    pub async fn get_state(&self) -> SimulationState {
        self.state.read().await.clone()
    }

    pub async fn pause(&self) {
        let mut state = self.state.write().await;
        if *state == SimulationState::Running {
            info!("Simulation mise en pause");
            *state = SimulationState::Paused;
            self.send_event(SimulationEvent::StateChange(
                "Simulation mise en pause".to_string(),
            ));
        }
    }

    pub async fn resume(&self) {
        let mut state = self.state.write().await;
        if *state == SimulationState::Paused {
            info!("Simulation reprise");
            *state = SimulationState::Running;
            self.send_event(SimulationEvent::StateChange(
                "Simulation reprise".to_string(),
            ));
        }
    }

    pub async fn toggle_pause(&self) {
        let state = self.state.read().await.clone();
        match state {
            SimulationState::Running => self.pause().await,
            SimulationState::Paused => self.resume().await,
            _ => {}
        }
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_event(SimulationEvent::Message(
            "Initialisation de la simulation...".to_string(),
        ));

        // Charger la configuration
        let config = match Config::load(Path::new("config.json")) {
            Ok(config) => {
                self.send_event(SimulationEvent::Message(
                    "Configuration chargée avec succès".to_string(),
                ));
                config
            }
            Err(e) => {
                let message = format!("Erreur de chargement de la configuration: {}", e);
                self.send_event(SimulationEvent::Message(message.clone()));

                self.send_event(SimulationEvent::Message(
                    "Création d'une configuration par défaut...".to_string(),
                ));
                let conf = Config::default();
                conf.save(Path::new("config.json"))?;

                self.send_event(SimulationEvent::Message(
                    "Configuration par défaut créée".to_string(),
                ));
                conf
            }
        };

        debug!("Configuration chargée: {:#?}", config);

        self.debug = config.debug;

        // Initialiser le message bus
        let msg_bus = MessageBus::new(config.debug);
        self.send_event(SimulationEvent::Message(
            "Bus de message initialisé".to_string(),
        ));

        // Mettre à jour le world_time avec la config
        let mut world_time = self.world_time.write().await;
        *world_time = WorldTime::new(config.world.ticks_per_hour);
        self.send_event(SimulationEvent::Message(format!(
            "Temps mondial configuré: {} ticks/heure",
            config.world.ticks_per_hour
        )));

        // Créer les agents à partir de la configuration
        self.send_event(SimulationEvent::Message(format!(
            "Création de {} agents...",
            config.agents.len()
        )));

        for (i, agent_config) in config.agents.iter().enumerate() {
            let agent = agent::Agent::new(agent_config, msg_bus.clone().into(), config.debug);
            self.send_event(SimulationEvent::Message(format!(
                "Agent {}/{} créé: {}",
                i + 1,
                config.agents.len(),
                agent_config.name
            )));

            // Injecter le sujet dans les agents
            {
                let mut agent_write = agent.write().await;
                agent_write.set_topic(&self.topic);
            }

            msg_bus.register_agent(agent.clone());
            self.agents.push(agent);
        }

        debug!("Agents initialisés: {:#?}", self.agents);

        let message = format!(
            "Simulation initialisée avec {} agents sur le sujet: {}",
            self.agents.len(),
            self.topic
        );
        self.send_event(SimulationEvent::Message(message));

        // Changer l'état à Running
        let mut state = self.state.write().await;
        *state = SimulationState::Running;
        self.send_event(SimulationEvent::StateChange(
            "Simulation démarrée".to_string(),
        ));

        Ok(())
    }

    // Modification de la méthode run_async pour améliorer la gestion des états et fournir plus de retours
    pub async fn run_async(
        simulation: Arc<RwLock<Self>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialisation
        {
            let mut sim = simulation.write().await;
            sim.send_event(SimulationEvent::Message(
                "Démarrage de l'initialisation...".to_string(),
            ));

            if let Err(e) = sim.initialize().await {
                let error_msg = format!("Erreur d'initialisation: {}", e);
                sim.send_event(SimulationEvent::Message(error_msg.clone()));
                return Err(error_msg.into());
            }
        }

        // Boucle principale
        let mut interval = time::interval(Duration::from_millis(500)); // Un peu plus lent pour moins charger le CPU
        let mut last_hour = 0;

        info!("Simulation démarrée");
        simulation.read().await.send_event(SimulationEvent::Message(
            "Début de la simulation".to_string(),
        ));

        loop {
            interval.tick().await;

            // Vérifier l'état
            let state;
            {
                let sim = simulation.read().await;
                state = sim.state.read().await.clone();
            }

            match state {
                SimulationState::Running => {
                    // Mise à jour du temps
                    {
                        let sim = simulation.read().await;
                        let mut world_time = sim.world_time.write().await;
                        world_time.increment();

                        let current_hour = world_time.get_hour();

                        // Envoyer un message seulement quand l'heure change
                        if current_hour != last_hour {
                            last_hour = current_hour;
                            let message = format!("Heure mondiale actuelle: {}", current_hour);
                            sim.send_event(SimulationEvent::Message(message));
                        }
                    }

                    // Mise à jour des agents
                    let current_tick;
                    let agents;
                    let debug;
                    {
                        let sim = simulation.read().await;
                        current_tick = sim.world_time.read().await.get_tick();
                        agents = sim.agents.clone();
                        debug = sim.debug;
                    }

                    for agent in &agents {
                        let agent_name;
                        {
                            let agent_read = agent.read().await;
                            agent_name = agent_read.name.clone();
                        }

                        let mut agent = agent.write().await;
                        if let Err(e) = agent.update(current_tick).await {
                            let sim = simulation.read().await;
                            let message =
                                format!("Erreur de mise à jour de l'agent {}: {}", agent.name, e);
                            sim.send_event(SimulationEvent::Message(message));
                        }

                        if debug {
                            let sim = simulation.read().await;
                            let message = format!(
                                "{}: État: {:?}, Énergie: {}",
                                agent.name,
                                agent.get_state(),
                                agent.get_energy()
                            );
                            sim.send_event(SimulationEvent::Message(message));
                        }
                    }

                    // Vérifier la condition de sortie
                    {
                        let sim = simulation.read().await;
                        let hour = sim.world_time.read().await.get_hour();
                        if hour >= 24 {
                            // 24 heures simulées
                            let mut state = sim.state.write().await;
                            *state = SimulationState::Finished;
                            sim.send_event(SimulationEvent::Message(
                                "Simulation terminée (24 heures complétées)".to_string(),
                            ));
                            sim.send_event(SimulationEvent::Finished);
                            break;
                        }
                    }
                }
                SimulationState::Paused => {
                    // En pause, on attend simplement
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                SimulationState::Finished => {
                    break;
                }
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
