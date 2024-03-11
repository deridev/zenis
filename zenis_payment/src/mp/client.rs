use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use zenis_discord::twilight_model::id::{marker::UserMarker, Id};

use super::{common::Item, preference::*};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct TransactionId(u64);

impl TransactionId {
    pub fn new() -> Self {
        static TRANSACTION_ID: AtomicU64 = AtomicU64::new(0);
        let id = TRANSACTION_ID.fetch_add(1, Ordering::Relaxed);

        Self(id)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for TransactionId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub discord_user_id: Id<UserMarker>,
    pub item: String,
}

#[derive(Debug, Clone)]
pub struct MercadoPagoClient {
    pub debug: bool,
    access_token: String,
    client: reqwest::Client,

    transactions: Arc<RwLock<Vec<Transaction>>>,
}

impl MercadoPagoClient {
    pub async fn new(debug: bool, access_token: impl Into<String>) -> anyhow::Result<Self> {
        Ok(Self {
            access_token: access_token.into(),
            debug,
            client: reqwest::Client::new(),

            transactions: Arc::new(RwLock::new(vec![])),
        })
    }

    pub fn notification_url(&self, id: TransactionId) -> String {
        format!(
            "{}/{}",
            std::env::var("MERCADO_PAGO_NOTIFICATION_URL").unwrap(),
            id.get()
        )
    }

    pub async fn create_preference(
        &self,
        user_id: Id<UserMarker>,
        items: Vec<Item>,
    ) -> anyhow::Result<CheckoutProPreferencesResponse> {
        let transaction = Transaction {
            id: TransactionId::new(),
            discord_user_id: user_id,
            item: items
                .first()
                .map(|i| i.id.clone().unwrap_or_default())
                .unwrap_or_default(),
        };

        let request = CheckoutProPreference::builder()
            .with_notification_url(self.notification_url(transaction.id))
            .with_items(items.clone())
            .build();

        let response = self
            .client
            .post("https://api.mercadopago.com/checkout/preferences")
            .json(&request)
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .json::<CheckoutProPreferencesResponse>()
            .await?;

        self.transactions.write().await.push(transaction);

        Ok(response)
    }

    pub async fn get_preference(
        &self,
        id: String,
    ) -> anyhow::Result<CheckoutProPreferencesResponse> {
        let response = self
            .client
            .get(format!(
                "https://api.mercadopago.com/checkout/preferences/{}",
                id
            ))
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .json::<CheckoutProPreferencesResponse>()
            .await?;

        Ok(response)
    }

    pub async fn get_transaction(&self, id: TransactionId) -> Option<Transaction> {
        let transactons = self.transactions.read().await;
        transactons.iter().find(|t| t.id == id).cloned()
    }
}
