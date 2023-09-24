use std::error::Error;
use std::collections::HashMap;
use nacos_sdk::api::config::ConfigService;

use volo_gen::frontend_base_service::{
    I18nRequest,
    JsonReply,
};

pub async fn get_route_service(config_service: impl ConfigService) -> Result<JsonReply, Box<dyn Error>> {
    let config_resp = config_service.get_config("frontend-route".to_owned(), "frontend-base-service".to_string()).await?;
    Ok(JsonReply {
        result: config_resp.content().to_string().into(),
    })
}

pub async fn get_i18n_service(i18n_request: I18nRequest, config_service: impl ConfigService) -> Result<JsonReply, Box<dyn Error>> {
    let config_resp = config_service.get_config(format!("i18n-{}", i18n_request.lang), "frontend-base-service".to_string()).await?;
    let lang_map: HashMap<String, String> = serde_yaml::from_str(&config_resp.content())?;
    Ok(JsonReply {
        result: serde_json::to_string(&lang_map)?.into(),
    })
}
