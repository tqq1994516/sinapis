// pub mod data_handle;
pub mod encryption;

use serde_json::Value;
use pilota::AHashMap;
use sonic_rs::FastStr;

pub fn extra_to_outer(extra: Option<Value>) -> AHashMap<FastStr, FastStr> {
    match extra {
        Some(info) => {
            match info {
                Value::Object(i) => {
                    let mut map: AHashMap<FastStr, FastStr> = AHashMap::new();
                    for (k, v) in i.iter() {
                        map.insert(k.to_owned().into(), v.to_string().into());
                    }
                    map
                }
                _ => AHashMap::new(),
            }
        }
        None => AHashMap::new(),
    }
}
