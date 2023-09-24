use route::register_route;

mod core;
mod service;
mod route;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::var("ADDR")?.parse()?;
    axum::Server::bind(&address)
        .serve(register_route().await.into_make_service())
        .await?;

    Ok(())
}

