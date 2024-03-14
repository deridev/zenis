use super::{
    common::{Item, PaymentPayload},
    preference::*,
};
use bson::oid::ObjectId;
use chrono::TimeDelta;

#[derive(Debug, Clone)]
pub struct MercadoPagoClient {
    pub debug: bool,
    access_token: String,
    client: reqwest::Client,
}

impl MercadoPagoClient {
    pub async fn new(debug: bool, access_token: impl Into<String>) -> anyhow::Result<Self> {
        Ok(Self {
            access_token: access_token.into(),
            debug,
            client: reqwest::Client::new(),
        })
    }

    pub fn notification_url(&self, id: impl ToString) -> String {
        format!(
            "{}/{}",
            std::env::var("MERCADO_PAGO_NOTIFICATION_URL").unwrap(),
            id.to_string()
        )
    }

    pub async fn create_preference(
        &self,
        transaction_id: ObjectId,
        items: Vec<Item>,
    ) -> anyhow::Result<CheckoutProPreferencesResponse> {
        let request = CheckoutProPreference::builder()
            .with_notification_url(self.notification_url(transaction_id))
            .with_items(items.clone())
            .with_expiration_duration(
                TimeDelta::try_minutes(30)
                    .expect("TimeDelta::try_minutes failed. It should never fail."),
            )
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

    pub async fn get_payment(&self, id: String) -> anyhow::Result<PaymentPayload> {
        let response = self
            .client
            .get(format!("https://api.mercadopago.com/v1/payments/{}", id))
            .bearer_auth(&self.access_token)
            .send()
            .await?
            .json::<PaymentPayload>()
            .await?;

        Ok(response)
    }
}
