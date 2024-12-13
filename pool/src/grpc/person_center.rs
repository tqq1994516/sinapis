use bb8::ManageConnection;
use std::net::SocketAddr;
use axum::{BoxError, async_trait};

use volo_gen::person_center::{UserClientBuilder, UserClient};

#[derive(Debug, Clone)]
pub struct PersonCenterGrpcClientManager {
    pub client_addr: SocketAddr,
}

impl PersonCenterGrpcClientManager {
    pub async fn new(host: &str) -> Result<Self, BoxError> {
        let client_addr = host.parse::<SocketAddr>().unwrap();

        Ok(Self { client_addr })
    }
}

#[async_trait]
impl ManageConnection for PersonCenterGrpcClientManager {
    type Connection = UserClient;
    type Error = BoxError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(UserClientBuilder::new("user")
            .address(self.client_addr)
            .build())
    }

    async fn is_valid(&self, _: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}
