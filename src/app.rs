use std::io;
use std::sync::{Arc, RwLock};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, KeyEvent};
use ratatui::{DefaultTerminal, Frame};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize, Text};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Widget};
use crate::simulation::{Simulation, SimulationEvent, SimulationState};
use crate::message::Message;
use tokio::sync::mpsc;

// États possibles de l'application
#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Initial,      // Écran d'accueil/configuration
    Running,      // Simulation en cours
    Paused,       // Simulation en pause
    Configuration // Écran de configuration
}

// Onglets disponibles
#[derive(Debug, Clone, Copy, PartialEq)]
enum TabState {
    Simulation = 0,
    Configuration = 1,
}

#[derive(Debug)]
pub struct App {
    // État de l'application
    state: AppState,
    // État des onglets
    tab_state: TabState,
    // Paramètres pour la simulation
    topic: String,
    // Messages capturés à afficher
    messages: Vec<String>,
    // Si l'application doit se terminer
    exit: bool,
    // Simulation
    simulation: Option<Arc<RwLock<Simulation>>>,
    // Position du curseur dans le prompt initial
    cursor_position: usize,
    // Sender d'événements pour la simulation
    event_sender: Option<mpsc::Sender<SimulationEvent>>,

    // Stockage des logs
    logs: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::Initial,
            tab_state: TabState::Simulation,
            topic: String::new(),
            messages: Vec::new(),
            exit: false,
            simulation: None,
            cursor_position: 0,
            event_sender: None,
            logs: vec![],
        }
    }
}

