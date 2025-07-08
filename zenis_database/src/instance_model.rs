use bson::oid::ObjectId;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::agent_model::{AgentModel, AgentPricing};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InstanceBrain {
    GeminiFlash,
    GeminiPro,
    ClaudeHaiku,
    ZenisFinetuned,
}

impl InstanceBrain {
    pub fn extra_price_per_reply(&self) -> i64 {
        match self {
            Self::GeminiFlash => 0,
            Self::GeminiPro => 2,
            Self::ClaudeHaiku => 2,
            Self::ZenisFinetuned => 2,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::GeminiFlash => "Gemini 2.5 Flash",
            Self::GeminiPro => "Gemini 2.5 Pro",
            Self::ClaudeHaiku => "Haiku",
            Self::ZenisFinetuned => "ZenisLLM",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CreditsPaymentMethod {
    UserCredits(u64),
    GuildPublicCredits(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstanceMessage {
    pub user_id: u64,
    pub is_assistant: bool,
    pub text: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub summoner_id: u64,
    pub channel_id: u64,
    pub agent_identifier: String,
    pub agent_name: String,
    pub agent_description: String,
    pub system_prompt: String,
    pub pricing: AgentPricing,
    pub brain: InstanceBrain,

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
    #[serde(default = "Default::default")]
    pub error_counter: u32,
}

impl InstanceModel {
    pub fn new(
        agent_brain: InstanceBrain,
        (channel_id, summoner_id): (u64, u64),
        agent_model: AgentModel,
        pricing: AgentPricing,
        (webhook_id, webhook_token): (u64, String),
        payment_method: CreditsPaymentMethod,
        system_prompt: String,
    ) -> Self {
        Self {
            id: ObjectId::new(),
            channel_id,
            summoner_id,
            pricing,
            brain: agent_brain,
            agent_identifier: agent_model.identifier.clone(),
            agent_name: agent_model.name.clone(),
            agent_description: agent_model.description.clone(),
            system_prompt,

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
            error_counter: 0,
        }
    }

    pub fn push_message(&mut self, message: InstanceMessage) {
        let instance_message: InstanceMessage = message.into();

        if let Some(last_message) = self.history.last_mut() {
            if !last_message.is_assistant && !instance_message.is_assistant {
                if last_message.text.chars().count() > 2000 {
                    last_message.text = last_message
                        .text
                        .chars()
                        .rev()
                        .take(2000)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                }

                last_message
                    .text
                    .push_str(&format!("\n{}", instance_message.text));
            } else {
                self.history.push(instance_message);
            }
        } else {
            self.history.push(instance_message);
        }

        if self.history.len() > 20 {
            self.history.remove(0);
        }

        self.last_received_message_timestamp = Utc::now().timestamp();
    }

    pub fn increment_error(&mut self) {
        self.error_counter += 1;
        if self.error_counter > 10 {
            self.exit_reason =
                Some("Erro interno. Desenvolvedor foi contactado sobre o problema.".to_string());
        }
    }

    pub fn introduce(&mut self, introduction_message: impl ToString) -> InstanceMessage {
        let first_message = self.history.first();
        if first_message.is_none() || first_message.is_some_and(|m| m.is_assistant) {
            self.push_message(InstanceMessage {
                is_assistant: false,
                user_id: 0,
                text: format!("<!system_instruction/>Se apresente, {}.", self.agent_name),
                image_url: None,
            });
        }

        let introduction_message = InstanceMessage {
            is_assistant: true,
            user_id: self.webhook_id,
            text: format!("<!message/>{}", introduction_message.to_string()),
            image_url: None,
        };

        self.push_message(introduction_message.clone());

        introduction_message
    }
}
