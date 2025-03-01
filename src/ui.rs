// ui.rs
use crate::message::Message;
use crate::simulation::{SimulationToUI, UIToSimulation};
use crate::state::AgentState;
use colored::*;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

const COLORS: [&str; 8] = [
    "red", "green", "yellow", "blue", "magenta", "cyan", "bright_red", "bright_green"
];

pub struct UI {
    ui_tx: Sender<UIToSimulation>,
    ui_rx: Receiver<SimulationToUI>,
    agent_colors: HashMap<String, String>,
}

impl UI {
    pub fn new(ui_tx: Sender<UIToSimulation>, ui_rx: Receiver<SimulationToUI>) -> Self {
        Self {
            ui_tx,
            ui_rx,
            agent_colors: HashMap::new()
        }
    }

    fn clear_screen() {
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }

    fn move_cursor_to_bottom(height: u16) {
        print!("\x1B[{};0H", height);
        io::stdout().flush().unwrap();
    }

    fn get_terminal_height() -> u16 {
        #[cfg(unix)]
        {
            use libc::{ioctl, winsize, TIOCGWINSZ};
            use std::os::unix::io::AsRawFd;

            let mut ws = winsize {
                ws_row: 0,
                ws_col: 0,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };

            if unsafe { ioctl(io::stdout().as_raw_fd(), TIOCGWINSZ, &mut ws) } == 0 {
                return ws.ws_row;
            }
        }

        // Valeur par défaut si on ne peut pas déterminer la taille
        24
    }

    fn get_agent_color(&mut self, agent_name: &str) -> ColoredString {
        if !self.agent_colors.contains_key(agent_name) {
            let color_index = self.agent_colors.len() % COLORS.len();
            self.agent_colors.insert(agent_name.to_string(), COLORS[color_index].to_string());
        }

        match self.agent_colors.get(agent_name).unwrap().as_str() {
            "red" => agent_name.red(),
            "green" => agent_name.green(),
            "yellow" => agent_name.yellow(),
            "blue" => agent_name.blue(),
            "magenta" => agent_name.magenta(),
            "cyan" => agent_name.cyan(),
            "bright_red" => agent_name.bright_red(),
            "bright_green" => agent_name.bright_green(),
            _ => agent_name.normal(),
        }
    }

    fn print_message(&mut self, message: &Message) {
        let sender_colored = match message.sender.as_str() {
            "Utilisateur" => "Utilisateur".bright_white().bold(),
            "Système" => "Système".bright_white().on_blue(),
            _ => self.get_agent_color(&message.sender)
        };

        let recipient_colored = match message.recipient.as_str() {
            "Utilisateur" => "Utilisateur".bright_white().bold(),
            "Système" => "Système".bright_white().on_blue(),
            "everyone" => "tout le monde".italic(),
            _ => self.get_agent_color(&message.recipient)
        };

        println!("\n[MESSAGE] De {} à {}: {}",
                 sender_colored,
                 recipient_colored,
                 message.content.to_string().trim_matches('"')
        );
    }

