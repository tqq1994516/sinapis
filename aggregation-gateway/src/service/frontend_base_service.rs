use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    Extension,
    extract::Query,
};
use pilota::serde::Deserialize;
use serde_json::Value;
use utoipa::IntoParams;
use pilota::prost::Message;
use prost_types::Any;

use volo_gen::frontend_base_service::{
    I18nRequest,
    JsonReply,
};

use crate::core::state::GrpcClientState;
use crate::core::response::{Res, ErrorShowType};

#[utoipa::path(
    get,
    path = "/v1/frontend-base-service/get-route",
    responses(
        (status = 200, description = "get route", body = Res)
    ),
)]
pub async fn get_route(Extension(state): Extension<Arc<Mutex<GrpcClientState>>>) -> Res<Value> {
    let request = {};
    let data = request.encode_to_vec();
    let data = Any {
        type_url: "".to_string(),
        value: data,
    };

    let response = state
        .lock()
        .await
        .client
        .invoke_service("frontend-base-service.mustard-seed", "getRoute", Some(data))
        .await
        .unwrap();

    if let Some(any) = &response.data {
        let data = &any.value;
        let resp = JsonReply::decode(&data[..]).unwrap();
        Res::with_data(serde_json::from_str(&resp.result).unwrap())
    } else {
        Res::with_err("route acquisition exception.", ErrorShowType::Notification)
    }
}

#[derive(Deserialize, IntoParams)]
pub struct I18nParams {
    lang: String,
}

#[utoipa::path(
    get,
    path = "/v1/frontend-base-service/get-i18n",
    responses(
        (status = 200, description = "get internationalization", body = Res)
    ),
    params(
        I18nParams
    )
)]
pub async fn get_i18n(Extension(state): Extension<Arc<Mutex<GrpcClientState>>>, Query(params): Query<I18nParams>) -> Res<Value> {
    let request = I18nRequest {
        lang: params.lang.into(),
    };
    let data = request.encode_to_vec();
    let data = Any {
        type_url: "".to_string(),
        value: data,
    };

    let response = state
        .lock()
        .await
        .client
        .invoke_service("frontend-base-service.mustard-seed", "getI18n", Some(data))
        .await
        .unwrap();

    if let Some(any) = &response.data {
        let data = &any.value;
        let resp = JsonReply::decode(&data[..]).unwrap();
        Res::with_data(serde_json::from_str(&resp.result).unwrap())
    } else {
        Res::with_err("Internationalization acquisition exception.", ErrorShowType::Notification)
    }
}
