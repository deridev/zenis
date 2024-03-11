use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub action: String,
    pub api_version: String,
    pub id: i64,
    pub live_mode: bool,
    #[serde(rename = "type")]
    pub ty: String,
    pub date_created: String,
    pub user_id: String,
    pub data: NotificationData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]

pub struct NotificationData {
    pub id: String,
}
