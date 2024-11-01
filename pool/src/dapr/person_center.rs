use bb8::ManageConnection;
use dapr::{Client, client::TonicClient};
use axum::{BoxError, async_trait};

pub type DaprClient = Client<TonicClient>;

#[derive(Debug, Clone)]
pub struct GrpcClientManager {
    pub client_addr: String,
}

impl GrpcClientManager {
    pub async fn new() -> Result<Self, BoxError> {
        let client_addr = String::from("https://127.0.0.1");

        Ok(Self { client_addr })
    }
}

#[async_trait]
impl ManageConnection for GrpcClientManager {
    type Connection = DaprClient;
    type Error = BoxError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(DaprClient::connect(String::from(self.client_addr.as_str())).await?)
    }

    async fn is_valid(&self, _: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}
