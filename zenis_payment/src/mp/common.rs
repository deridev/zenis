pub use crate::helpers::*;
use serde::{Deserialize, Serialize};

// Code heavily inspired by https://github.com/saskenuba/mercadopago-sdk-rust
pub type DocumentType = String;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Item {
    pub title: String,
    pub description: String,
    pub quantity: i64,
    pub unit_price: f64,

    pub id: Option<String>,
    pub currency: Option<String>,
    pub picture_url: Option<String>,
    pub category_id: Option<String>,
}

impl Item {
    pub fn simple(
        price: f64,
        title: impl ToString,
        description: impl ToString,
        quantity: i64,
    ) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            quantity,
            unit_price: price,
            id: None,
            currency: None,
            picture_url: None,
            category_id: None,
        }
    }

    pub fn with_id(mut self, id: impl ToString) -> Self {
        self.id = Some(id.to_string());
        self
    }
}

/// Documents for personal identification, such as RG, CPF, CNH
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]

pub struct PersonalIdentification {
    #[serde(rename = "type")]
    pub document_type: Option<DocumentType>,
    #[serde(
        default,
        serialize_with = "option_stringify",
        deserialize_with = "serde_aux::field_attributes::deserialize_option_number_from_string"
    )]
    pub number: Option<i64>,
}

impl PersonalIdentification {
    pub fn new(document_type: DocumentType, document_number: i64) -> Self {
        Self {
            document_type: Some(document_type),
            number: Some(document_number),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Phone {
    #[serde(
        default,
        deserialize_with = "serde_aux::field_attributes::deserialize_option_number_from_string"
    )]
    pub area_code: Option<i64>,
    pub number: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]

pub struct Address {
    pub zip_code: Option<String>,
    pub state_name: Option<String>,
    pub city_name: Option<String>,
    pub street_name: Option<String>,
    pub street_number: Option<i64>,
}

/// A payer will ALWAYS have a `PersonalIdentification`, and an `email` since it's the bare minimum.
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CheckoutProPayer {
    pub(crate) email: Option<String>,
    pub identification: PersonalIdentification,

    pub name: Option<String>,
    pub surname: Option<String>,
    pub phone: Option<Phone>,
    pub address: Option<Address>,
}

impl CheckoutProPayer {
    pub fn validate(&self) -> bool {
        if self.email.is_none()
            || self.identification.number.is_none()
            || self.identification.document_type.is_none()
        {
            return false;
        }
        true
    }

    pub fn standard_payer<II>(
        email: String,
        document_type: DocumentType,
        document_number: II,
    ) -> Self
    where
        II: Into<Option<i64>>,
    {
        Self {
            email: Some(email),
            identification: PersonalIdentification {
                document_type: Some(document_type),
                number: document_number.into(),
            },

            name: None,
            surname: None,
            phone: None,
            address: None,
        }
    }

    pub fn minimal_payer<II>(
        email: String,
        document_type: DocumentType,
        document_number: II,
    ) -> Self
    where
        II: Into<Option<i64>>,
    {
        Self {
            email: Some(email),
            identification: PersonalIdentification {
                document_type: Some(document_type),
                number: document_number.into(),
            },

            name: None,
            surname: None,
            phone: None,
            address: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PaymentPayload {
    pub id: i64,
    pub payment_method_id: String,
    pub payment_type_id: String,
    pub status: String,
    pub status_detail: String,
    pub currency_id: String,
    pub description: String,
    pub external_reference: Option<String>,
    pub transaction_amount: i64,
}
