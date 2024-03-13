use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    brain::{Brain, BrainParameters, DEFAULT_SYSTEM_PROMPT},
    common::{ChatMessage, ChatResponse, Role},
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
    fn api_key(&self, _debug: bool) -> String {
        std::env::var("CLAUDE_API_KEY").expect("Expected a valid Claude API key")
    }

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            debug: true,
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 512,
            system_prompt: String::new(),
            strip_italic_actions: true,
        }
    }

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        let request = ClaudeRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages: messages
                .iter()
                .map(|m| ClaudeChatMessage {
                    role: match m.role {
                        Role::User => "user".to_string(),
                        Role::Assistant => "assistant".to_string(),
                    },
                    content: m.content.clone(),
                })
                .collect(),
            system: format!("{}\n<{}>", DEFAULT_SYSTEM_PROMPT, params.system_prompt),
        };

        let response = self
            .http_client()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.api_key(params.debug))
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let mut response: ClaudeChatResponse = response.json().await?;

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

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: response
                    .content
                    .first()
                    .map(|r| r.text.clone())
                    .unwrap_or_default(),
            },
        })
    }
}