    pub fn run(&mut self) {
        Self::clear_screen();

        println!("\n\n");
        println!("{}", "██████╗ ██████╗  ██████╗ ████████╗ ██████╗ ██████╗  ██████╗ ██╗     ██╗███████╗".bright_cyan());
        println!("{}", "██╔══██╗██╔══██╗██╔═══██╗╚══██╔══╝██╔═══██╗██╔══██╗██╔═══██╗██║     ██║██╔════╝".bright_cyan());
        println!("{}", "██████╔╝██████╔╝██║   ██║   ██║   ██║   ██║██████╔╝██║   ██║██║     ██║███████╗".bright_cyan());
        println!("{}", "██╔═══╝ ██╔══██╗██║   ██║   ██║   ██║   ██║██╔═══╝ ██║   ██║██║     ██║╚════██║".bright_cyan());
        println!("{}", "██║     ██║  ██║╚██████╔╝   ██║   ╚██████╔╝██║     ╚██████╔╝███████╗██║███████║".bright_cyan());
        println!("{}", "╚═╝     ╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝ ╚═╝      ╚═════╝ ╚══════╝╚═╝╚══════╝".bright_cyan());
        println!("\n\n");
        println!("=== Simulation d'Agents avec Communication Ollama ===");
        println!("Commandes disponibles:");
        println!("  start - Démarrer la simulation");
        println!("  pause - Mettre en pause la simulation");
        println!("  resume - Reprendre la simulation");
        println!("  stop - Arrêter la simulation");
        println!("  topic <sujet> - Définir un sujet de discussion");
        println!("  {} <agent> <message> - Envoyer un message à un agent", "msg".green());
        println!("  exit - Quitter l'application");
        println!("\n");

        // Créer un thread séparé pour gérer les mises à jour de la simulation
        let tx = self.ui_tx.clone();
        let mut ui_rx = std::sync::mpsc::channel().1;
        std::mem::swap(&mut self.ui_rx, &mut ui_rx);
        let agent_colors_clone = self.agent_colors.clone();

        let terminal_height = Self::get_terminal_height();

        thread::spawn(move || {
            let mut local_agent_colors = agent_colors_clone;

            while let Ok(update) = ui_rx.recv() {
                // Effacer la ligne du prompt
                print!("\r\x1B[K");

                match update {
                    SimulationToUI::TickUpdate(tick) => {
                        if tick % 10 == 0 {
                            println!("Tick: {}", tick.to_string().yellow());
                        }
                    }
                    SimulationToUI::AgentUpdate(name, state, energy) => {
                        // Déterminer la couleur de l'agent
                        let color_index = local_agent_colors.len() % COLORS.len();
                        if !local_agent_colors.contains_key(&name) {
                            local_agent_colors.insert(name.clone(), COLORS[color_index].to_string());
                        }

                        let agent_name = match local_agent_colors.get(&name).unwrap().as_str() {
                            "red" => name.red(),
                            "green" => name.green(),
                            "yellow" => name.yellow(),
                            "blue" => name.blue(),
                            "magenta" => name.magenta(),
                            "cyan" => name.cyan(),
                            "bright_red" => name.bright_red(),
                            "bright_green" => name.bright_green(),
                            _ => name.normal(),
                        };

                        let state_str = match state {
                            AgentState::Idle => state.to_string().bright_black(),
                            AgentState::Thinking => state.to_string().yellow(),
                            AgentState::Speaking => state.to_string().bright_green(),
                            _ => state.to_string().normal()
                        };

                        let energy_color = if energy < 30.0 {
                            energy.to_string().red()
                        } else if energy < 70.0 {
                            energy.to_string().yellow()
                        } else {
                            energy.to_string().green()
                        };

                        println!("Agent {} est maintenant {} (énergie: {})",
                                 agent_name, state_str, energy_color);
                    }
                    SimulationToUI::MessageUpdate(message) => {
                        // Déterminer les couleurs
                        let sender_color = match message.sender.as_str() {
                            "Utilisateur" => "Utilisateur".bright_white().bold(),
                            "Système" => "Système".bright_white().on_blue(),
                            _ => {
                                let color_index = local_agent_colors.len() % COLORS.len();
                                if !local_agent_colors.contains_key(&message.sender) {
                                    local_agent_colors.insert(message.sender.clone(), COLORS[color_index].to_string());
                                }

                                match local_agent_colors.get(&message.sender).unwrap().as_str() {
                                    "red" => message.sender.red(),
                                    "green" => message.sender.green(),
                                    "yellow" => message.sender.yellow(),
                                    "blue" => message.sender.blue(),
                                    "magenta" => message.sender.magenta(),
                                    "cyan" => message.sender.cyan(),
                                    "bright_red" => message.sender.bright_red(),
                                    "bright_green" => message.sender.bright_green(),
                                    _ => message.sender.normal(),
                                }
                            }
                        };

                        let recipient_color = match message.recipient.as_str() {
                            "Utilisateur" => "Utilisateur".bright_white().bold(),
                            "Système" => "Système".bright_white().on_blue(),
                            "everyone" => "tout le monde".italic(),
                            _ => {
                                let color_index = local_agent_colors.len() % COLORS.len();
                                if !local_agent_colors.contains_key(&message.recipient) {
                                    local_agent_colors.insert(message.recipient.clone(), COLORS[color_index].to_string());
                                }

                                match local_agent_colors.get(&message.recipient).unwrap().as_str() {
                                    "red" => message.recipient.red(),
                                    "green" => message.recipient.green(),
                                    "yellow" => message.recipient.yellow(),
                                    "blue" => message.recipient.blue(),
                                    "magenta" => message.recipient.magenta(),
                                    "cyan" => message.recipient.cyan(),
                                    "bright_red" => message.recipient.bright_red(),
                                    "bright_green" => message.recipient.bright_green(),
                                    _ => message.recipient.normal(),
                                }
                            }
                        };

                        println!("\n[MESSAGE] De {} à {}: {}",
                                 sender_color,
                                 recipient_color,
                                 message.content.to_string().trim_matches('"')
                        );
                    }
                    SimulationToUI::StateUpdate(state) => {
                        println!("[{}] {}", "SYSTÈME".bright_blue(), state);
                    }
                }

                // Réécrire le prompt
                print!("> ");
                io::stdout().flush().unwrap();
            }
        });

        // Boucle principale pour les entrées utilisateur
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();

                    match input {
                        "start" => {
                            let _ = self.ui_tx.send(UIToSimulation::Start);
                            println!("Démarrage de la simulation...");
                        }
                        "pause" => {
                            let _ = self.ui_tx.send(UIToSimulation::Pause);
                            println!("Mise en pause de la simulation...");
                        }
                        "resume" => {
                            let _ = self.ui_tx.send(UIToSimulation::Resume);
                            println!("Reprise de la simulation...");
                        }
                        "stop" => {
                            let _ = self.ui_tx.send(UIToSimulation::Stop);
                            println!("Arrêt de la simulation...");
                        }
                        "exit" => {
                            let _ = self.ui_tx.send(UIToSimulation::Stop);
                            println!("Fermeture de l'application...");
                            break;
                        }
                        _ if input.starts_with("topic ") => {
                            let topic = input.trim_start_matches("topic ").to_string();
                            let _ = self.ui_tx.send(UIToSimulation::SetDiscussionTopic(topic.clone()));
                            println!("Sujet de discussion défini: {}", topic.green());
                        }
                        // Nouvelle commande pour envoyer un message à un agent
                        _ if input.starts_with("msg ") => {
                            let parts: Vec<&str> = input.splitn(3, ' ').collect();
                            if parts.len() == 3 {
                                let agent_name = parts[1];
                                let message = parts[2];
                                let _ = self.ui_tx.send(UIToSimulation::UserMessage(
                                    agent_name.to_string(),
                                    message.to_string()
                                ));
                                println!("Message envoyé à {}: {}",
                                         self.get_agent_color(agent_name),
                                         message.bright_white());
                            } else {
                                println!("{}", "Format incorrect. Utilisez: msg <agent> <message>".red());
                            }
                        }
                        _ => {
                            println!("{}", "Commande non reconnue. Essayez 'start', 'pause', 'resume', 'stop', 'topic <sujet>', 'msg <agent> <message>' ou 'exit'.".red());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("Erreur de lecture: {}", e).red());
                    break;
                }
            }
        }
    }
}



