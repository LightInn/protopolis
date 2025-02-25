// Créer un nouveau module logging.rs
use chrono::Local;
use colored::*;
use std::sync::Once;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use crate::simulation::SimulationEvent;

static INIT: Once = Once::new();

#[derive(Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

pub struct Logger {
    sender: mpsc::Sender<(LogLevel, String)>,
    ui_sender: Option<mpsc::Sender<SimulationEvent>>,
}

impl Logger {
    pub fn new(sender: mpsc::Sender<(LogLevel, String)>, ui_sender: Option<mpsc::Sender<SimulationEvent>>) -> Self {
        Self { sender, ui_sender }
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        let timestamp = Local::now().format("%H:%M:%S").to_string();
        let formatted = format!("[{}] {}", timestamp, message);

        // Envoyer au logger
        if let Err(e) = self.sender.blocking_send((level.clone(), formatted.clone())) {
            eprintln!("Erreur logging: {}", e);
        }

        // Envoyer à l'UI si configuré
        if let Some(sender) = &self.ui_sender {
            if let Err(e) = sender.blocking_send(SimulationEvent::Message(formatted)) {
                eprintln!("Erreur UI logging: {}", e);
            }
        }
    }
}

// Créer une instance globale
lazy_static! {
    static ref LOGGER: Logger = {
        let (tx, mut rx) = mpsc::channel(100);
        // Spawn un task pour gérer les logs
        tokio::spawn(async move {
            while let Some((level, msg)) = rx.recv().await {
                match level {
                    LogLevel::Debug => println!("{} {}", "DEBUG".blue(), msg),
                    LogLevel::Info => println!("{} {}", "INFO".green(), msg),
                    LogLevel::Warning => println!("{} {}", "WARN".yellow(), msg),
                    LogLevel::Error => println!("{} {}", "ERROR".red(), msg),
                }
            }
        });
        Logger::new(tx, None)
    };
}
