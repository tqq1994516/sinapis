use std::{fmt::Debug, str::FromStr};
use bb8::Pool;
use sea_orm::{Database, DatabaseConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{Config, NoTls};

#[derive(Clone)]
pub struct PostgresqlService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for PostgresqlService<S>
where
    Req: Send + 'static + Debug,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: Debug,
    Cx: Send + 'static + volo::context::Context,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        if cx.extensions_mut().get::<DatabaseConnection>().is_none() {
            let db = Database::connect(std::env::var("DATABASE_URL").unwrap()).await.unwrap();
            cx.extensions_mut().insert(db);
        }
        if cx.extensions_mut().get::<Pool<PostgresConnectionManager<NoTls>>>().is_none() {
            let config = Config::from_str(&std::env::var("DATABASE_URL").unwrap()).unwrap();
            let pg_mgr = PostgresConnectionManager::new(config, NoTls);
            let pool = Pool::builder().build(pg_mgr).await.unwrap();
            cx.extensions_mut().insert(pool);
        }
        let resp = self.0.call(cx, req).await;
        resp
    }
}

pub struct PostgresqlLayer;

impl<S> volo::Layer<S> for PostgresqlLayer {
    type Service = PostgresqlService<S>;

    fn layer(self, inner: S) -> Self::Service {
        PostgresqlService(inner)
    }
}
