use std::collections::HashSet;

use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AgentPricing {
    pub price_per_reply: i64,
    pub price_per_invocation: i64,
}

impl Default for AgentPricing {
    fn default() -> Self {
        Self {
            price_per_reply: 5,
            price_per_invocation: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStats {
    pub invocations: u64,
    pub replies: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub creator_user_id: u64,
    pub guild_id: Option<u64>,
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub introduction_message: String,
    pub agent_url_image: Option<String>,
    pub pricing: AgentPricing,

    pub public: bool,
    pub is_waiting_for_approval: bool,
    pub tags: HashSet<String>,

    pub stats: AgentStats,
}

impl AgentModel {
    pub fn new(
        creator_user_id: u64,
        identifier: impl ToString,
        name: impl ToString,
        description: impl ToString,
        introduction_message: impl ToString,
        pricing: AgentPricing,
    ) -> Self {
        Self {
            id: ObjectId::new(),
            creator_user_id,
            guild_id: None,
            identifier: identifier.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            introduction_message: introduction_message.to_string(),
            agent_url_image: None,
            pricing,

            public: false,
            is_waiting_for_approval: false,
            tags: HashSet::new(),

            stats: AgentStats {
                invocations: 0,
                replies: 0,
            },
        }
    }

    pub fn with_url_image(mut self, url_image: impl ToString) -> Self {
        self.agent_url_image = Some(url_image.to_string());
        self
    }

    pub fn with_guild_id(mut self, guild_id: u64) -> Self {
        self.guild_id = Some(guild_id);
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl ToString>) -> Self {
        self.tags = tags.into_iter().map(|t| t.to_string()).collect();
        self
    }
}
