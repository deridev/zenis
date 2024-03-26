use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    brain::{Brain, BrainParameters, ARENA_CONTEXT_GENERATION_PROMPT},
    common::{ArenaCharacter, ArenaMessage, ArenaOutput, ChatMessage, ChatResponse, Role},
    util::remove_italic_actions,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MessageHistory {
    pub role: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CohereChatRequest {
    pub message: String,
    pub model: String,
    pub chat_history: Vec<MessageHistory>,
    #[serde(rename = "preamble")]
    pub system_prompt: String,
    pub max_tokens: usize,
    pub temperature: f64,
    pub frequency_penalty: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CohereChatResponse {
    pub text: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CohereBrain;

#[async_trait]
impl Brain for CohereBrain {
    fn api_key(&self, debug: bool) -> String {
        if debug {
            std::env::var("DEBUG_COHERE_API_KEY").expect("Expected a valid DEBUG Cohere API key")
        } else {
            std::env::var("COHERE_API_KEY").expect("Expected a valid Cohere API key")
        }
    }

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            debug: true,
            model: "command-r".to_string(),
            max_tokens: 300,
            system_prompt: String::new(),
            strip_italic_actions: true,
        }
    }

    async fn prompt_chat(
        &self,
        mut params: BrainParameters,
        mut messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        let last_message = messages.pop();

        if params.max_tokens < 750 {
            params.max_tokens = 750;
        }
        let request = CohereChatRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            message: last_message.map(|m| m.content.clone()).unwrap_or_default(),
            system_prompt: format!(
                "{}\n<{}>",
                self.system_prompt(messages.len()),
                params.system_prompt
            ),
            temperature: 0.6,
            chat_history: messages
                .iter()
                .map(|m| MessageHistory {
                    role: match m.role {
                        Role::User => "USER".to_string(),
                        Role::Assistant => "CHATBOT".to_string(),
                    },
                    message: m.content.clone(),
                })
                .collect(),
            frequency_penalty: 0.15,
        };

        let response = self
            .http_client()
            .post("https://api.cohere.ai/v1/chat")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header(
                "Authorization",
                format!("bearer {}", self.api_key(params.debug)),
            )
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let mut response: CohereChatResponse = response.json().await?;

        if params.strip_italic_actions {
            response.text = remove_italic_actions(&response.text);
        }

        let content = response.text.to_uppercase().trim().to_owned();
        if content.contains("{AWAIT}") {
            response.text = "{AWAIT}".to_string();
        } else if content.contains("{EXIT}") {
            response.text = "{EXIT}".to_string();
        }

        if content.contains('<') && content.contains('>') && content.contains('@') {
            response.text = remove_after_lt(&response.text).trim().to_string();
        }

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: response.text,
                image_url: None,
            },
        })
    }

    async fn prompt_arena(
        &self,
        params: BrainParameters,
        context: String,
        characters: Vec<ArenaCharacter>,
        messages: Vec<ArenaMessage>,
    ) -> anyhow::Result<ArenaMessage> {
        let mut arena_messages = Vec::with_capacity(messages.len());
        for message in messages {
            arena_messages.push(match message {
                ArenaMessage::Input(input) => {
                    let json = serde_json::to_string_pretty(&input)?;

                    MessageHistory {
                        role: "USER".to_string(),
                        message: json,
                    }
                }
                ArenaMessage::Error(error) => {
                    MessageHistory {
                        role: "USER".to_string(),
                        message: format!("[SYSTEM ERROR. REWRITE YOUR OUTPUT OR THE BOT WILL CRASH.]\n{}", error),
                    }
                }
                ArenaMessage::Output(output) => {
                    let json = serde_json::to_string_pretty(&output)?;

                    MessageHistory {
                        role: "CHATBOT".to_string(),
                        message: json,
                    }
                }
            });
        }

        while arena_messages.first().is_some_and(|m| m.role != "USER") {
            arena_messages.remove(0);
        }

        let last_message = arena_messages.pop();

        let request = CohereChatRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            message: last_message.map(|m| m.message).unwrap_or_default(),
            system_prompt: self.make_arena_system_prompt(
                arena_messages.len(),
                context,
                &characters,
            ),
            temperature: 0.6,
            chat_history: arena_messages,
            frequency_penalty: 0.15,
        };

        let response = self
            .http_client()
            .post("https://api.cohere.ai/v1/chat")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header(
                "Authorization",
                format!("bearer {}", self.api_key(params.debug)),
            )
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: CohereChatResponse = response.json().await?;

        let output = serde_json::from_str::<ArenaOutput>(&response.text)?;

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

        let request = CohereChatRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            message: format!("[\n{}\n]", fighter_strings.join(",\n")),
            system_prompt: ARENA_CONTEXT_GENERATION_PROMPT.to_owned(),
            temperature: 0.6,
            chat_history: vec![],
            frequency_penalty: 0.15,
        };

        let response = self
            .http_client()
            .post("https://api.cohere.ai/v1/chat")
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header(
                "Authorization",
                format!("bearer {}", self.api_key(params.debug)),
            )
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: CohereChatResponse = response.json().await?;
        Ok(response.text)
    }
}

fn remove_after_lt(input: &str) -> &str {
    if let Some(index) = input.find('<') {
        &input[..index]
    } else {
        input
    }
}
