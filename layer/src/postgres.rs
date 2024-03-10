use std::fmt::Debug;
use sea_orm::{Database, DatabaseConnection};

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
        if !cx.extensions_mut().get::<DatabaseConnection>().is_some() {
            let db = Database::connect(std::env::var("POSTGRES_URL").unwrap()).await.unwrap();
            cx.extensions_mut().insert(db);
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
