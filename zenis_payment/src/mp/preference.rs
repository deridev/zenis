use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

use super::common::{CheckoutProPayer, Item, PersonalIdentification};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PreferencePaymentMethods {
    pub excluded_payment_methods: Vec<String>,
    pub excluded_payment_types: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub default_payment_method_id: String,
    pub installments: Option<u64>,
    pub default_installments: Option<u64>,
}

impl Default for PreferencePaymentMethods {
    fn default() -> Self {
        Self {
            excluded_payment_methods: vec![],
            excluded_payment_types: vec![],
            default_payment_method_id: "pix".to_string(),
            installments: None,
            default_installments: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub struct CheckoutProPreference {
    pub payer: Option<CheckoutProPayer>,
    pub items: Vec<Item>,
    pub expires: bool,
    pub notification_url: String,
    pub external_reference: Option<String>,
    pub payment_methods: PreferencePaymentMethods,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PreferenceBuilder {
    preference: CheckoutProPreference,
}

impl CheckoutProPreference {
    pub fn builder() -> PreferenceBuilder {
        PreferenceBuilder {
            preference: CheckoutProPreference::default(),
        }
    }
}

impl PreferenceBuilder {
    pub fn with_notification_url(mut self, notification_url: String) -> Self {
        self.preference.notification_url = notification_url;
        self
    }

    pub fn with_items(mut self, items: Vec<Item>) -> Self {
        self.preference.items = items;
        self
    }

    pub fn with_external_reference(mut self, external_reference: impl ToString) -> Self {
        self.preference.external_reference = Some(external_reference.to_string());
        self
    }

    pub fn expires(mut self, expires: bool) -> Self {
        self.preference.expires = expires;
        self
    }

    pub fn with_cpf(mut self, cpf: i64) -> Self {
        self.preference.payer = Some(CheckoutProPayer {
            identification: PersonalIdentification {
                document_type: Some("CPF".to_string()),
                number: Some(cpf),
            },
            ..Default::default()
        });
        self
    }

    pub fn build(self) -> CheckoutProPreference {
        self.preference
    }
}

/// Responses
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckoutProPreferencesResponse {
    pub id: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub client_id: i64,
    pub collector_id: i64,

    pub items: Vec<Item>,
    pub payer: CheckoutProPayer,

    /// Automatically generated URL to open the Checkout.
    #[serde(rename = "init_point")]
    pub checkout_url: String,

    /// Automatically generated URL to open the Checkout in sandbox mode. Real users are used here,
    /// but transactions are executed using test credentials.
    #[serde(rename = "sandbox_init_point")]
    pub checkout_sandbox_url: String,
}
