pub mod entities;

// use std::env;
// use serde_json::Value;
// use pilota::AHashMap;
// use apache_age::{tokio::{AgeClient, Client}, NoTls};

// use sea_orm::sqlx::error::BoxDynError;
// use sonic_rs::FastStr;

// pub async fn get_age() -> Result<Client, BoxDynError> {
//     let databse_url = env::var("DATABASE_URL")?;
//     let urls: Vec<&str> = databse_url.split("/").collect();
//     let info = urls[2].split(":").collect::<Vec<&str>>();
//     let user = info[0];
//     let password = info[1].split("@").collect::<Vec<&str>>()[0];
//     let host = info[1].split("@").collect::<Vec<&str>>()[1];
//     let port = info[2];

//     let (client, _) = Client::connect_age(
//         &format!(
//             "host={host} user={user} password={password} port={port}",
//             host = host,
//             user = user,
//             password = password,
//             port = port,
//         ),
//         NoTls,
//     )
//     .await?;
//     Ok(client)
// }

// pub fn extra_to_outer(extra: Option<Value>) -> AHashMap<FastStr, FastStr> {
//     let outer: AHashMap<FastStr, FastStr>;
//     match extra {
//         Some(info) => {
//             outer = match info {
//                 Value::Object(i) => {
//                     let mut map: AHashMap<FastStr, FastStr> = AHashMap::new();
//                     for (k, v) in i.iter() {
//                         map.insert(k.to_owned().into(), v.to_string().into());
//                     }
//                     map
//                 }
//                 _ => AHashMap::new(),
//             }
//         }
//         None => outer = AHashMap::new(),
//     };
//     outer
// }