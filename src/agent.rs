// agent.rs
use crate::action::{Action, ActionHandler};
use crate::config::AgentConfig;
use crate::message::Message;
use crate::personality::Personality;
use crate::state::AgentState;
use chrono::Utc;
use serde_json::Value;
use std::collections::VecDeque;

pub struct Agent {
    pub name: String,
    pub personality: Personality,
    state: AgentState,
    action_handler: ActionHandler,
    messages: VecDeque<Message>,
    energy: f32,
    position: (i32, i32),
    debug: bool,
}

impl Agent {
    pub fn new(config: &AgentConfig, debug: bool) -> Self {
        Self {
            name: config.name.clone(),
            personality: crate::personality::get_personality_template(&config.personality_template),
            state: AgentState::Idle,
            action_handler: ActionHandler::new(),
            messages: VecDeque::new(),
            energy: config.initial_energy,
            position: config.initial_position,
            debug,
        }
    }

    pub fn update(&mut self, current_tick: u64) {
        // Consommer de l'énergie
        self.energy -= 0.1;

        // Décider de la prochaine action
        let action = self.decide_next_action();

        // Exécuter l'action
        if let Ok(result) = self.action_handler.execute(&action) {
            self.state = result.new_state;
            self.energy += result.energy_delta;

            if self.debug && result.message.is_some() {
                println!("{}: {}", self.name, result.message.unwrap());
            }
        }

        // Générer un message (simulation simplifiée)
        if current_tick % 10 == 0 {
            self.messages.push_back(Message {
                sender: self.name.clone(),
                recipient: "everyone".to_string(),
                content: Value::String(format!("Tick {}: I'm currently {:?}", current_tick, self.state)),
                timestamp: Utc::now(),
            });
        }
    }

    fn decide_next_action(&self) -> Action {
        // Logique simplifiée pour décider de l'action
        if self.energy < 30.0 {
            Action::Sleep { duration: 10 }
        } else {
            let actions = vec![
                Action::Think { topic: "next move".to_string() },
                Action::Listen { duration: 5 },
                Action::Speak { message: "Hello!".to_string(), target: None },
            ];

            let chosen_index = self.personality.influence_decision(
                &actions.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>()
            );

            actions[chosen_index].clone()
        }
    }

    pub fn get_messages(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();
        while let Some(msg) = self.messages.pop_front() {
            messages.push(msg);
        }
        messages
    }

    pub fn get_state(&self) -> &AgentState {
        &self.state
    }

    pub fn get_energy(&self) -> f32 {
        self.energy
    }

    pub fn get_position(&self) -> (i32, i32) {
        self.position
    }
}
