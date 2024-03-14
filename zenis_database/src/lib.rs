pub mod agent_commands;
pub mod agent_model;
pub mod common;
pub mod guild_commands;
pub mod guild_model;
pub mod instance_commands;
pub mod instance_model;
pub mod transaction;
pub mod user_commands;
pub mod user_model;

use std::sync::Arc;

use agent_commands::AgentCommands;
use agent_model::AgentModel;
use bson::doc;
use guild_commands::GuildCommands;
use guild_model::GuildModel;
use instance_commands::InstanceCommands;
use instance_model::InstanceModel;
use mongodb::{Client, Collection, Database, IndexModel};

pub use mongodb::bson;
pub use mongodb::error::Error as MongoDBError;
use transaction::TransactionCommands;
use user_commands::UserCommands;
use user_model::UserModel;

#[derive(Debug, Clone)]
pub enum DatabaseState {
    Debug,
    Release,
}

#[derive(Debug, Clone)]
pub struct ZenisDatabase {
    /* MongoDB's Client uses Arc internally */
    client: Client,
    state: Arc<DatabaseState>,
}

impl ZenisDatabase {
    pub async fn new(state: DatabaseState) -> ZenisDatabase {
        let uri = std::env::var("DATABASE_URI").unwrap();

        let client = Client::with_uri_str(&uri).await.unwrap();

        ZenisDatabase {
            client,
            state: Arc::new(state),
        }
    }

    pub async fn setup(&self) {
        let users: Collection<UserModel> = self.db().collection("users");
        users
            .create_index(
                IndexModel::builder().keys(doc! { "user_id": 1 }).build(),
                None,
            )
            .await
            .unwrap();

        // GUILD INDEXES
        let guilds: Collection<GuildModel> = self.db().collection("guilds");
        guilds
            .create_index(
                IndexModel::builder().keys(doc! { "guild_id": 1 }).build(),
                None,
            )
            .await
            .unwrap();

        // AGENT INDEXES
        let agents: Collection<AgentModel> = self.db().collection("agents");
        agents
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "identifier": 1, "tags": 1 })
                    .build(),
                None,
            )
            .await
            .unwrap();

        // INSTANCE INDEXES
        let instances: Collection<InstanceModel> = self.db().collection("instances");
        instances
            .create_index(
                IndexModel::builder().keys(doc! { "channel_id": 1 }).build(),
                None,
            )
            .await
            .unwrap();
    }

    pub fn db(&self) -> Database {
        self.client.database(match *self.state {
            DatabaseState::Debug => "zenis_debug",
            DatabaseState::Release => "zenis_release",
        })
    }

    pub fn users(&self) -> UserCommands {
        let collection = self.db().collection("users");
        UserCommands::new(collection, self.clone())
    }

    pub fn guilds(&self) -> GuildCommands {
        let collection = self.db().collection("guilds");
        GuildCommands::new(collection, self.clone())
    }

    pub fn agents(&self) -> AgentCommands {
        let collection = self.db().collection("agents");
        AgentCommands::new(collection, self.clone())
    }

    pub fn instances(&self) -> InstanceCommands {
        let collection = self.db().collection("instances");
        InstanceCommands::new(collection, self.clone())
    }

    pub fn transactions(&self) -> TransactionCommands {
        let collection = self.db().collection("transactions");
        TransactionCommands::new(collection, self.clone())
    }
}
