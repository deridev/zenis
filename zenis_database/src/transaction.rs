use std::hash::Hash;

use bson::{doc, oid::ObjectId, Document};
use mongodb::Collection;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use zenis_common::Cache;

use crate::{common::query_by_id, ZenisDatabase};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreditDestination {
    User(u64),
    PublicGuild(u64),
    PrivateGuild(u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionModel {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub discord_user_id: u64,
    pub product_id: String,
    pub credit_destination: CreditDestination,
    pub expires_at_timestamp: i64
}

impl TransactionModel {
    pub fn new(
        discord_user_id: u64,
        product_id: impl ToString,
        credit_destination: CreditDestination,
    ) -> Self {
        Self {
            id: ObjectId::new(),
            discord_user_id,
            product_id: product_id.to_string(),
            credit_destination,
            expires_at_timestamp: (chrono::Utc::now() + chrono::Duration::try_hours(8).unwrap()).timestamp()
        }
    }
}

static CACHE_ID: Lazy<Cache<ObjectId, TransactionModel>> = Lazy::new(|| Cache::new(1000));

#[allow(unused)]
pub struct TransactionCommands {
    pub collection: Collection<TransactionModel>,
    db: ZenisDatabase,
}

impl TransactionCommands {
    pub const fn new(collection: Collection<TransactionModel>, db: ZenisDatabase) -> Self {
        Self { collection, db }
    }

    pub async fn save(&self, transaction: TransactionModel) -> anyhow::Result<()> {
        CACHE_ID.remove(&transaction.id);
        self.collection
            .replace_one(query_by_id(transaction.id), &transaction, None)
            .await?;
        Ok(())
    }

    async fn get<K: Eq + Hash>(
        &self,
        cache: &Cache<K, TransactionModel>,
        key: K,
        query: Document,
    ) -> anyhow::Result<Option<TransactionModel>> {
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

    pub async fn get_by_id(&self, id: ObjectId) -> anyhow::Result<Option<TransactionModel>> {
        self.get(&CACHE_ID, id, query_by_id(id)).await
    }

    pub async fn delete_transaction(&self, id: ObjectId) -> anyhow::Result<()> {
        self.collection.delete_one(query_by_id(id), None).await?;
        Ok(())
    }

    pub async fn create_transaction(&self, transaction: TransactionModel) -> anyhow::Result<()> {
        self.collection.insert_one(transaction, None).await?;

        Ok(())
    }

    pub async fn delete_expired_transactions(&self) -> anyhow::Result<()> {
        let now = chrono::Utc::now().timestamp();
        self
            .collection
            .delete_many(doc! { "expires_at": { "$lt": now } }, None)
            .await?;

        Ok(())
    }
}
