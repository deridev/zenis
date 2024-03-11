use std::sync::Arc;

use tokio::sync::RwLock;
use zenis_discord::twilight_model::id::{marker::UserMarker, Id};

use super::{common::Item, preference::*};

#[derive(Debug, Clone)]
pub struct Transaction {
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

    pub fn notification_url(&self) -> String {
        std::env::var("MERCADO_PAGO_NOTIFICATION_URL").unwrap()
    }

    pub async fn create_preference(
        &self,
        user_id: Id<UserMarker>,
        items: Vec<Item>,
    ) -> anyhow::Result<CheckoutProPreferencesResponse> {
        let request = CheckoutProPreference::builder()
            .with_notification_url(self.notification_url())
            .with_items(items.clone())
            .with_external_reference("ref_testing_zenis")
            .with_cpf(15322988383)
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

        self.transactions.write().await.push(Transaction {
            discord_user_id: user_id,
            item: items
                .first()
                .map(|i| i.id.clone().unwrap_or_default())
                .unwrap_or_default(),
        });

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
}
