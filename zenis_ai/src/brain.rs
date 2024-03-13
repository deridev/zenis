use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::common::{ChatMessage, ChatResponse};

static DEFAULT_CLIENT: Lazy<Arc<reqwest::Client>> = Lazy::new(|| Arc::new(reqwest::Client::new()));

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BrainParameters {
    pub debug: bool,
    pub model: String,
    pub max_tokens: usize,
    pub system_prompt: String,
    pub strip_italic_actions: bool,
}

pub const DEFAULT_SYSTEM_PROMPT: &str = include_str!("default_system_prompt.txt");

#[async_trait]
pub trait Brain {
    fn api_key(&self, debug: bool) -> String;

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            debug: true,
            model: "unknown".to_string(),
            max_tokens: 1024,
            system_prompt: String::new(),
            strip_italic_actions: false,
        }
    }

    fn http_client(&self) -> Arc<reqwest::Client> {
        DEFAULT_CLIENT.clone()
    }

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse>;
}
