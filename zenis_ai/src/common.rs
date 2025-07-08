use serde::{Deserialize, Serialize};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArenaTag {
    ExageratedAction,
    InvalidAction,
    OPAction,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArenaMessage {
    Input(ArenaInput),
    Output(ArenaOutput),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArenaInput {
    pub character_name: String,
    pub action: String,
    pub luck: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArenaOutput {
    #[serde(default = "Vec::new")]
    pub tags: Vec<ArenaTag>,
    pub output_message: String,
    pub consequences: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "Option::default")]
    pub winner: Option<String>,
}

impl ArenaOutput {
    pub fn make_invalid(string: &str) -> ArenaOutput {
        ArenaOutput {
            tags: vec![],
            output_message: string.to_string(),
            consequences: "MALFORMED_INPUT".to_string(),
            winner: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArenaCharacter {
    pub name: String,
    pub description: String,
}
