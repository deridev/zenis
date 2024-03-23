use std::{collections::HashSet, fmt::Display, hash::Hash};

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use zenis_discord::twilight_model::id::{marker::UserMarker, Id};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct UserSettings {
    pub is_notifications_enabled: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum UserFlags {
    AlreadyReceivedFreeGuildCredits,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum AdminPermission {
    All,
    UseAdmCommand,
    ManageCredits,
    ManageAgents,
    ManageInstances,
}

impl Display for AdminPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "ALL"),
            Self::UseAdmCommand => write!(f, "USE_ADM_COMMAND"),
            Self::ManageCredits => write!(f, "MANAGE_CREDITS"),
            Self::ManageAgents => write!(f, "MANAGE_AGENTS"),
            Self::ManageInstances => write!(f, "MANAGE_INSTANCES"),
        }
    }
}

impl TryFrom<String> for AdminPermission {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "all" => Ok(Self::All),
            "use_adm_command" => Ok(Self::UseAdmCommand),
            "manage_credits" => Ok(Self::ManageCredits),
            "manage_agents" => Ok(Self::ManageAgents),
            "manage_instances" => Ok(Self::ManageInstances),
            _ => Err(anyhow::anyhow!("Invalid admin permission")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct UserModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub user_id: u64,
    pub credits: i64,
    pub settings: UserSettings,
    #[serde(default = "HashSet::new")]
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub flags: HashSet<UserFlags>,
    #[serde(default = "HashSet::new")]
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub adm_permissions: HashSet<AdminPermission>,
}

impl UserModel {
    pub fn new(user_id: Id<UserMarker>) -> Self {
        Self {
            id: ObjectId::new(),
            user_id: user_id.get(),
            credits: 0,
            settings: UserSettings {
                is_notifications_enabled: true,
            },
            flags: HashSet::new(),
            adm_permissions: HashSet::new(),
        }
    }

    pub fn add_credits(&mut self, quantity: i64) {
        self.credits += quantity;
    }

    pub fn remove_credits(&mut self, quantity: i64) {
        self.credits = (self.credits - quantity).max(0);
    }

    pub fn insert_flag(&mut self, flag: UserFlags) {
        self.flags.insert(flag);
    }

    pub fn remove_flag(&mut self, flag: UserFlags) {
        self.flags.remove(&flag);
    }

    pub fn has_flag(&self, flag: UserFlags) -> bool {
        self.flags.contains(&flag)
    }

    pub fn has_admin_permission(&self, permission: AdminPermission) -> bool {
        self.adm_permissions.contains(&permission)
            || self.adm_permissions.contains(&AdminPermission::All)
    }

    pub fn insert_admin_permission(&mut self, permission: AdminPermission) {
        self.adm_permissions.insert(permission);
    }

    pub fn remove_admin_permission(&mut self, permission: AdminPermission) {
        self.adm_permissions.remove(&permission);
    }
}
