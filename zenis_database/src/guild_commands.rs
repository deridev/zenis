use std::hash::Hash;

use bson::{doc, oid::ObjectId, Document};
use mongodb::Collection;
use once_cell::sync::Lazy;
use tokio_stream::StreamExt;
use zenis_common::Cache;
use zenis_discord::twilight_model::id::{marker::GuildMarker, Id};

use crate::{common::query_by_id, guild_model::GuildModel, ZenisDatabase};

static CACHE_ID: Lazy<Cache<ObjectId, GuildModel>> = Lazy::new(|| Cache::new(1000));
static CACHE_GUILD_ID: Lazy<Cache<String, GuildModel>> = Lazy::new(|| Cache::new(1000));

#[allow(unused)]
pub struct GuildCommands {
    pub collection: Collection<GuildModel>,
    db: ZenisDatabase,
}

impl GuildCommands {
    pub const fn new(collection: Collection<GuildModel>, db: ZenisDatabase) -> Self {
        Self { collection, db }
    }

    pub async fn save(&self, guild_data: GuildModel) -> anyhow::Result<()> {
        CACHE_ID.remove(&guild_data.id);
        CACHE_GUILD_ID.remove(&guild_data.guild_id);
        self.collection
            .replace_one(query_by_id(guild_data.id), &guild_data, None)
            .await?;
        Ok(())
    }

    pub fn remove_from_cache(&self, guild_data: &GuildModel) {
        CACHE_ID.remove(&guild_data.id);
        CACHE_GUILD_ID.remove(&guild_data.guild_id);
    }

    async fn get<K: Eq + Hash>(
        &self,
        cache: &Cache<K, GuildModel>,
        key: K,
        query: Document,
    ) -> anyhow::Result<Option<GuildModel>> {
        let cached = cache.get_cloned(&key);
        match cached {
            Some(model) => Ok(Some(model)),
            None => {
                let Some(model) = self.collection.find_one(query, None).await? else {
                    return Ok(None);
                };

                cache.insert(key, model.clone());
                Ok(Some(model))
            }
        }
    }

    pub async fn get_by_id(&self, id: ObjectId) -> anyhow::Result<Option<GuildModel>> {
        self.get(&CACHE_ID, id, query_by_id(id)).await
    }

    pub async fn get_by_guild(&self, guild_id: Id<GuildMarker>) -> anyhow::Result<GuildModel> {
        let query = doc! {
            "guild_id": guild_id.get().to_string(),
        };

        match self
            .get(&CACHE_GUILD_ID, guild_id.to_string(), query)
            .await?
        {
            Some(guild_data) => Ok(guild_data),
            None => Ok(self.create_guild_data(guild_id).await?),
        }
    }

    pub async fn get_all_guilds(&self) -> anyhow::Result<Vec<GuildModel>> {
        let query = doc! {};

        Ok(self
            .collection
            .find(query, None)
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?)
    }

    pub async fn create_guild_data(&self, guild_id: Id<GuildMarker>) -> anyhow::Result<GuildModel> {
        let guild_data = GuildModel::new(guild_id);
        self.collection.insert_one(guild_data.clone(), None).await?;

        Ok(guild_data)
    }
}