impl App {
    /// Exécute la boucle principale de l'application jusqu'à ce que l'utilisateur quitte
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        match self.state {
            AppState::Initial => self.draw_initial_screen(frame),
            AppState::Running | AppState::Paused => self.draw_simulation_screen(frame),
            AppState::Configuration => self.draw_configuration_screen(frame),
        }
    }

    fn draw_initial_screen(&self, frame: &mut Frame) {
        let area = frame.size();

        // Création d'un layout centré verticalement
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Percentage(35),
            ])
            .split(area);

        let title = Line::from(" Simulateur d'Échange entre IA ".bold());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .title(title.centered());

        frame.render_widget(block, area);

        // Affichage du prompt
        let prompt = Paragraph::new("Veuillez entrer un sujet de discussion pour les IA:")
            .centered();
        frame.render_widget(prompt, chunks[1]);

        // Champ de texte pour le sujet
        let input = Paragraph::new(self.topic.as_str())
            .block(Block::default().borders(Borders::ALL))
            .centered();
        frame.render_widget(input, chunks[2]);

        // Positionnement du curseur
        frame.set_cursor(
            chunks[2].x + 1 + self.cursor_position as u16,
            chunks[2].y
        );

        // Instructions
        let instructions = Line::from(vec![
            " Démarrer ".into(),
            "<Entrée>".blue().bold(),
            " Quitter ".into(),
            "<Esc/Q>".blue().bold(),
        ]);

        let instructions_widget = Paragraph::new(instructions)
            .centered();
        frame.render_widget(instructions_widget, chunks[3]);
    }

    fn draw_simulation_screen(&self, frame: &mut Frame) {
        let area = frame.size();


        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Onglets
                Constraint::Min(1),     // Contenu principal
                Constraint::Length(5),  // Logs
                Constraint::Length(1),  // Barre d'état
            ])
            .split(area);
        

        // Onglets
        let titles = vec!["Simulation", "Configuration"];
        let tabs = Tabs::new(titles)
            .select(self.tab_state as usize)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(ratatui::style::Style::default().bold());
        frame.render_widget(tabs, chunks[0]);

        // Contenu principal en fonction de l'onglet sélectionné
        match self.tab_state {
            TabState::Simulation => {
                // Pour l'onglet Simulation, affichage des messages
                let messages_block = Block::default()
                    .title(format!("Discussion sur le sujet: {}", self.topic))
                    .borders(Borders::ALL);

                let messages_items: Vec<ListItem> = self.messages
                    .iter()
                    .map(|msg| ListItem::new(msg.clone()))
                    .collect();

                let messages_list = List::new(messages_items)
                    .block(messages_block);

                frame.render_widget(messages_list, chunks[1]);
            },
            TabState::Configuration => {
                self.draw_configuration_screen(frame);
            }
        }

        // Affichage des logs
        let logs_block = Block::default()
            .title("Logs")
            .borders(Borders::ALL);

        let logs_items: Vec<ListItem> = self.logs
            .iter()
            .map(|log| ListItem::new(log.clone()))
            .collect();

        let logs_list = List::new(logs_items).block(logs_block);
        frame.render_widget(logs_list, chunks[2]);
        
        // Barre d'état
        let status = match self.state {
            AppState::Running => "En cours",
            AppState::Paused => "En pause",
            _ => "",
        };

        let status_line = Line::from(vec![
            " État: ".into(),
            status.to_string().yellow(),
            " | ".into(),
            " Pause ".into(),
            "<Espace>".blue().bold(),
            " Onglets ".into(),
            "<Tab>".blue().bold(),
            " Quitter ".into(),
            "<Q> ".blue().bold(),
        ]);

        let status_widget = Paragraph::new(status_line);
        frame.render_widget(status_widget, chunks[2]);
    }

    fn draw_configuration_screen(&self, frame: &mut Frame) {
        // Pour le moment, affichage basique de la configuration
        let area = frame.size();

        let block = Block::default()
            .title("Configuration")
            .borders(Borders::ALL);

        let config_text = Text::from(vec![
            Line::from("Configuration de la simulation:"),
            Line::from(""),
            Line::from(vec![
                "Sujet: ".into(),
                self.topic.clone().yellow(),
            ]),
            // Vous pourriez ajouter d'autres paramètres ici
        ]);

        let paragraph = Paragraph::new(config_text)
            .block(block);

        frame.render_widget(paragraph, area);
    }

    /// Gestion des événements
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.state {
            AppState::Initial => self.handle_initial_key_event(key_event),
            AppState::Running | AppState::Paused => self.handle_simulation_key_event(key_event),
            AppState::Configuration => self.handle_configuration_key_event(key_event),
        }
    }

    fn handle_initial_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.exit(),
            KeyCode::Enter => {
                if !self.topic.is_empty() {
                    self.start_simulation();
                }
            },
            KeyCode::Char(c) => {
                self.topic.insert(self.cursor_position, c);
                self.cursor_position += 1;
            },
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.topic.remove(self.cursor_position);
                }
            },
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            },
            KeyCode::Right => {
                if self.cursor_position < self.topic.len() {
                    self.cursor_position += 1;
                }
            },
            _ => {}
        }
    }

    fn handle_simulation_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char(' ') => self.toggle_pause(),
            KeyCode::Tab => self.toggle_tab(),
            _ => {}
        }
    }

    fn handle_configuration_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Tab => self.toggle_tab(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    async fn start_simulation(&mut self) {
        // Ici, vous allez initialiser votre simulation avec le sujet
        // Pour l'instant, nous allons simplement changer l'état
        self.state = AppState::Running;

        // Simuler des messages pour test
        self.messages.push(format!("Simulation démarrée sur le sujet: {}", self.topic));
        self.messages.push("Agent 1: Bonjour, comment puis-je vous aider?".to_string());
        self.messages.push("Agent 2: Je suis ici pour discuter du sujet.".to_string());

        // TODO: Initialiser la vraie simulation
        // self.simulation = Some(Arc::new(RwLock::new(Simulation::new(self.topic.clone()))));
    }

    fn toggle_pause(&mut self) {
        match self.state {
            AppState::Running => self.state = AppState::Paused,
            AppState::Paused => self.state = AppState::Running,
            _ => {}
        }
    }

    fn toggle_tab(&mut self) {
        match self.tab_state {
            TabState::Simulation => self.tab_state = TabState::Configuration,
            TabState::Configuration => self.tab_state = TabState::Simulation,
        }
    }

    // Fonction pour ajouter un message à l'interface
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }
    
    
    // set event sender
    pub fn set_event_sender(&mut self, event_sender: mpsc::Sender<SimulationEvent>) {
        self.event_sender = Some(event_sender);
    }

    pub fn log(&mut self, message: String) {
        self.logs.push(message);
        if self.logs.len() > 10 { // Limite d'affichage
            self.logs.remove(0);
        }
    }
}

// Pour la compatibilité avec le code existant
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Cette implémentation est remplacée par les méthodes draw_*
        // mais nous la gardons pour compatibilité avec votre code existant
        let title = Line::from(" Simulateur d'IA ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        Paragraph::new("Appuyez sur Q pour quitter")
            .centered()
            .block(block)
            .render(area, buf);
    }
}