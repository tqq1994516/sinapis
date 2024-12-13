use axum::async_trait;
use tokio_postgres::Error;
pub use tokio_postgres::{Client, NoTls};

const LOAD_AGE: &str = "LOAD 'age'";
const SET_AGE: &str = "SET search_path = ag_catalog, \"$user\", public";

#[async_trait]
pub trait AgeClientExtend {
    async fn connect_age_extend(pool: &Client) -> Result<&Client, Error>;
}

#[async_trait]
impl AgeClientExtend for Client {
    async fn connect_age_extend(client: &Client) -> Result<&Client, Error> {
        for query in [
            client.simple_query(LOAD_AGE).await,
            client.simple_query(SET_AGE).await,
        ] {
            query?;
        }
        Ok(client)
    }
}
