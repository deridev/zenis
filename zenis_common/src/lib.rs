mod cache;
mod color;
pub mod config;
mod identifiable;
mod image;
mod pagination;
mod probability;

pub use cache::Cache;
pub use color::Color;
pub use identifiable::Identifiable;
pub use image::*;
pub use pagination::Pagination;
pub use probability::Probability;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
    Default,
)]
pub struct Attribute {
    pub value: i32,
    pub max: i32,
}

impl From<i32> for Attribute {
    fn from(value: i32) -> Self {
        Self { value, max: value }
    }
}

impl Attribute {
    pub fn new(value: i32, max: i32) -> Attribute {
        Attribute { value, max }
    }

    pub fn add(&mut self, value: i32) {
        self.value = (self.value + value).min(self.max);
    }

    pub fn remove(&mut self, value: i32) {
        self.value = (self.value - value).max(0);
    }

    pub fn reduce_by(&mut self, amount: f32) {
        let subtracted_amount = self.max - (self.value as f32 * amount) as i32;
        self.value -= subtracted_amount;
        self.max -= subtracted_amount;
    }
}

pub fn clear_string(s: &str) -> String {
    unidecode::unidecode(&s.to_lowercase())
}

pub trait ReadableNumber {
    fn to_readable_string(self) -> String;
}

impl<T: std::fmt::Display + Copy + PartialOrd + std::ops::Add> ReadableNumber for T {
    fn to_readable_string(self) -> String {
        let mut input = self.to_string();
        if input.is_empty() {
            return input;
        }

        let mut output = String::with_capacity(input.len() + 5);
        let mut negative = false;
        if input.starts_with('-') {
            input.remove(0);
            negative = true;
        }

        for (i, c) in input.chars().rev().enumerate() {
            if i % 3 == 0 && i != 0 {
                output.insert(0, ',');
            }

            output.insert(0, c);
        }

        if negative {
            output.insert(0, '-');
        }

        output.replace(",.", ".")
    }
}

pub fn calculate_power_level(
    vitality: Attribute,
    resistance: Attribute,
    ether: Attribute,
    strength_level: u32,
    intelligence_level: u32,
    weighted_skills: f64,
) -> i64 {
    let health = (resistance.value + vitality.value) / 2;
    let health_pl = (health as f64).powf(0.45).round();
    let ether_pl = (ether.value as f64).powf(0.45).round();
    let strength_pl = ((strength_level + intelligence_level) as f64 / 2.0)
        .powf(0.65)
        .round();

    let skill_multiplier = weighted_skills / 3.0;
    let skill_multiplier = 1.0 + skill_multiplier;

    ((health_pl + ether_pl + strength_pl) * skill_multiplier) as i64
}
