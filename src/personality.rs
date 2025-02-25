// personality.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defines personality traits and their values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// Map of trait names to their values (-1.0 to 1.0)
    traits: HashMap<String, f32>,
    /// Brief description of the personality
    description: String,
}

impl Personality {
    /// Creates a new personality with default traits
    pub fn new(description: &str) -> Self {
        let mut traits = HashMap::new();
        traits.insert("openness".to_string(), 0.0);
        traits.insert("conscientiousness".to_string(), 0.0);
        traits.insert("extraversion".to_string(), 0.0);
        traits.insert("agreeableness".to_string(), 0.0);
        traits.insert("neuroticism".to_string(), 0.0);

        Self {
            traits,
            description: description.to_string(),
        }
    }

    /// Sets a trait value, clamping it between -1.0 and 1.0
    pub fn set_trait(&mut self, trait_name: &str, value: f32) {
        self.traits
            .insert(trait_name.to_string(), value.clamp(-1.0, 1.0));
    }

    /// Gets a trait value
    pub fn get_trait(&self, trait_name: &str) -> Option<f32> {
        self.traits.get(trait_name).cloned()
    }

    /// Returns the personality description
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Influences decision making based on personality
    pub fn influence_decision(&self, options: &[String]) -> usize {
        // Simple example of personality affecting decisions
        let extraversion = self.get_trait("extraversion").unwrap_or(0.0);
        let openness = self.get_trait("openness").unwrap_or(0.0);

        // More extraverted agents prefer social actions
        // More open agents prefer novel actions
        // This is a simple example and could be made more sophisticated
        let mut scores: Vec<f32> = vec![0.0; options.len()];

        for (i, option) in options.iter().enumerate() {
            if option.contains("speak") || option.contains("listen") {
                scores[i] += extraversion;
            }
            if option.contains("think") || option.contains("explore") {
                scores[i] += openness;
            }
        }

        // Return index of highest scoring option
        scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(index, _)| index)
            .unwrap_or(0)
    }
}

/// Predefined personality templates
pub fn get_personality_template(template_name: &str) -> Personality {
    match template_name {
        "optimistic" => {
            let mut personality = Personality::new("An optimistic and social personality");
            personality.set_trait("extraversion", 0.8);
            personality.set_trait("neuroticism", -0.6);
            personality.set_trait("openness", 0.6);
            personality
        }
        "analytical" => {
            let mut personality = Personality::new("A logical and thoughtful personality");
            personality.set_trait("conscientiousness", 0.7);
            personality.set_trait("openness", 0.5);
            personality.set_trait("extraversion", -0.3);
            personality
        }
        _ => Personality::new("Default balanced personality"),
    }
}
