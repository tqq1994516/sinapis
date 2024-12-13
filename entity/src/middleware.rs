use serde::{Deserialize, Serialize};
use chrono::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub snow_id: i64,
    pub aud: String,
    #[serde(with = "jwt_numeric_date")]
    pub exp: DateTime<Utc>,
    #[serde(with = "jwt_numeric_date")]
    pub iat: DateTime<Utc>,
    pub iss: String,
    #[serde(with = "jwt_numeric_date")]
    pub nbf: DateTime<Utc>,
    pub sub: i64,
}

mod jwt_numeric_date {
    use chrono::prelude::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(date.timestamp())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DateTime::from_timestamp(i64::deserialize(deserializer)?, 0)
            .expect("invalid timestamp value!"))
    }
}
