// personality.rs

use rand::Rng;

/// Represents an agent's personality using the Big Five personality traits.
#[derive(Debug, Clone)]
pub struct Personality {
    /// Openness to experience (curiosity, creativity).
    pub openness: f32,

    /// Conscientiousness (organization, responsibility).
    pub conscientiousness: f32,

    /// Extraversion (sociability, energy levels).
    pub extraversion: f32,

    /// Agreeableness (cooperation, trust).
    pub agreeableness: f32,

    /// Neuroticism (emotional stability, anxiety).
    pub neuroticism: f32,
}

impl Personality {
    /// Creates a new personality with specified trait values.
    ///
    /// # Arguments
    /// * `openness` - Degree of openness to experience.
    /// * `conscientiousness` - Level of organization and responsibility.
    /// * `extraversion` - Level of sociability and energy.
    /// * `agreeableness` - Degree of cooperativeness and kindness.
    /// * `neuroticism` - Degree of emotional instability.
    ///
    /// # Returns
    /// * A `Personality` instance with the given trait values.
    pub fn new(
        openness: f32,
        conscientiousness: f32,
        extraversion: f32,
        agreeableness: f32,
        neuroticism: f32,
    ) -> Self {
        Self {
            openness,
            conscientiousness,
            extraversion,
            agreeableness,
            neuroticism,
        }
    }
}

/// Generates a personality based on a predefined template.
///
/// # Arguments
/// * `template` - The name of the personality template (e.g., "friendly").
///
/// # Returns
/// * A `Personality` instance corresponding to the template.
pub fn get_personality_template(template: &str) -> Personality {
    match template {
        "friendly" => Personality::new(0.6, 0.7, 0.8, 0.9, 0.3),
        "curious" => Personality::new(0.9, 0.5, 0.6, 0.7, 0.4),
        "cautious" => Personality::new(0.4, 0.8, 0.3, 0.6, 0.7),
        _ => Personality::new(0.5, 0.5, 0.5, 0.5, 0.5), // Default balanced personality.
    }
}
