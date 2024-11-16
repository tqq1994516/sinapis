use axum::{
    extract::{FromRequestParts, Extension},
    middleware::from_extractor,
    Router,
    http::{header, StatusCode, request::Parts},
    async_trait,
};
use sea_orm::DatabaseConnection;

// An extractor that performs authorization.
struct RequireAuth;

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        match auth_header {
            Some(auth_header) if token_is_valid(auth_header) => {
                Ok(Self)
            }
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

pub async fn get_sea_orm_connect_extension() -> Extension<DatabaseConnection> {
    Extension(Database::connect(std::env::var("POSTGRES_URL").unwrap()).await.unwrap())
}

fn token_is_valid(token: &str) -> bool {
    // ...
}

// async fn handler() {
//     // If we get here the request has been authorized
// }

// async fn other_handler() {
//     // If we get here the request has been authorized
// }

// let app = Router::new()
//     .route("/", get(handler))
//     .route("/foo", post(other_handler))
//     // The extractor will run before all routes
//     .route_layer(from_extractor::<RequireAuth>());