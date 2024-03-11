use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    brain::{Brain, BrainParameters, DEFAULT_SYSTEM_PROMPT},
    util::remove_italic_actions,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeChatResponse {
    pub content: Vec<ClaudeContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeContent {
    pub text: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub system: String,
    pub max_tokens: usize,
    pub messages: Vec<ClaudeChatMessage>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClaudeBrain;

#[async_trait]
impl Brain for ClaudeBrain {
    type ChatMessage = ClaudeChatMessage;
    type ChatResponse = ClaudeChatResponse;

    fn api_key(&self) -> String {
        std::env::var("CLAUDE_API_KEY").expect("Expected a valid Claude API key")
    }

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 512,
            system_message: String::new(),
            strip_italic_actions: true,
        }
    }

    fn system_messages(&self, _bot_name: String, _message: String) -> Vec<ClaudeChatMessage> {
        vec![]
    }

    fn make_user_message(&self, message_content: String) -> ClaudeChatMessage {
        ClaudeChatMessage {
            role: "user".to_string(),
            content: message_content,
        }
    }

    fn convert_response_to_message(&self, response: Self::ChatResponse) -> Self::ChatMessage {
        let content = response
            .content
            .first()
            .map(|r| r.text.clone())
            .unwrap_or_default();

        ClaudeChatMessage {
            role: "assistant".to_string(),
            content,
        }
    }

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        messages: Vec<Self::ChatMessage>,
    ) -> anyhow::Result<Self::ChatResponse> {
        let request = ClaudeRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages,
            system: format!("{}: {}", DEFAULT_SYSTEM_PROMPT, params.system_message),
        };

        let response = self
            .http_client()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.api_key())
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let mut response: Self::ChatResponse = response.json().await?;

        if params.strip_italic_actions {
            response.content.iter_mut().for_each(|reply| {
                reply.text = remove_italic_actions(&reply.text);
            });
        }

        response.content.iter_mut().for_each(|reply| {
            let content = reply.text.to_uppercase().trim().to_owned();
            if content.contains("<AWAIT>") {
                reply.text = "<AWAIT>".to_string();
            } else if content.contains("<EXIT>") {
                reply.text = "<EXIT>".to_string();
            }
        });

        Ok(response)
    }
}
