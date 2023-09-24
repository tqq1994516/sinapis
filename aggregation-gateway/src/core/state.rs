use std::env;

type DaprClient = dapr::Client<dapr::client::TonicClient>;

pub struct GrpcClientState {
    pub client: DaprClient,
}

impl GrpcClientState {
    pub async fn build() -> Result<GrpcClientState, Box<dyn std::error::Error>> {
        let port: u16 = env::var("DAPR_GRPC_PORT")?.parse()?;
        let address = format!("https://127.0.0.1:{}", port);

        let client = DaprClient::connect(address).await?;
        Ok(Self { client })
    }
}
