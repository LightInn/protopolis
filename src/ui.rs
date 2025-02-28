// ui.rs (correction)
use crate::message::Message;
use crate::simulation::{SimulationToUI, UIToSimulation};
use crate::state::AgentState;
use std::io::{self, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct UI {
    ui_tx: Sender<UIToSimulation>,
    ui_rx: Receiver<SimulationToUI>,
}

impl UI {
    pub fn new(ui_tx: Sender<UIToSimulation>, ui_rx: Receiver<SimulationToUI>) -> Self {
        Self { ui_tx, ui_rx }
    }

    pub fn run(&mut self) {
        println!("=== Simulation d'Agents avec Communication Ollama ===");
        println!("Commandes disponibles:");
        println!("  start - Démarrer la simulation");
        println!("  pause - Mettre en pause la simulation");
        println!("  resume - Reprendre la simulation");
        println!("  stop - Arrêter la simulation");
        println!("  topic <sujet> - Définir un sujet de discussion");
        println!("  exit - Quitter l'application");

        // Créer un thread séparé pour gérer les mises à jour de la simulation
        let tx = self.ui_tx.clone();
        let mut ui_rx = std::sync::mpsc::channel().1;
        std::mem::swap(&mut self.ui_rx, &mut ui_rx);

        thread::spawn(move || {
            while let Ok(update) = ui_rx.recv() {
                match update {
                    SimulationToUI::TickUpdate(tick) => {
                        if tick % 10 == 0 {
                            println!("Tick: {}", tick);
                        }
                    }
                    SimulationToUI::AgentUpdate(name, state, energy) => {
                        println!("Agent {} est maintenant {:?} (énergie: {:.1})", name, state, energy);
                    }
                    SimulationToUI::MessageUpdate(message) => {
                        println!("\n[MESSAGE] De {} à {}: {}",
                                 message.sender,
                                 message.recipient,
                                 message.content.to_string().trim_matches('"')
                        );
                    }
                    SimulationToUI::StateUpdate(state) => {
                        println!("[SYSTÈME] {}", state);
                    }
                }
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
                            println!("Sujet de discussion défini: {}", topic);
                        }
                        _ => {
                            println!("Commande non reconnue. Essayez 'start', 'pause', 'resume', 'stop', 'topic <sujet>' ou 'exit'.");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Erreur de lecture: {}", e);
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
