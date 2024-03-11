use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentPricing {
    pub price_per_reply: i64,
}

impl Default for AgentPricing {
    fn default() -> Self {
        Self { price_per_reply: 5 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentModel {
    pub name: String,
    pub description: String,
    pub agent_url_image: Option<String>,
    pub pricing: AgentPricing,
}
