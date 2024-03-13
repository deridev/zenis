use std::hash::Hash;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use zenis_discord::twilight_model::id::{marker::UserMarker, Id};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct UserSettings {
    pub is_notifications_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: u64,
    pub credits: i64,
    pub settings: UserSettings,
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
        }
    }

    pub fn add_credits(&mut self, quantity: i64) {
        self.credits += quantity;
    }

    pub fn remove_credits(&mut self, quantity: i64) {
        self.credits = (self.credits - quantity).max(0);
    }
}
