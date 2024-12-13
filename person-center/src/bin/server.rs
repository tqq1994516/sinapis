use volo_grpc::server::{Server, ServiceBuilder};
use std::net::SocketAddr;

use volo_gen::person_center::{UserServer, UserAttributeServer};
use layer::postgres::PostgresqlLayer;
use person_center::controller::{user::UserService, user_attribute::UserAttributeService};

#[volo::main]
async fn main() {
    let addr = "[::]:8080".parse::<SocketAddr>().unwrap();
    let addr = volo::net::Address::from(addr);
    let project_dir = std::env::current_dir().unwrap();
    dotenv::from_path(project_dir.parent().unwrap().join("entity").join(".env")).unwrap();

    Server::new()
        .add_service(ServiceBuilder::new(UserServer::new(UserService)).build())
        .add_service(ServiceBuilder::new(UserAttributeServer::new(UserAttributeService)).build())
        .layer_front(PostgresqlLayer)
        .run(addr)
        .await
        .unwrap();
}
