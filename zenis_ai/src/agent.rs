use chrono::Utc;
use zenis_database::agent_model::{AgentModel, AgentPricing};
use zenis_discord::{
    twilight_model::{
        id::{
            marker::{ChannelMarker, GuildMarker, UserMarker, WebhookMarker},
            Id,
        },
        user::User,
    },
    UserExtension,
};

use crate::brain::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CreditsPaymentMethod {
    UserCredits(Id<UserMarker>),
    GuildPublicCredits(Id<GuildMarker>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Agent<AgentBrain: Brain> {
    pub channel_id: Id<ChannelMarker>,
    pub webhook_id: Id<WebhookMarker>,
    pub webhook_token: String,

    pub agent_url_image: Option<String>,
    pub agent_name: String,
    pub agent_description: String,
    pub agent_pricing: AgentPricing,
    pub agent_payment_method: CreditsPaymentMethod,

    pub exit_reason: Option<String>,

    pub last_received_message_timestamp: i64,
    pub last_sent_message_timestamp: i64,

    pub participants: Vec<Id<UserMarker>>,
    pub brain: AgentBrain,

    pub message_history: Vec<AgentBrain::ChatMessage>,
    pub message_queue: Vec<(User, String)>,
    pub awaiting_message: bool,
}

impl<AgentBrain> Agent<AgentBrain>
where
    AgentBrain: Brain,
{
    pub fn new(
        channel_id: Id<ChannelMarker>,
        webhook: (String, Id<WebhookMarker>),
        agent_data: AgentModel,
        pricing: AgentPricing,
        payment_method: CreditsPaymentMethod,
        brain: AgentBrain,
    ) -> Self {
        let (webhook_token, webhook_id) = webhook;

        Self {
            channel_id,
            webhook_id,
            webhook_token,

            agent_name: agent_data.name.clone(),
            agent_description: agent_data.description.clone(),
            agent_url_image: agent_data.agent_url_image.clone(),
            agent_pricing: pricing,
            agent_payment_method: payment_method,

            brain,

            exit_reason: None,
            participants: vec![],

            last_received_message_timestamp: Utc::now().timestamp() + 5,
            last_sent_message_timestamp: 0,

            message_history: vec![],
            message_queue: vec![],
            awaiting_message: false,
        }
    }

    pub fn push_message(&mut self, message: AgentBrain::ChatMessage) {
        self.message_history.push(message);

        if self.message_history.len() > 15 {
            self.message_history.remove(0);
        }
    }

    pub async fn enqueue_message(&mut self, user: User, message: String) {
        self.message_queue.push((user, message));

        if self.message_queue.len() > 4 {
            self.message_queue.remove(0);
        }
    }

    pub async fn process_message_queue(&mut self) -> anyhow::Result<AgentBrain::ChatResponse> {
        let mut parameters = self.brain.default_parameters();
        parameters.system_message = self.agent_description.clone();

        let message = self.brain.make_user_message(
            self.message_queue
                .iter()
                .map(|(user, message)| {
                    format!("<{} (@{})>: {}", user.display_name(), user.name, message)
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );
        self.message_queue.clear();

        self.push_message(message.clone());

        let mut messages = self
            .brain
            .system_messages(self.agent_name.clone(), self.agent_description.clone());
        messages.extend_from_slice(&self.message_history);

        let response = self.brain.prompt_chat(parameters, messages).await?;

        let response_as_message = self.brain.convert_response_to_message(response.clone());
        self.push_message(response_as_message);

        Ok(response)
    }
}
