use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static DEFAULT_CLIENT: Lazy<Arc<reqwest::Client>> = Lazy::new(|| Arc::new(reqwest::Client::new()));

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BrainParameters {
    pub model: String,
    pub max_tokens: usize,
    pub system_message: String,
    pub strip_italic_actions: bool,
}

pub const DEFAULT_SYSTEM_PROMPT: &str = include_str!("default_system_prompt.txt");

#[async_trait]
pub trait Brain {
    type ChatMessage: Clone + Debug;
    type ChatResponse: Clone + Debug;

    fn api_key(&self) -> String;

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            model: "unknown".to_string(),
            max_tokens: 1024,
            system_message: String::new(),
            strip_italic_actions: false,
        }
    }

    fn http_client(&self) -> Arc<reqwest::Client> {
        DEFAULT_CLIENT.clone()
    }

    fn system_messages(&self, bot_name: String, message: String) -> Vec<Self::ChatMessage>;
    fn make_user_message(&self, message_content: String) -> Self::ChatMessage;

    fn convert_response_to_message(&self, response: Self::ChatResponse) -> Self::ChatMessage;

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        messages: Vec<Self::ChatMessage>,
    ) -> anyhow::Result<Self::ChatResponse>;
}
