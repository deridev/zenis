use std::hash::Hash;

use bson::{doc, oid::ObjectId, Document};
use mongodb::Collection;
use once_cell::sync::Lazy;
use tokio_stream::StreamExt;
use zenis_common::Cache;

use crate::{agent_model::AgentModel, common::query_by_id, ZenisDatabase};

static CACHE_ID: Lazy<Cache<ObjectId, AgentModel>> = Lazy::new(|| Cache::new(1000));
static CACHE_IDENTIFIER: Lazy<Cache<String, AgentModel>> = Lazy::new(|| Cache::new(1000));

#[allow(unused)]
pub struct AgentCommands {
    pub collection: Collection<AgentModel>,
    db: ZenisDatabase,
}

impl AgentCommands {
    pub const fn new(collection: Collection<AgentModel>, db: ZenisDatabase) -> Self {
        Self { collection, db }
    }

    pub async fn save(&self, agent_data: AgentModel) -> anyhow::Result<()> {
        CACHE_IDENTIFIER.remove(&agent_data.identifier);
        CACHE_ID.remove(&agent_data.id);
        self.collection
            .replace_one(query_by_id(agent_data.id), &agent_data)
            .await?;
        Ok(())
    }

    pub fn remove_from_cache(&self, agent_data: &AgentModel) {
        CACHE_IDENTIFIER.remove(&agent_data.identifier);
        CACHE_ID.remove(&agent_data.id);
    }

    async fn get<K: Eq + Hash>(
        &self,
        cache: &Cache<K, AgentModel>,
        key: K,
        query: Document,
    ) -> anyhow::Result<Option<AgentModel>> {
        let cached = cache.get_cloned(&key);
        match cached {
            Some(model) => Ok(Some(model)),
            None => {
                let Some(model) = self.collection.find_one(query).await? else {
                    return Ok(None);
                };

                cache.insert(key, model.clone());
                Ok(Some(model))
            }
        }
    }

    pub async fn get_by_id(&self, id: ObjectId) -> anyhow::Result<Option<AgentModel>> {
        self.get(&CACHE_ID, id, query_by_id(id)).await
    }

    pub async fn get_by_identifier(
        &self,
        identifier: impl ToString,
    ) -> anyhow::Result<Option<AgentModel>> {
        let identifier = identifier.to_string().to_lowercase();
        let identifier = identifier.trim();

        let query = doc! {
            "identifier": identifier,
        };

        self.get(&CACHE_IDENTIFIER, identifier.to_string(), query)
            .await
    }

    pub async fn get_all_with_tags(
        &self,
        tags: impl IntoIterator<Item = impl ToString>,
    ) -> anyhow::Result<Vec<AgentModel>> {
        let tags = tags.into_iter().map(|t| t.to_string()).collect::<Vec<_>>();

        let query = doc! {
            "tags": {
                "$all": tags,
            },
        };

        Ok(self
            .collection
            .find(query)
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?)
    }

    pub async fn get_all_by_creator(&self, creator_id: u64) -> anyhow::Result<Vec<AgentModel>> {
        let query = doc! {
            "creator_user_id": creator_id as i64,
        };

        Ok(self
            .collection
            .find(query)
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?)
    }

    pub async fn get_all_private_by_guild(&self, guild_id: u64) -> anyhow::Result<Vec<AgentModel>> {
        let query = doc! {
            "public": false,
            "guild_id": guild_id as i64,
        };

        Ok(self
            .collection
            .find(query)
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?)
    }

    pub async fn get_all_public(&self) -> anyhow::Result<Vec<AgentModel>> {
        let query = doc! {
            "public": true,
        };

        Ok(self
            .collection
            .find(query)
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?)
    }

    pub async fn create_agent(&self, agent_model: AgentModel) -> anyhow::Result<()> {
        if self
            .get_by_identifier(&agent_model.identifier)
            .await?
            .is_some()
        {
            return Err(anyhow::anyhow!("Agent with this identifier already exists"));
        }

        self.collection.insert_one(agent_model).await?;

        Ok(())
    }
}
