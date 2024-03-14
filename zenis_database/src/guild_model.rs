use std::collections::HashSet;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use zenis_discord::twilight_model::id::{marker::GuildMarker, Id};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GuildFlag {
    AlreadyAknowledged,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GuildModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub guild_id: String,
    pub credits: i64,
    pub public_credits: i64,

    #[serde(default = "HashSet::new")]
    pub flags: HashSet<GuildFlag>,
}

impl GuildModel {
    pub fn new(guild_id: Id<GuildMarker>) -> Self {
        Self {
            id: ObjectId::new(),
            guild_id: guild_id.get().to_string(),
            credits: 0,
            public_credits: 0,
            flags: HashSet::new(),
        }
    }

    pub fn add_credits(&mut self, quantity: i64) {
        self.credits += quantity;
    }

    pub fn remove_credits(&mut self, quantity: i64) {
        self.credits = (self.credits - quantity).max(0);
    }

    pub fn add_public_credits(&mut self, quantity: i64) {
        self.public_credits += quantity;
    }

    pub fn remove_public_credits(&mut self, quantity: i64) {
        self.public_credits = (self.public_credits - quantity).max(0);
    }

    pub fn has_flag(&self, flag: GuildFlag) -> bool {
        self.flags.contains(&flag)
    }

    pub fn add_flag(&mut self, flag: GuildFlag) {
        self.flags.insert(flag);
    }

    pub fn remove_flag(&mut self, flag: GuildFlag) {
        self.flags.remove(&flag);
    }
}
