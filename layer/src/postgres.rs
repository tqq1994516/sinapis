use tokio::runtime::Runtime;
use tonic::{Request, Status};
use sea_orm::Database;

pub fn postgres_pool(mut req: Request<()>) -> Result<Request<()>, Status> {
    dotenv::dotenv().ok();
    let db = Runtime::new().unwrap().block_on(async {
        Database::connect(std::env::var("DATABASE_URL").unwrap()).await.unwrap()
    });
    req.extensions_mut().insert(db);
    Ok(req)
}
