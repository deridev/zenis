use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zenis_common::load_image_from_url;

use crate::{
    brain::{Brain, BrainParameters, ARENA_CONTEXT_GENERATION_PROMPT},
    common::{ArenaCharacter, ArenaMessage, ArenaOutput, ChatMessage, ChatResponse, Role},
    util::remove_italic_actions,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeImage {
    #[serde(rename = "type")]
    pub ty: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeContent {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ClaudeImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClaudeChatMessage {
    pub role: String,
    pub content: Vec<ClaudeContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct ClaudeChatResponse {
    pub content: Vec<ClaudeContent>,
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
            model: "claude-3-haiku-20240307".to_string(),
            max_tokens: 300,
            system_prompt: String::new(),
            strip_italic_actions: true,
        }
    }

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        mut messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        let mut claude_messages = Vec::with_capacity(messages.len());
        let len = messages.len();

        while messages.first().is_some_and(|m| m.role != Role::User) {
            messages.remove(0);
        }

        for (index, message) in messages.iter().enumerate() {
            let mut contents = vec![];
            if let Some(image_url) = &message.image_url {
                if index == len - 1 && message.role == Role::User {
                    let image = load_image_from_url(image_url).await?;
                    contents.push(ClaudeContent {
                        ty: "image".to_string(),
                        source: Some(ClaudeImage {
                            ty: "base64".to_string(),
                            media_type: image.mime_type,
                            data: image.data,
                        }),
                        text: None,
                    });
                }
            }

            contents.push(ClaudeContent {
                ty: "text".to_string(),
                text: Some(message.content.clone()),
                source: None,
            });

            let claude_message = ClaudeChatMessage {
                role: match message.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: contents,
            };

            claude_messages.push(claude_message);
        }

        let request = ClaudeRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages: claude_messages,
            system: format!(
                "{}\n<{}>",
                self.system_prompt(messages.len()),
                params.system_prompt
            ),
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

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let mut response: ClaudeChatResponse = response.json().await?;

        if params.strip_italic_actions {
            response.content.iter_mut().for_each(|reply| {
                reply.text = reply.text.as_ref().map(|t| remove_italic_actions(t));
            });
        }

        response.content.iter_mut().for_each(|reply| {
            let Some(content) = &mut reply.text else {
                return;
            };

            let text = content.to_uppercase().trim().to_owned();
            if text.contains("{AWAIT}") {
                *content = "{AWAIT}".to_string();
            } else if text.contains("{EXIT}") {
                *content = "{EXIT}".to_string();
            }
        });

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: response
                    .content
                    .first()
                    .map(|r| r.text.clone().unwrap_or_default())
                    .unwrap_or_default()
                    .trim()
                    .to_owned(),
                image_url: None,
            },
        })
    }

    async fn prompt_arena(
        &self,
        mut params: BrainParameters,
        context: String,
        characters: Vec<ArenaCharacter>,
        messages: Vec<ArenaMessage>,
    ) -> anyhow::Result<ArenaMessage> {
        let mut claude_messages = Vec::with_capacity(messages.len());
        for message in messages {
            claude_messages.push(match message {
                ArenaMessage::Input(input) => {
                    let json = serde_json::to_string_pretty(&input)?;

                    ClaudeChatMessage {
                        role: "user".to_string(),
                        content: vec![ClaudeContent {
                            ty: "text".to_string(),
                            text: Some(json.to_string()),
                            source: None,
                        }],
                    }
                }
                ArenaMessage::Output(output) => {
                    let json = serde_json::to_string_pretty(&output)?;

                    ClaudeChatMessage {
                        role: "assistant".to_string(),
                        content: vec![ClaudeContent {
                            ty: "text".to_string(),
                            text: Some(json.to_string()),
                            source: None,
                        }],
                    }
                }
            });
        }

        while claude_messages.first().is_some_and(|m| m.role != "user") {
            claude_messages.remove(0);
        }

        if params.max_tokens < 450 {
            params.max_tokens = 450;
        }

        let request = ClaudeRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            system: self.make_arena_system_prompt(claude_messages.len(), context, &characters),
            messages: claude_messages,
        };

        let response = self
            .http_client()
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.api_key(params.debug))
            .header("content-type", "application/json")
            .header("Anthropic-Version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: ClaudeChatResponse = response.json().await?;
        let Some(output) = response
            .content
            .first()
            .map(|r| r.text.clone().unwrap_or_default())
        else {
            return Err(anyhow::anyhow!("No output found"));
        };

        let output = match serde_json::from_str::<ArenaOutput>(&output) {
            Ok(output) => output,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse output as ArenaOutput.\nOUTPUT: {output}\n{}",
                    e
                ))
            }
        };

        Ok(ArenaMessage::Output(output))
    }

    async fn generate_context(&self, fighters: Vec<ArenaCharacter>) -> anyhow::Result<String> {
        let mut fighter_strings = vec![];

        for fighter in fighters {
            fighter_strings.push(format!(
                "   {{ \"name\": \"{}\", \"description\": \"{}\" }}",
                fighter.name, fighter.description
            ));
        }

        let params = self.default_parameters();

        let request = ClaudeRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages: vec![ClaudeChatMessage {
                role: "user".to_string(),
                content: vec![ClaudeContent {
                    ty: "text".to_string(),
                    text: Some(format!("[\n{}\n]", fighter_strings.join(",\n"))),
                    source: None,
                }],
            }],
            system: ARENA_CONTEXT_GENERATION_PROMPT.to_owned(),
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

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: ClaudeChatResponse = response.json().await?;

        let Some(output) = response
            .content
            .first()
            .map(|r| r.text.clone().unwrap_or_default())
        else {
            return Err(anyhow::anyhow!("No output found"));
        };

        Ok(output)
    }
}
