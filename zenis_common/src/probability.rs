use std::fmt::Display;

use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, Default,
)]
pub struct Probability(u8);

impl Probability {
    pub const ALWAYS: Probability = Probability::new(100);
    pub const ALMOST_NEVER: Probability = Probability::new(1);
    pub const NEVER: Probability = Probability::new(0);

    pub const fn new(probability: u8) -> Self {
        if probability > 100 {
            return Self(100);
        }

        Self(probability)
    }

    pub const fn value(&self) -> u8 {
        self.0
    }

    pub fn add(&mut self, prob: u8) {
        self.0 = self.0.saturating_add(prob);
    }

    pub fn value_f64(&self) -> f64 {
        (self.0 as f64) / 100.0
    }

    pub fn generate_random_bool(&self) -> bool {
        let probability = self.value_f64();

        StdRng::from_os_rng().random_bool(probability.clamp(0.0, 1.0))
    }
}

impl Display for Probability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}
