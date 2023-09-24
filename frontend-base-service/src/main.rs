#![feature(impl_trait_in_assoc_type)]
#![feature(async_fn_in_trait)]
use volo_grpc::server::{Server, ServiceBuilder};
use std::net::SocketAddr;

use volo_gen::frontend_base_service::{
    RouteServer,
    I18nServer,
};
use controller::{
    RouteService,
    I18nService,
};

mod controller;
mod service;
mod helper;

#[volo::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = std::env::var("ADDR")?.parse()?;
    let addr = volo::net::Address::from(addr);

    let user_service = RouteService {};
    let i18n_service = I18nService {};

    Server::new()
        .add_service(ServiceBuilder::new(RouteServer::new(user_service)).build())
        .add_service(ServiceBuilder::new(I18nServer::new(i18n_service)).build())
        .run(addr)
        .await?;
    Ok(())
}
