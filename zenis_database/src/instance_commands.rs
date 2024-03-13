use std::hash::Hash;

use bson::{doc, oid::ObjectId, Document};
use mongodb::Collection;
use once_cell::sync::Lazy;
use tokio_stream::StreamExt;
use zenis_common::Cache;

use crate::{common::query_by_id, instance_model::InstanceModel, ZenisDatabase};

static CACHE_ID: Lazy<Cache<ObjectId, InstanceModel>> = Lazy::new(|| Cache::new(1000));

#[allow(unused)]
pub struct InstanceCommands {
    pub collection: Collection<InstanceModel>,
    db: ZenisDatabase,
}

impl InstanceCommands {
    pub const fn new(collection: Collection<InstanceModel>, db: ZenisDatabase) -> Self {
        Self { collection, db }
    }

    pub async fn save(&self, mut instance: InstanceModel) -> anyhow::Result<()> {
        instance.active = instance.exit_reason.is_none();
        CACHE_ID.remove(&instance.id);
        self.collection
            .replace_one(query_by_id(instance.id), &instance, None)
            .await?;
        Ok(())
    }

    pub fn remove_from_cache(&self, instance: &InstanceModel) {
        CACHE_ID.remove(&instance.id);
    }

    async fn get<K: Eq + Hash>(
        &self,
        cache: &Cache<K, InstanceModel>,
        key: K,
        query: Document,
    ) -> anyhow::Result<Option<InstanceModel>> {
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

    pub async fn delete_instance(&self, instance_id: ObjectId) -> anyhow::Result<()> {
        CACHE_ID.remove(&instance_id);
        self.collection
            .delete_one(query_by_id(instance_id), None)
            .await?;
        Ok(())
    }

    pub async fn all_actives(&self) -> anyhow::Result<Vec<InstanceModel>> {
        Ok(self
            .collection
            .find(doc! { "active": true }, None)
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?)
    }

    pub async fn all_inactives(&self) -> anyhow::Result<Vec<InstanceModel>> {
        Ok(self
            .collection
            .find(doc! { "active": false }, None)
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?)
    }

    pub async fn get_by_id(&self, id: ObjectId) -> anyhow::Result<Option<InstanceModel>> {
        self.get(&CACHE_ID, id, query_by_id(id)).await
    }

    pub async fn get_all_by_channel(&self, channel_id: u64) -> anyhow::Result<Vec<InstanceModel>> {
        let query = doc! {
            "channel_id": channel_id as i64,
        };

        Ok(self
            .collection
            .find(query, None)
            .await?
            .collect::<Result<Vec<_>, _>>()
            .await?)
    }

    pub async fn create_instance(&self, instance_model: InstanceModel) -> anyhow::Result<()> {
        self.collection.insert_one(instance_model, None).await?;

        Ok(())
    }
}
