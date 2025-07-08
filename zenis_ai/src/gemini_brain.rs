use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zenis_common::load_image_from_url;

use crate::{brain::*, common::*, util::remove_italic_actions};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GeminiBrain {
    pub model: GeminiModel,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GeminiModel {
    Flash25,
    Pro25,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiGenerateContentRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_settings: Option<Vec<GeminiSafetySetting>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiContentPart {
    Text { text: String },
    InlineData { inline_data: GeminiInlineData },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiInlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiSafetySetting {
    category: String,
    threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiGenerateContentResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_feedback: Option<GeminiPromptFeedback>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_ratings: Option<Vec<GeminiSafetySetting>>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiPromptFeedback {
    #[serde(skip_serializing_if = "Option::is_none")]
    block_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_ratings: Option<Vec<GeminiSafetySetting>>,
}

#[async_trait]
impl Brain for GeminiBrain {
    fn api_key(&self, _debug: bool) -> String {
        std::env::var("GEMINI_API_KEY").expect("Expected a valid Gemini API key")
    }

    fn default_parameters(&self) -> BrainParameters {
        BrainParameters {
            debug: true,
            model: match self.model {
                GeminiModel::Flash25 => "gemini-2.5-flash".to_string(),
                GeminiModel::Pro25 => "gemini-2.5-pro".to_string(),
            },
            max_tokens: 12000,
            system_prompt: String::new(),
            strip_italic_actions: false,
        }
    }

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        mut messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        let mut gemini_contents = Vec::with_capacity(messages.len());

        if !params.system_prompt.is_empty() {
            gemini_contents.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiContentPart::Text {
                    text: params.system_prompt,
                }],
            });
        }

        while messages.first().is_some_and(|m| m.role != Role::User) {
            messages.remove(0);
        }

        for message in messages {
            let mut parts = vec![];
            if let Some(image_url) = &message.image_url {
                let image = load_image_from_url(image_url).await?;
                parts.push(GeminiContentPart::InlineData {
                    inline_data: GeminiInlineData {
                        mime_type: image.mime_type,
                        data: image.data,
                    },
                });
            }
            parts.push(GeminiContentPart::Text {
                text: message.content,
            });

            gemini_contents.push(GeminiContent {
                role: match message.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "model".to_string(),
                },
                parts,
            });
        }

        let request = GeminiGenerateContentRequest {
            contents: gemini_contents,
            safety_settings: Some(vec![
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HARASSMENT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
            ]),
            generation_config: Some(GeminiGenerationConfig {
                stop_sequences: None,
                temperature: Some(0.8),
                max_output_tokens: Some(params.max_tokens),
                top_p: None,
                top_k: None,
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            params.model,
            self.api_key(params.debug)
        );

        let response = self.http_client().post(&url).json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: GeminiGenerateContentResponse = response.json().await?;

        let text = response
            .candidates
            .into_iter()
            .next()
            .and_then(|c| {
                c.content.parts.into_iter().find_map(|p| match p {
                    GeminiContentPart::Text { text } => Some(text),
                    _ => None,
                })
            })
            .unwrap_or_default();
        let final_content = if params.strip_italic_actions {
            remove_italic_actions(&text)
        } else {
            text
        };

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: final_content.trim().to_owned(),
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
        let mut gemini_contents = vec![GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiContentPart::Text {
                text: self.make_arena_system_prompt(messages.len(), context, &characters),
            }],
        }];

        for message in messages {
            gemini_contents.push(match message {
                ArenaMessage::Input(input) => {
                    let json = serde_json::to_string_pretty(&input)?;
                    GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiContentPart::Text { text: json }],
                    }
                }
                ArenaMessage::Error(error) => GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiContentPart::Text {
                        text: format!(
                            "[SYSTEM ERROR. REWRITE YOUR OUTPUT FOR THE LAST INPUT OR THE BOT WILL CRASH.]\n{}",
                            error
                        ),
                    }],
                },
                ArenaMessage::Output(output) => {
                    let json = serde_json::to_string_pretty(&output)?;
                    GeminiContent {
                        role: "model".to_string(),
                        parts: vec![GeminiContentPart::Text { text: json }],
                    }
                }
            });
        }

        // Gemini models typically alternate roles. If the last message is from the user,
        // we add an empty model message to prime the model for its response.
        if let Some(last_message) = gemini_contents.last() {
            if last_message.role == "user" {
                gemini_contents.push(GeminiContent {
                    role: "model".to_string(),
                    parts: vec![], // Empty parts for the model's turn to respond
                });
            }
        }

        if params.max_tokens < 1024 {
            params.max_tokens = 1024;
        }

        let request = GeminiGenerateContentRequest {
            contents: gemini_contents,
            safety_settings: Some(vec![
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HARASSMENT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
            ]),
            generation_config: Some(GeminiGenerationConfig {
                stop_sequences: None,
                temperature: Some(1.1),
                max_output_tokens: Some(params.max_tokens),
                top_p: None,
                top_k: None,
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            params.model,
            self.api_key(params.debug)
        );

        let response = self.http_client().post(&url).json(&request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: GeminiGenerateContentResponse = response.json().await?;
        let output_text = response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| match p {
                GeminiContentPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default();

        let output = serde_json::from_str::<ArenaOutput>(&output_text)
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

        let request = GeminiGenerateContentRequest {
            contents: vec![
                GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiContentPart::Text {
                        text: ARENA_CONTEXT_GENERATION_PROMPT.to_owned(),
                    }],
                },
                GeminiContent {
                    role: "model".to_string(), // Add a model turn to simulate previous interaction
                    parts: vec![GeminiContentPart::Text {
                        text: "Understood. Please provide the fighter details.".to_string(),
                    }],
                },
                GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiContentPart::Text {
                        text: format!("[\n{}\n]", fighter_strings),
                    }],
                },
            ],
            safety_settings: Some(vec![
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HARASSMENT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                    threshold: "BLOCK_NONE".to_string(),
                },
            ]),
            generation_config: Some(GeminiGenerationConfig {
                stop_sequences: None,
                temperature: Some(1.3),
                max_output_tokens: Some(params.max_tokens),
                top_p: None,
                top_k: None,
            }),
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            params.model,
            self.api_key(params.debug)
        );

        let response = self.http_client().post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let mut text = response.text().await?;
            text.truncate(1800);
            return Err(anyhow::anyhow!("Status code: {}\n{:?}", status, text));
        }

        let response: GeminiGenerateContentResponse = response.json().await?;
        let content = response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| match p {
                GeminiContentPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default();

        Ok(content)
    }
}