// Extension pour vérifier si des données sont disponibles sur stdin
trait DataReady {
    fn data_ready(&self) -> io::Result<bool>;
}

impl DataReady for io::Stdin {
    #[cfg(unix)]
    fn data_ready(&self) -> io::Result<bool> {
        use std::os::unix::io::AsRawFd;
        use std::io::Error;
        use libc::{fd_set, select, timeval, FD_ISSET, FD_SET, FD_ZERO};

        let fd = io::stdin().as_raw_fd();
        let mut read_fds = unsafe { std::mem::zeroed::<fd_set>() };
        unsafe { FD_ZERO(&mut read_fds); }
        unsafe { FD_SET(fd, &mut read_fds); }

        let mut timeout = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };

        let result = unsafe { select(fd + 1, &mut read_fds, std::ptr::null_mut(), std::ptr::null_mut(), &mut timeout) };

        if result < 0 {
            Err(Error::last_os_error())
        } else {
            Ok(unsafe { FD_ISSET(fd, &read_fds) })
        }
    }

    #[cfg(windows)]
    fn data_ready(&self) -> io::Result<bool> {
        // Sur Windows, on ne peut pas facilement vérifier si stdin est prêt
        // On retourne simplement false pour que le programme continue
        Ok(false)
    }

    #[cfg(not(any(unix, windows)))]
    fn data_ready(&self) -> io::Result<bool> {
        Ok(false)
    }
}