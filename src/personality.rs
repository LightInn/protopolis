// personality.rs
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Personality {
    pub openness: f32,
    pub conscientiousness: f32,
    pub extraversion: f32,
    pub agreeableness: f32,
    pub neuroticism: f32,
}

impl Personality {
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

    pub fn influence_decision(&self, options: &[String]) -> usize {
        // Implémentation simplifiée: choix aléatoire influencé par la personnalité
        let mut rng = rand::thread_rng();
        rng.gen_range(0..options.len())
    }
}

pub fn get_personality_template(template: &str) -> Personality {
    match template {
        "friendly" => Personality::new(0.6, 0.7, 0.8, 0.9, 0.3),
        "curious" => Personality::new(0.9, 0.5, 0.6, 0.7, 0.4),
        "cautious" => Personality::new(0.4, 0.8, 0.3, 0.6, 0.7),
        _ => Personality::new(0.5, 0.5, 0.5, 0.5, 0.5), // Personnalité équilibrée par défaut
    }
}
