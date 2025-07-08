use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::common::{ArenaCharacter, ArenaMessage, ChatMessage, ChatResponse};

static DEFAULT_CLIENT: Lazy<Arc<reqwest::Client>> = Lazy::new(|| Arc::new(reqwest::Client::new()));

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BrainParameters {
    pub debug: bool,
    pub model: String,
    pub max_tokens: usize,
    pub system_prompt: String,
    pub strip_italic_actions: bool,
}

pub const DEFAULT_CHAT_SYSTEM_PROMPT: &str = include_str!("default_chat_system_prompt.txt");
pub const CHAT_SYSTEM_PROMPT_EXAMPLES: &str = include_str!("chat_system_prompt_examples.txt");

pub const DEFAULT_ARENA_SYSTEM_PROMPT: &str = include_str!("arena_system_prompt.txt");
pub const ARENA_FULL_EXAMPLES: &str = include_str!("arena_full_examples.txt");
pub const ARENA_PARTIAL_EXAMPLES: &str = include_str!("arena_partial_examples.txt");

pub const ARENA_CONTEXT_GENERATION_PROMPT: &str =
    include_str!("arena_context_generation_prompt.txt");

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

    fn system_prompt(&self, messages: usize) -> String {
        let mut system_prompt = DEFAULT_CHAT_SYSTEM_PROMPT.to_string();

        if messages < 3 {
            system_prompt = system_prompt.replace("%EXAMPLES%", CHAT_SYSTEM_PROMPT_EXAMPLES);
        } else {
            system_prompt = system_prompt.replace("%EXAMPLES%", "");
        }

        system_prompt
    }

    fn make_arena_system_prompt(
        &self,
        messages_len: usize,
        context: String,
        characters: &[ArenaCharacter],
    ) -> String {
        let mut system_prompt = DEFAULT_ARENA_SYSTEM_PROMPT.to_string();

        if messages_len < 3 {
            system_prompt = system_prompt.replace("%EXAMPLES%", ARENA_FULL_EXAMPLES);
        } else {
            system_prompt = system_prompt.replace("%EXAMPLES%", ARENA_PARTIAL_EXAMPLES);
        }

        let character_table = characters
            .iter()
            .map(|c| {
                format!(
                    "   {{ \"name\": \"{}\", \"description\": \"{}\" }}",
                    c.name, c.description
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");

        format!("{system_prompt}\n[REAL BATTLE]\nCharacterTable: [\n{character_table}\n]\nBattleContext: \"{context}\"")
    }

    fn http_client(&self) -> Arc<reqwest::Client> {
        DEFAULT_CLIENT.clone()
    }

    async fn prompt_raw(
        &self,
        params: BrainParameters,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse> {
        self.prompt_chat(params, messages).await
    }

    async fn prompt_chat(
        &self,
        params: BrainParameters,
        messages: Vec<ChatMessage>,
    ) -> anyhow::Result<ChatResponse>;

    async fn prompt_arena(
        &self,
        params: BrainParameters,
        context: String,
        characters: Vec<ArenaCharacter>,
        messages: Vec<ArenaMessage>,
    ) -> anyhow::Result<ArenaMessage>;

    async fn generate_context(&self, fighters: Vec<ArenaCharacter>) -> anyhow::Result<String>;
}
