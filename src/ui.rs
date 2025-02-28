// ui.rs
use std::io;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Terminal,
};

use crate::message::Message;
use crate::simulation::{SimulationToUI, UIToSimulation};

pub struct UI {
    sim_tx: Sender<UIToSimulation>,
    sim_rx: Receiver<SimulationToUI>,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: String,
    current_tab: usize,
    messages: Vec<Message>,
    agent_states: Vec<(String, String, u32)>, // nom, état, énergie
    current_tick: u64,
}

impl UI {
    pub fn new(
        sim_tx: Sender<UIToSimulation>,
        sim_rx: Receiver<SimulationToUI>,
    ) -> Result<Self, io::Error> {
        // Configurer le terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            sim_tx,
            sim_rx,
            terminal,
            state: "Initialisation".to_string(),
            current_tab: 0,
            messages: Vec::new(),
            agent_states: Vec::new(),
            current_tick: 0,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut last_draw = Instant::now();

        // Envoyer le signal de démarrage à la simulation
        let _ = self.sim_tx.send(UIToSimulation::Start);

        loop {
            // Vérifier les messages de la simulation
            while let Ok(message) = self.sim_rx.try_recv() {
                match message {
                    SimulationToUI::Status(status) => {
                        self.state = status;
                    }
                    SimulationToUI::AgentUpdate(name, state, energy) => {
                        // Mettre à jour ou ajouter l'état de l'agent
                        if let Some(agent) = self.agent_states.iter_mut().find(|(n, _, _)| n == &name) {
                            *agent = (name, state, energy);
                        } else {
                            self.agent_states.push((name, state, energy));
                        }
                    }
                    SimulationToUI::Messages(messages) => {
                        // Ajouter les nouveaux messages
                        self.messages.extend(messages);
                    }
                    SimulationToUI::Tick(tick) => {
                        self.current_tick = tick;
                    }
                }
            }

            // Dessiner l'interface toutes les 100ms
            if last_draw.elapsed() >= Duration::from_millis(100) {
                self.draw()?;
                last_draw = Instant::now();
            }

            // Traiter les événements clavier
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        // Envoyer un signal d'arrêt à la simulation
                        let _ = self.sim_tx.send(UIToSimulation::Stop);
                        break;
                    } else if key.code == KeyCode::Char('p') {
                        // Mettre en pause ou reprendre la simulation
                        if self.state == "Running" {
                            let _ = self.sim_tx.send(UIToSimulation::Pause);
                        } else if self.state == "Paused" {
                            let _ = self.sim_tx.send(UIToSimulation::Resume);
                        }
                    } else if key.code == KeyCode::Tab {
                        // Changer d'onglet
                        self.current_tab = (self.current_tab + 1) % 3;
                    } else if key.code == KeyCode::BackTab {
                        // Onglet précédent
                        self.current_tab = (self.current_tab + 2) % 3;
                    }
                }
            }
        }

        // Restaurer le terminal
        disable_raw_mode()?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    fn draw(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| {
            // Créer la mise en page principale
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),  // Barre d'état
                    Constraint::Length(3),  // Onglets
                    Constraint::Min(0),     // Contenu principal
                ])
                .split(f.size());

            // Barre d'état
            let status = Paragraph::new(format!(
                "État: {} | Tick: {} | Appuyez sur 'q' pour quitter, 'p' pour pause/reprise, Tab pour changer d'onglet",
                self.state, self.current_tick
            ))
                .block(Block::default().borders(Borders::ALL).title("Simulation"));
            f.render_widget(status, chunks[0]);

            // Onglets
            let titles = vec!["Agents", "Messages", "Configuration"];
            let tabs = Tabs::new(titles.iter().map(|t| t.to_string()).collect::<Vec<_>>())
                .block(Block::default().borders(Borders::ALL))
                .select(self.current_tab)
                .style(Style::default())
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            f.render_widget(tabs, chunks[1]);

            // Contenu principal selon l'onglet sélectionné
            match self.current_tab {
                0 => {
                    // Onglet Agents
                    let agent_items: Vec<ListItem> = self.agent_states
                        .iter()
                        .map(|(name, state, energy)| {
                            ListItem::new(format!("{}: État={}, Énergie={}", name, state, energy))
                        })
                        .collect();

                    let agents_list = List::new(agent_items)
                        .block(Block::default().borders(Borders::ALL).title("Agents"))
                        .style(Style::default())
                        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

                    f.render_widget(agents_list, chunks[2]);
                }
                1 => {
                    // Onglet Messages
                    let message_items: Vec<ListItem> = self.messages
                        .iter()
                        .rev()
                        .take(50)  // Limiter à 50 messages pour éviter de surcharger l'affichage
                        .map(|msg| {
                            let content = match &msg.content {
                                serde_json::Value::String(s) => s.clone(),
                                _ => format!("{:?}", msg.content),
                            };
                            ListItem::new(format!(
                                "[{}] {} -> {}: {}",
                                msg.timestamp.format("%H:%M:%S"),
                                msg.sender,
                                msg.recipient,
                                content
                            ))
                        })
                        .collect();

                    let messages_list = List::new(message_items)
                        .block(Block::default().borders(Borders::ALL).title("Messages"))
                        .style(Style::default());

                    f.render_widget(messages_list, chunks[2]);
                }
                2 => {
                    // Onglet Configuration
                    let config_text = Paragraph::new(
                        "Configuration de la simulation\n\
                        (Cette section permettrait de modifier les paramètres en temps réel)"
                    )
                        .block(Block::default().borders(Borders::ALL).title("Configuration"));

                    f.render_widget(config_text, chunks[2]);
                }
                _ => {}
            }
        })?;

        Ok(())
    }
}

