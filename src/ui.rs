use crate::message::Message;
use crate::simulation::{SimulationToUI, UIToSimulation};
use crate::state::AgentState;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::layout::{Alignment, Margin, Position};
use ratatui::prelude::CrosstermBackend;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::collections::{HashMap, VecDeque};
use std::io::{self, stdout, Stdout};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};
use color_eyre::owo_colors::OwoColorize;
use ratatui::widgets::{Padding, Scrollbar, ScrollbarOrientation, ScrollbarState};

// Map of colors for agents
const COLORS: [Color; 8] = [
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::LightRed,
    Color::LightGreen,
];

/// UI struct for managing the TUI interface
pub struct UI {
    ui_tx: Sender<UIToSimulation>,
    ui_rx: Receiver<SimulationToUI>,
    agent_colors: HashMap<String, Color>,
    input: String,
    messages: VecDeque<FormattedMessage>,
    agent_states: HashMap<String, (AgentState, f32)>,
    simulation_status: String,
    current_tick: u64,
    should_quit: bool,
    message_scroll: usize,
    message_scroll_state: ScrollbarState,
}

/// A formatted message with sender/recipient information
struct FormattedMessage {
    sender: String,
    sender_color: Color,
    recipient: String,
    recipient_color: Color,
    content: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl UI {
    /// Creates a new UI instance
    pub fn new(ui_tx: Sender<UIToSimulation>, ui_rx: Receiver<SimulationToUI>) -> Self {
        Self {
            ui_tx,
            ui_rx,
            agent_colors: HashMap::new(),
            input: String::new(),
            messages: VecDeque::with_capacity(100),
            agent_states: HashMap::new(),
            simulation_status: "Waiting to start".to_string(),
            current_tick: 0,
            should_quit: false,
            message_scroll: 0,
            message_scroll_state: ScrollbarState::default(),
        }
    }

    /// Get the color for an agent
    fn get_agent_color(&mut self, agent_name: &str) -> Color {
        if !self.agent_colors.contains_key(agent_name) {
            let color_index = self.agent_colors.len() % COLORS.len();
            self.agent_colors
                .insert(agent_name.to_string(), COLORS[color_index]);
        }
        *self.agent_colors.get(agent_name).unwrap()
    }

    /// Add a message to the message history
    fn add_message(&mut self, message: &Message) {
        let sender_color = match message.sender.as_str() {
            "User" => Color::White,
            "System" => Color::Blue,
            _ => self.get_agent_color(&message.sender),
        };

        let recipient_color = match message.recipient.as_str() {
            "User" => Color::White,
            "System" => Color::Blue,
            "everyone" => Color::Gray,
            _ => self.get_agent_color(&message.recipient),
        };

        self.messages.push_back(FormattedMessage {
            sender: message.sender.clone(),
            sender_color,
            recipient: message.recipient.clone(),
            recipient_color,
            content: message.content.to_string().trim_matches('"').to_string(),
            timestamp: message.timestamp,
        });

        self.message_scroll = self.messages.len();
        self.message_scroll_state = self.message_scroll_state
            .content_length(self.messages.len())
            .position(self.message_scroll);

        // Keep message history limited
        if self.messages.len() > 100 {
            self.messages.pop_front();
        }
    }

    /// Process a command from the input field
    fn process_command(&mut self, command: &str) {
        let command = command.trim();

        match command {
            "start" => {
                let _ = self.ui_tx.send(UIToSimulation::Start);
                self.simulation_status = "Starting simulation...".to_string();
            }
            "pause" => {
                let _ = self.ui_tx.send(UIToSimulation::Pause);
                self.simulation_status = "Pausing simulation...".to_string();
            }
            "resume" => {
                let _ = self.ui_tx.send(UIToSimulation::Resume);
                self.simulation_status = "Resuming simulation...".to_string();
            }
            "stop" => {
                let _ = self.ui_tx.send(UIToSimulation::Stop);
                self.simulation_status = "Stopping simulation...".to_string();
            }
            "exit" => {
                let _ = self.ui_tx.send(UIToSimulation::Stop);
                self.should_quit = true;
            }
            _ if command.starts_with("topic ") => {
                let topic = command.trim_start_matches("topic ").to_string();
                let _ = self
                    .ui_tx
                    .send(UIToSimulation::SetDiscussionTopic(topic.clone()));
                self.simulation_status = format!("Discussion topic set: {}", topic);
            }
            _ if command.starts_with("msg ") => {
                let parts: Vec<&str> = command.splitn(3, ' ').collect();
                if parts.len() == 3 {
                    let agent_name = parts[1];
                    let message = parts[2];
                    let _ = self.ui_tx.send(UIToSimulation::UserMessage(
                        agent_name.to_string(),
                        message.to_string(),
                    ));
                    self.simulation_status = format!("Message sent to {}", agent_name);
                } else {
                    self.simulation_status =
                        "Incorrect format. Use: msg <agent> <message>".to_string();
                }
            }
            _ => {
                self.simulation_status =
                    "Unrecognized command. Try 'start', 'pause', 'resume', 'stop', 'topic <subject>', 'msg <agent> <message>' or 'exit'."
                        .to_string();
            }
        }
    }

    /// Main UI loop
    pub fn run(&mut self) -> Result<(), io::Error> {
        // Terminal setup
        enable_raw_mode()?;
        let mut stdout = stdout();
        // execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        execute!(stdout, EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        // Render splash screen
        self.render_splash_screen(&mut terminal)?;

        // Show welcome message
        self.messages.push_back(FormattedMessage {
            sender: "System".to_string(),
            sender_color: Color::Blue,
            recipient: "User".to_string(),
            recipient_color: Color::White,
            content: "Welcome to Protopolis! Type commands below to interact.".to_string(),
            timestamp: chrono::Utc::now(),
        });

        self.messages.push_back(FormattedMessage {
            sender: "System".to_string(),
            sender_color: Color::Blue,
            recipient: "User".to_string(),
            recipient_color: Color::White,
            content: "Available commands: start, pause, resume, stop, topic <subject>, msg <agent> <message>, exit".to_string(),
            timestamp: chrono::Utc::now(),
        });

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        // Main event loop
        while !self.should_quit {
            terminal.draw(|f| self.ui(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            // Check for events
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => {
                                let input_clone = self.input.clone();
                                self.process_command(&input_clone);
                                self.input.clear();
                            }
                            KeyCode::Char(c) => {
                                if c.is_alphanumeric() || c.is_whitespace() {
                                    self.input.push(c);
                                }
                            }
                            KeyCode::Backspace => {
                                self.input.pop();
                            }
                            KeyCode::Esc => {
                                self.should_quit = true;
                            },
                            KeyCode::PageUp => {
                                self.message_scroll = self.message_scroll.saturating_sub(10);
                                self.message_scroll_state = self.message_scroll_state.position(self.message_scroll);
                            },
                            KeyCode::PageDown => {
                                self.message_scroll = self.message_scroll.saturating_add(10);
                                self.message_scroll_state = self.message_scroll_state.position(self.message_scroll);
                            },
                            KeyCode::Home => {
                                self.message_scroll = 0;
                                self.message_scroll_state = self.message_scroll_state.position(0);
                            },
                            KeyCode::End => {
                                self.message_scroll = self.messages.len();
                                self.message_scroll_state = self.message_scroll_state.position(self.message_scroll);
                            },
                            _ => {}
                        }
                    }
                }
            }

            // Check for simulation updates
            while let Ok(update) = self.ui_rx.try_recv() {
                match update {
                    SimulationToUI::TickUpdate(tick) => {
                        self.current_tick = tick;
                    }
                    SimulationToUI::AgentUpdate(name, state, energy) => {
                        self.agent_states.insert(name, (state, energy));
                    }
                    SimulationToUI::MessageUpdate(message) => {
                        self.add_message(&message);
                    }
                    SimulationToUI::StateUpdate(state) => {
                        self.simulation_status = state;
                    }
                }
            }

            // Check if we should tick
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        let _ = self.ui_tx.send(UIToSimulation::Stop);
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            // DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Draw the UI
    fn ui(&self, f: &mut Frame) {
        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(5),    // Main content
                Constraint::Length(3), // Input
            ])
            .split(f.area());

