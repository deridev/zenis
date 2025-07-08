use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    brain::{Brain, BrainParameters, ARENA_CONTEXT_GENERATION_PROMPT},
    common::{ArenaCharacter, ArenaMessage, ArenaOutput, ChatMessage, ChatResponse, Role},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OpenAIChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OpenAIChatResponse {
    pub choices: Vec<OpenAIChatChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OpenAIChatChoice {
    pub message: OpenAIChatMessage,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ResponseFormat {
    pub r#type: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub max_tokens: usize,
    pub messages: Vec<OpenAIChatMessage>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpenAIBrain {
    pub model: OpenAIModel,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OpenAIModel {
    Gpt3,
    Gpt4o,
    Gpt4oMini,
    Finetuned,
}

#[async_trait]
impl Brain for OpenAIBrain {
    fn api_key(&self, _debug: bool) -> String {
        std::env::var("OPENAI_API_KEY").expect("Expected a valid OpenAI API key")
    }

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            debug: true,
            model: match self.model {
                OpenAIModel::Gpt3 => "gpt-3.5-turbo".to_string(),
                OpenAIModel::Gpt4o => "gpt-4o".to_string(),
                OpenAIModel::Gpt4oMini => "gpt-4o-mini".to_string(),
                OpenAIModel::Finetuned => {
                    "ft:gpt-4o-2024-08-06:personal:think-zenis-001:A4DAOcaI".to_string()
                }
            },
            max_tokens: 1500,
            system_prompt: String::new(),
            strip_italic_actions: true,
        }
    }

    async fn prompt_raw(
        &self,
        params: BrainParameters,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        let mut openai_messages = vec![OpenAIChatMessage {
            role: "system".to_string(),
            content: params.system_prompt.clone(),
        }];

        openai_messages.extend(messages.iter().map(|message| OpenAIChatMessage {
            role: match message.role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
            },
            content: message.content.clone(),
        }));

        let request = OpenAIRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages: openai_messages,
            temperature: 0.8,
            response_format: None,
        };

        let response = self
            .http_client()
            .post("https://api.openai.com/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key(params.debug)),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: OpenAIChatResponse = response.json().await?;

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: format!(
                    "{}",
                    response
                        .choices
                        .first()
                        .map(|choice| choice.message.content.clone())
                        .unwrap_or_default()
                ),
                image_url: None,
            },
        })
    }

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        let mut openai_messages = vec![OpenAIChatMessage {
            role: "system".to_string(),
            content: format!(
                "{}\n{}",
                self.system_prompt(messages.len()),
                params.system_prompt
            ),
        }];

        openai_messages.extend(messages.iter().map(|message| OpenAIChatMessage {
            role: match message.role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
            },
            content: message.content.clone(),
        }));

        let request = OpenAIRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages: openai_messages,
            temperature: 0.4,
            response_format: Some(ResponseFormat {
                r#type: "json_object".to_string(),
            }),
        };

        let response = self
            .http_client()
            .post("https://api.openai.com/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key(params.debug)),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: OpenAIChatResponse = response.json().await?;

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: format!(
                    "{}",
                    response
                        .choices
                        .first()
                        .map(|choice| choice.message.content.clone())
                        .unwrap_or_default()
                ),
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
        let system_prompt = self.make_arena_system_prompt(messages.len(), context, &characters);
        let mut openai_messages = vec![OpenAIChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }];

        openai_messages.extend(messages.iter().map(|message| match message {
                ArenaMessage::Input(input) => {
                    let json = serde_json::to_string_pretty(&input).unwrap();
                    OpenAIChatMessage {
                        role: "user".to_string(),
                        content: json,
                    }
                }
                ArenaMessage::Error(error) => OpenAIChatMessage {
                    role: "user".to_string(),
                    content: format!("[SYSTEM ERROR. REWRITE YOUR OUTPUT FOR THE LAST INPUT OR THE BOT WILL CRASH. ONLY JSON, NO MARKDOWN, NO ADDITIONAL TEXT.]\n{}", error),
                },
                ArenaMessage::Output(output) => {
                    let json = serde_json::to_string_pretty(&output).unwrap();
                    OpenAIChatMessage {
                        role: "assistant".to_string(),
                        content: json,
                    }
                },
            }));

        if params.max_tokens < 1324 {
            params.max_tokens = 1324;
        }

        let request = OpenAIRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages: openai_messages,
            temperature: 1.1,
            response_format: Some(ResponseFormat {
                r#type: "json_object".to_string(),
            }),
        };

        let response = self
            .http_client()
            .post("https://api.openai.com/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key(params.debug)),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: OpenAIChatResponse = response.json().await?;
        let output = response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();
        let output = serde_json::from_str::<ArenaOutput>(&output)
            .map_err(|e| anyhow::anyhow!("Failed to parse output as ArenaOutput: {}", e))?;

        Ok(ArenaMessage::Output(output))
    }

    async fn generate_context(&self, fighters: Vec<ArenaCharacter>) -> anyhow::Result<String> {
        let fighter_strings = fighters
            .iter()
            .map(|f| {
                format!(
                    "{{\"name\": \"{}\", \"description\": \"{}\"}}",
                    f.name, f.description
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");

        let params = self.default_parameters();

        let system_prompt = ARENA_CONTEXT_GENERATION_PROMPT.to_string();

        let request = OpenAIRequest {
            model: params.model,
            max_tokens: params.max_tokens,
            messages: vec![
                OpenAIChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                OpenAIChatMessage {
                    role: "user".to_string(),
                    content: format!("[\n{}\n]", fighter_strings),
                },
            ],
            temperature: 1.3,
            response_format: Some(ResponseFormat {
                r#type: "json_object".to_string(),
            }),
        };

        let response = self
            .http_client()
            .post("https://api.openai.com/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key(params.debug)),
            )
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get response from OpenAI API"));
        }

        let response: OpenAIChatResponse = response.json().await?;
        let content = response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        Ok(content)
    }
}
