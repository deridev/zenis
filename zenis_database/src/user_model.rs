use std::{collections::HashSet, hash::Hash};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use zenis_discord::twilight_model::id::{marker::UserMarker, Id};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct UserSettings {
    pub is_notifications_enabled: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum UserFlags {
    AlreadyReceivedFreeGuildCredits,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: u64,
    pub credits: i64,
    pub settings: UserSettings,
    #[serde(default = "HashSet::new")]
    pub flags: HashSet<UserFlags>,
}

impl UserModel {
    pub fn new(user_id: Id<UserMarker>) -> Self {
        Self {
            id: ObjectId::new(),
            user_id: user_id.get(),
            credits: 0,
            settings: UserSettings {
                is_notifications_enabled: true,
            },
            flags: HashSet::new(),
        }
    }

    pub fn add_credits(&mut self, quantity: i64) {
        self.credits += quantity;
    }

    pub fn remove_credits(&mut self, quantity: i64) {
        self.credits = (self.credits - quantity).max(0);
    }

    pub fn insert_flag(&mut self, flag: UserFlags) {
        self.flags.insert(flag);
    }

    pub fn remove_flag(&mut self, flag: UserFlags) {
        self.flags.remove(&flag);
    }

    pub fn has_flag(&self, flag: UserFlags) -> bool {
        self.flags.contains(&flag)
    }
}
