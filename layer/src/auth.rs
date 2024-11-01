use axum::{Router, http::{Request, Response}};
use http_body::Body as HttpBody;
use std::{
    fmt::Debug,
    task::{Poll, Context},
    pin::Pin,
    future::Future,
};
use tower::{util::Either, Layer, Service};
use pin_project::pin_project;

use entity::entities::middleware::Middleware;

pub struct AuthLayer {
    pub auth_service: S1,
    pub unauth_service: S2,
    pub open_apis: Vec<String>,
}

impl<S1, S2> Layer<Router> for AuthLayer {
    type Service = AuthService<S1, S2>;

    fn layer(&self, _: Router) -> Self::Service {
        AuthService {
            auth_service: self.auth_service.clone(),
            unauth_service: self.unauth_service.clone(),
            open_apis: self.open_apis,
        }
    }
}

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    response_future: F,
    #[pin]
    token: Option<String>,
}

impl<F, Response, Error> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response, Error>>,
    Error: Into<anyhow::Error>,
{
    type Output = Result<Response, anyhow::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.response_future.poll(cx) {
            Poll::Ready(result) => {
                return Poll::Ready(result);
            }
            Poll::Pending => {}
        }

        match this.token.poll(cx) {
            Poll::Ready(token) => {
                return Poll::Ready(token);
            }
            Poll::Pending => {}
        }

        Poll::Pending
    }
}

#[derive(Debug, Clone)]
pub struct AuthService<S1, S2> {
    pub auth_service: S1,
    pub unauth_service: S2,
    pub open_apis: Vec<String>,
}

impl<S1, S2, Req, Res> Service<Request<Req>> for AuthService<S1, S2>
where
    S1: Service<Request<Req>, Response = Response<Res>> + Clone + Send + 'static,
    S1::Future: Send + 'static,
    S2: Service<Request<Req>, Response = Response<Res>> + Clone + Send + 'static,
    S2::Future: Send + 'static,
    Req: Send + 'static,
    Res: HttpBody<Data = Bytes> + Default + Send + 'static,
    Res::Error: Into<anyhow::Error>,
{
    type Response = Response<Res>;

    type Error = anyhow::Error;

    type Future = Either<ResponseFuture<S1::Future>, ResponseFuture<S2::Future>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.auth_service
            .poll_ready(cx)
            .ant_then(|_| self.unauth_service.poll_ready(cx))
    }

    async fn call(&self, mut req: Request<Req>) -> Self::Future {
        if req.uri().path().starts_with("/") {
            Either::A(self.auth_service.call(req))
        } else {
            Either::B(self.unauth_service.call(req))
        }
    }
}