        // Title bar with status
        let title = Paragraph::new(vec![Line::from(vec![
            Span::styled("Protopolis", Style::default().fg(Color::Cyan)),
            Span::raw(" | "),
            Span::raw(format!("Tick: {}", self.current_tick)),
            Span::raw(" | "),
            Span::raw(&self.simulation_status),
        ])])
        .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(title, chunks[0]);

        // Split the main content area
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70), // Messages
                Constraint::Percentage(30), // Agent states
            ])
            .split(chunks[1]);

        // Messages area
        self.render_messages_panel(f, main_chunks[0]);

        // Agent states panel
        self.render_agent_states_panel(f, main_chunks[1]);

        // Input field
        let input = Paragraph::new(self.input.as_str())
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[2]);

        // Set cursor position
        f.set_cursor_position(Position::new(
            chunks[2].x + self.input.len() as u16 + 1,
            chunks[2].y + 1,
        ));
    }

    /// Render the messages panel
    fn render_messages_panel(&self, f: &mut Frame, area: Rect) {
        // Create message content with proper text wrapping
        let mut text = Vec::new();
        for m in &self.messages {
            // Header line with sender and recipient
            text.push(Line::from(vec![
                Span::styled(
                    format!("[{}]", m.sender),
                    Style::default().fg(m.sender_color),
                ),
                Span::raw(" to "),
                Span::styled(
                    format!("[{}]:", m.recipient),
                    Style::default().fg(m.recipient_color),
                ),
            ]));

            // Content line with automatic wrapping
            text.push(Line::from(Span::raw(&m.content)));

            // Empty line as separator
            text.push(Line::from(""));
        }

        // Calculate appropriate scroll position
        let content_height = text.len();
        let viewport_height = area.height.saturating_sub(2) as usize; // -2 for borders
        let max_scroll = content_height.saturating_sub(viewport_height);
        let scroll = self.message_scroll.min(max_scroll);

        // Render the message content with scroll applied
        let messages_widget = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Messages"))
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((scroll as u16, 0));

        f.render_widget(messages_widget, area);

        // Render the scrollbar if content exceeds viewport
        if content_height > viewport_height {
            f.render_stateful_widget(
                Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                area.inner(Margin { vertical: 1, horizontal: 0 }),
                &mut self.message_scroll_state.clone().content_length(content_height).position(scroll)
            );
        }
    }

    /// Render the agent states panel
    fn render_agent_states_panel(&self, f: &mut Frame, area: Rect) {
        let agents: Vec<ListItem> = self
            .agent_states
            .iter()
            .map(|(name, (state, energy))| {
                let state_color = match state {
                    AgentState::Idle => Color::DarkGray,
                    AgentState::Thinking => Color::Yellow,
                    AgentState::Speaking => Color::Green,
                    _ => Color::White,
                };

                let energy_color = if *energy < 30.0 {
                    Color::Red
                } else if *energy < 70.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                let agent_color = self.agent_colors.get(name).unwrap_or(&Color::White);

                let content = Line::from(vec![
                    Span::styled(name, Style::default().fg(*agent_color)),
                    Span::raw(" - "),
                    Span::styled(format!("{}", state), Style::default().fg(state_color)),
                    Span::raw(" - "),
                    Span::styled(format!("{:.1}", energy), Style::default().fg(energy_color)),
                ]);

                ListItem::new(content)
            })
            .collect();

        let agents_list =
            List::new(agents).block(Block::default().borders(Borders::ALL).title("Agents"));

        f.render_widget(agents_list, area);
    }

    fn render_splash_screen(
        &self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), io::Error> {
        let splash_text = r#"
 ,ggggggggggg,                                                                                          
dP"""88""""""Y8,                      I8                                          ,dPYb,                
Yb,  88      `8b                      I8                                          IP'`Yb                
 `"  88      ,8P                   88888888                                       I8  8I  gg            
     88aaaad8P"                       I8                                          I8  8'  ""            
     88"""""   ,gggggg,    ,ggggg,    I8      ,ggggg,    gg,gggg,      ,ggggg,    I8 dP   gg     ,g,    
     88        dP""""8I   dP"  "Y8ggg I8     dP"  "Y8ggg I8P"  "Yb    dP"  "Y8ggg I8dP    88    ,8'8,   
     88       ,8'    8I  i8'    ,8I  ,I8,   i8'    ,8I   I8'    ,8i  i8'    ,8I   I8P     88   ,8'  Yb  
     88      ,dP     Y8,,d8,   ,d8' ,d88b, ,d8,   ,d8'  ,I8 _  ,d8' ,d8,   ,d8'  ,d8b,_ _,88,_,8'_   8) 
     88      8P      `Y8P"Y8888P"  88P""Y88P"Y8888P"    PI8 YY88888PP"Y8888P"    8P'"Y888P""Y8P' "YY8P8P
                                                         I8                                             
                                                         I8                                             
                                                         I8                                             
                                                         I8                                             
                                                         I8                                             
                                                         I8                                             


<Press SPACE to continue>
        "#;

        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().borders(Borders::ALL);
            let paragraph = Paragraph::new(splash_text)
                .block(block.padding(Padding::new(
                    0, // left
                    0, // right
                    size.height / 4, // top
                    0, // bottom
                )))
                .style(Style::default().fg(Color::LightYellow).bg(Color::Black))
                .alignment(Alignment::Center);
            f.render_widget(paragraph, size);
        })?;

        // Wait for the space key press to continue
        loop {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char(' ') {
                    break;
                }
            }
        }

        // Clear the input buffer
        while event::poll(Duration::from_millis(0))? {
            event::read()?;
        }

        Ok(())
    }
}
