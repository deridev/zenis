use serde::{Deserialize, Serialize};
use zenis_database::instance_model::InstanceMessage;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChatResponse {
    pub message: ChatMessage,
}

impl From<ChatMessage> for InstanceMessage {
    fn from(value: ChatMessage) -> Self {
        Self {
            is_user: value.role == Role::User,
            content: value.content,
            image_url: value.image_url,
        }
    }
}

impl From<ChatResponse> for InstanceMessage {
    fn from(value: ChatResponse) -> Self {
        value.message.into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArenaTag {
    ExageratedAction,
    InvalidAction,
    OPAction,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArenaMessage {
    Input(ArenaInput),
    Output(ArenaOutput),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArenaInput {
    pub character_name: String,
    pub action: String,
    pub luck: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArenaOutput {
    pub tags: Vec<ArenaTag>,
    pub output_message: String,
    pub consequences: String,
    pub winner: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArenaCharacter {
    pub name: String,
    pub description: String,
}
