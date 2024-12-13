use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct Account {
    pub user_id: i64,
    pub snow_id: i64,
    pub refresh_token: String,
    #[serde(with = "jwt_str_date")]
    pub exp: DateTime<Utc>,
}

mod jwt_str_date {
    use chrono::prelude::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // date时间戳转字符串再执行序列化
        serializer.serialize_str(&date.timestamp().to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 原始数据反序列化为字符串类型再转i64构造时间戳
        let s = String::deserialize(deserializer)?;
        Ok(DateTime::from_timestamp(s.parse::<i64>().unwrap(), 0)
            .expect("invalid timestamp value!"))
    }
}
