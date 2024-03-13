use bson::oid::ObjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::agent_model::{AgentModel, AgentPricing};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CreditsPaymentMethod {
    UserCredits(u64),
    GuildPublicCredits(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstanceMessage {
    pub is_user: bool,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub channel_id: u64,
    pub agent_identifier: String,
    pub agent_name: String,
    pub agent_description: String,
    pub pricing: AgentPricing,

    pub webhook_id: u64,
    pub webhook_token: String,

    pub payment_method: CreditsPaymentMethod,
    pub exit_reason: Option<String>,
    pub active: bool,
    pub history: Vec<InstanceMessage>,

    pub last_sent_message_timestamp: i64,
    pub last_received_message_timestamp: i64,

    pub already_introduced: bool,
    pub is_awaiting_new_messages: bool,
}

impl InstanceModel {
    pub fn new(
        channel_id: u64,
        agent_model: AgentModel,
        pricing: AgentPricing,
        (webhook_id, webhook_token): (u64, String),
        payment_method: CreditsPaymentMethod,
    ) -> Self {
        Self {
            id: ObjectId::new(),
            channel_id,
            pricing,
            agent_identifier: agent_model.identifier.clone(),
            agent_name: agent_model.name.clone(),
            agent_description: agent_model.description.clone(),

            webhook_id,
            webhook_token,

            payment_method,
            exit_reason: None,
            active: true,
            history: vec![],

            last_sent_message_timestamp: 0,
            last_received_message_timestamp: Utc::now().timestamp(),

            already_introduced: false,
            is_awaiting_new_messages: true,
        }
    }

    pub fn push_message(&mut self, message: impl Into<InstanceMessage>) {
        let instance_message: InstanceMessage = message.into();

        if let Some(last_message) = self.history.last_mut() {
            if last_message.is_user {
                last_message
                    .content
                    .push_str(&format!("\n{}", instance_message.content));
            }
        } else {
            self.history.push(instance_message);
        }

        if self.history.len() > 15 {
            self.history.remove(0);
        }
    }

    pub fn introduce(&mut self, introduction_message: impl ToString) -> InstanceMessage {
        let first_message = self.history.first();
        if first_message.is_none() || first_message.is_some_and(|m| !m.is_user) {
            self.push_message(InstanceMessage {
                is_user: true,
                content: "<Se apresente>".to_string(),
            });
        }

        let introduction_message = InstanceMessage {
            is_user: false,
            content: introduction_message.to_string(),
        };

        self.push_message(introduction_message.clone());

        InstanceMessage {
            is_user: false,
            content: introduction_message.content,
        }
    }
}
