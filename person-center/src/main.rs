mod controller;
mod service;
mod helper;

use volo_grpc::server::{Server, ServiceBuilder};
use std::net::SocketAddr;

use volo_gen::person_center::UserServer;
use layer::postgres::PostgresqlLayer;
use layer::neo4j::Neo4jLayer;

#[volo::main]
async fn main() {
    let addr: SocketAddr = "[::]:8080".parse().unwrap();
    let addr = volo::net::Address::from(addr);

    Server::new()
        .add_service(ServiceBuilder::new(UserServer::new(controller::UserService)).build())
        .layer_front(PostgresqlLayer)
        .layer_front(Neo4jLayer)
        .run(addr)
        .await
        .unwrap();
}
