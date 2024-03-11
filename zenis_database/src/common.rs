use bson::{doc, oid::ObjectId, Document};
use chrono::TimeZone;
use serde::{Deserialize, Serialize};

pub fn query_by_id(id: ObjectId) -> Document {
    doc! {
        "_id": id
    }
}

pub fn _default_now() -> DatabaseDateTime {
    DatabaseDateTime(chrono::Utc::now())
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Deserialize, Serialize,
)]
pub struct DatabaseDateTime(
    // `bson::[..]_as_bson_datetime` converts a chrono::DateTime to a valid BSON DateTime representation
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub  chrono::DateTime<chrono::Utc>,
);

impl DatabaseDateTime {
    pub fn zeroed() -> Self {
        Self(
            chrono::Utc
                .timestamp_millis_opt(0)
                .single()
                .unwrap_or_default(),
        )
    }

    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for DatabaseDateTime {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for DatabaseDateTime {
    type Target = chrono::DateTime<chrono::Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
