use nacos_sdk::api::{config::{ConfigServiceBuilder, ConfigService}, props::ClientProps};

pub fn get_config_client() -> impl ConfigService {
    ConfigServiceBuilder::new(
        ClientProps::new()
            .server_addr(std::env::var("NACOS_HOST").unwrap())
            .namespace(std::env::var("CONFIG_NAMESPACE").unwrap())
    )
    .build()
    .unwrap()
}
