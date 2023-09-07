use tower::ServiceBuilder;
use tonic::{service::interceptor, transport::Server};
use dapr::dapr::dapr::proto::runtime::v1::app_callback_server::AppCallbackServer;

use layer::postgres::postgres_pool;

pub mod controller;
pub mod service;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::from_filename("share_resources/.env").ok();

    let addr = std::env::var("ADDR")?.parse()?;

    let user_info = controller::UserInfo {};
    let layer = ServiceBuilder::new()
        .layer(interceptor(postgres_pool))
        .into_inner();

    Server::builder()
        .layer(layer)
        .add_service(AppCallbackServer::new(user_info))
        .serve(addr)
        .await?;
    Ok(())
}
