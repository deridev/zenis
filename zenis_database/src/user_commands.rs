use std::hash::Hash;

use bson::{doc, oid::ObjectId, Document};
use mongodb::Collection;
use once_cell::sync::Lazy;
use zenis_common::Cache;
use zenis_discord::twilight_model::id::{marker::UserMarker, Id};

use crate::{common::query_by_id, user_model::UserModel, ZenisDatabase};

static CACHE_ID: Lazy<Cache<ObjectId, UserModel>> = Lazy::new(|| Cache::new(1000));
static CACHE_USER_ID: Lazy<Cache<u64, UserModel>> = Lazy::new(|| Cache::new(1000));

#[allow(unused)]
pub struct UserCommands {
    pub collection: Collection<UserModel>,
    db: ZenisDatabase,
}

impl UserCommands {
    pub const fn new(collection: Collection<UserModel>, db: ZenisDatabase) -> Self {
        Self { collection, db }
    }

    pub async fn save(&self, user_data: UserModel) -> anyhow::Result<()> {
        CACHE_ID.remove(&user_data.id);
        CACHE_USER_ID.remove(&user_data.user_id);
        self.collection
            .replace_one(query_by_id(user_data.id), &user_data)
            .await?;
        Ok(())
    }

    pub fn remove_from_cache(&self, user_data: &UserModel) {
        CACHE_ID.remove(&user_data.id);
        CACHE_USER_ID.remove(&user_data.user_id);
    }

    async fn get<K: Eq + Hash>(
        &self,
        cache: &Cache<K, UserModel>,
        key: K,
        query: Document,
    ) -> anyhow::Result<Option<UserModel>> {
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

    pub async fn get_by_id(&self, id: ObjectId) -> anyhow::Result<Option<UserModel>> {
        self.get(&CACHE_ID, id, query_by_id(id)).await
    }

    pub async fn get_by_user(&self, user_id: Id<UserMarker>) -> anyhow::Result<UserModel> {
        let query = doc! {
            "user_id": user_id.get() as i64,
        };

        match self.get(&CACHE_USER_ID, user_id.get(), query).await? {
            Some(user_data) => Ok(user_data),
            None => Ok(self.create_user_data(user_id).await?),
        }
    }

    pub async fn create_user_data(&self, user_id: Id<UserMarker>) -> anyhow::Result<UserModel> {
        let user_data = UserModel::new(user_id);
        self.collection.insert_one(user_data.clone()).await?;

        Ok(user_data)
    }
}
