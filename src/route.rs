use std::sync::Arc;
use axum::{
    Router,
    routing::get,
    Extension
};
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::service::frontend_base_service::{
    get_route,
    get_i18n,
};
use crate::core::state::GrpcClientState;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::service::frontend_base_service::get_route,
        crate::service::frontend_base_service::get_i18n
    ),
    components(
        schemas(crate::core::response::Res<serde_json::Value>)
    ),
    // modifiers(&SecurityAddon),
    tags(
        (name = "sinapis", description = "Sinapis management API")
    )
)]
struct ApiDoc;

// struct SecurityAddon;

// impl Modify for SecurityAddon {
//     fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
//         if let Some(components) = openapi.components.as_mut() {
//             components.add_security_scheme(
//                 "api_key",
//                 SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
//             )
//         }
//     }
// }

pub async fn register_route() -> Router {
    const API_VERSION: &str = "v1";
    let dapr_grpc_client_extension = Arc::new(Mutex::new(GrpcClientState::build().await.expect("Grpc client connect failed.")));
    let frontend_base_service_router = Router::new()
        .layer(Extension(dapr_grpc_client_extension))
        .route("/get-route", get(get_route))
        .route("/get-i18n", get(get_i18n));

    let aggregation_router = Router::new()
        .nest(&format!("/{}/frontend-base-service", API_VERSION), frontend_base_service_router)
        .merge(SwaggerUi::new("/openapi").url("/api-docs/openapi.json", ApiDoc::openapi()));
    aggregation_router
}
