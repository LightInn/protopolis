// app.rs
use crate::message::Message;
use crate::simulation::Simulation;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::Rect;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use std::io;
use std::sync::mpsc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum AppState {
    Welcome,
    Configuration,
    Running,
    Paused,
    Exit,
}

#[derive(Debug)]
pub enum Action {
    StartSimulation,
    PauseSimulation,
    ResumeSimulation,
    SetTopic(String),
    SendMessage(String),
    Exit,
}

pub struct App {
    state: AppState,
    topic: String,
    messages: Vec<Message>,
    agents: Vec<String>,
    current_tab: usize,
    simulation: Option<Simulation>,
    action_tx: mpsc::Sender<Action>,
    action_rx: mpsc::Receiver<Action>,
}

impl Default for App {
    fn default() -> Self {
        let (action_tx, action_rx) = mpsc::channel();
        Self {
            state: AppState::Welcome,
            topic: String::new(),
            messages: Vec::new(),
            agents: Vec::new(),
            current_tab: 0,
            simulation: None,
            action_tx,
            action_rx,
        }
    }
}

impl App {
    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        while !matches!(self.state, AppState::Exit) {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.size());

        let tabs = Tabs::new(vec!["Messages", "Agents", "Configuration"])
            .block(Block::default().borders(Borders::ALL))
            .select(self.current_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(tabs, chunks[0]);

        match self.current_tab {
            0 => self.draw_messages_tab(frame, chunks[1]),
            1 => self.draw_agents_tab(frame, chunks[1]),
            2 => self.draw_configuration_tab(frame, chunks[1]),
            _ => {}
        }

        let status_bar = Paragraph::new(Line::from(vec![
            Span::raw("Status: "),
            Span::styled(
                match self.state {
                    AppState::Welcome => "Welcome",
                    AppState::Configuration => "Configuration",
                    AppState::Running => "Running",
                    AppState::Paused => "Paused",
                    _ => "Unknown",
                },
                Style::default().fg(Color::Green),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(status_bar, chunks[2]);
    }

    fn draw_messages_tab(&self, frame: &mut Frame, area: Rect) {
        let messages: Vec<Line> = self
            .messages
            .iter()
            .map(|msg| {
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", msg.sender),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::raw(msg.content.to_string()),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(messages)
            .block(Block::default().borders(Borders::ALL).title("Messages"));
        frame.render_widget(paragraph, area);
    }

    fn draw_agents_tab(&self, frame: &mut Frame, area: Rect) {
        let agents: Vec<Line> = self
            .agents
            .iter()
            .map(|agent| Line::from(vec![Span::raw(agent.clone())]))
            .collect();

        let paragraph =
            Paragraph::new(agents).block(Block::default().borders(Borders::ALL).title("Agents"));
        frame.render_widget(paragraph, area);
    }

    fn draw_configuration_tab(&self, frame: &mut Frame, area: Rect) {
        let topic_input = Paragraph::new(self.topic.as_str()) // Convertit explicitement en &str
            .block(Block::default().borders(Borders::ALL).title("Topic"));
        frame.render_widget(topic_input, area);
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        self.action_tx.send(Action::Exit).unwrap();
                        self.state = AppState::Exit;
                    }
                    KeyCode::Tab => {
                        self.current_tab = (self.current_tab + 1) % 3;
                    }
                    KeyCode::Char(' ') => match self.state {
                        AppState::Running => {
                            self.action_tx.send(Action::PauseSimulation).unwrap();
                            self.state = AppState::Paused;
                        }
                        AppState::Paused => {
                            self.action_tx.send(Action::ResumeSimulation).unwrap();
                            self.state = AppState::Running;
                        }
                        _ => {}
                    },
                    KeyCode::Enter => match self.state {
                        AppState::Welcome => {
                            self.state = AppState::Configuration;
                        }
                        AppState::Configuration => {
                            self.action_tx.send(Action::StartSimulation).unwrap();
                            self.state = AppState::Running;
                        }
                        _ => {}
                    },
                    KeyCode::Char(c) => {
                        if let AppState::Configuration = self.state {
                            self.topic.push(c);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
