use std::fmt::Debug;
use neo4rs::Graph;

#[derive(Clone)]
pub struct Neo4jService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for Neo4jService<S>
where
    Req: Send + 'static + Debug,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: Debug,
    Cx: Send + 'static + volo::context::Context,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        if !cx.extensions_mut().get::<Graph>().is_some() {
            let db = Graph::new(
                std::env::var("NEO4J_HOST").unwrap(),
                std::env::var("NEO4J_USER").unwrap(),
                std::env::var("NEO4J_PASSWORD").unwrap()
            ).await.unwrap();
            cx.extensions_mut().insert(db);
        }
        let resp = self.0.call(cx, req).await;
        resp
    }
}

pub struct Neo4jLayer;

impl<S> volo::Layer<S> for Neo4jLayer {
    type Service = Neo4jService<S>;

    fn layer(self, inner: S) -> Self::Service {
        Neo4jService(inner)
    }
}
