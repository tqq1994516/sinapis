use pool::dapr::person_center::GrpcClientManager;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;

pub struct Middleware {
    dapr_grpc_client: GrpcClientManager,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: String,
    #[serde(with = "jwt_numeric_date")]
    pub exp: DateTime<Utc>,
    #[serde(with = "jwt_numeric_date")]
    pub iat: DateTime<Utc>,
    pub iss: String,
    #[serde(with = "jwt_numeric_date")]
    pub nbf: DateTime<Utc>,
    pub sub: String,
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
