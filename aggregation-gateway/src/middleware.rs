use axum::{extract::ConnectInfo, response::Response, BoxError};
use bytes::Bytes;
use chrono::{Duration, Utc};
use cookie::{Cookie, CookieJar, Key};
use forwarded_header_value::{ForwardedHeaderValue, Identifier};
use futures::future::BoxFuture;
use http::{
    header::{COOKIE, FORWARDED, SET_COOKIE, USER_AGENT},
    HeaderMap, Request,
};
use http_body::Body as HttpBody;
use std::{
    convert::Infallible,
    net::{IpAddr, SocketAddr},
    task::{Context, Poll},
};
use tower_service::Service;

#[derive(Clone)]
pub struct SessionService<S> {
    pub(crate) inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for SessionService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    Infallible: From<<S as Service<Request<ReqBody>>>::Error>,
    ResBody: HttpBody<Data = Bytes> + Default + Send + std::fmt::Debug + 'static,
    ResBody::Error: Into<BoxError>,
{
    type Response = Response<ResBody>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let not_ready_inner = self.inner.clone();
        let mut ready_inner = std::mem::replace(&mut self.inner, not_ready_inner);

        Box::pin(async move {
            let ip_user_agent = get_ips_hash(&req);

            let cookies = get_cookies(req.headers());

            let snow_id = get_headers_and_key(cookies, &ip_user_agent).await;

            println!("snow_id:{:#?}", snow_id);

            let current_time = Utc::now();

            let mut response = ready_inner.call(req).await?;

            tracing::trace!("Session id: {:#?}", response.body());

            set_headers(response.headers_mut(), &ip_user_agent);

            Ok(response)
        })
    }
}

const X_FORWARDED_FOR: &str = "x-forwarded-for";
const X_REAL_IP: &str = "x-real-ip";

fn get_ips_hash<T>(req: &Request<T>) -> String {
    let headers = req.headers();

    let ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_default();

    let x_forward_for_ip = headers
        .get(X_FORWARDED_FOR)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.split(",").find_map(|s| s.trim().parse::<IpAddr>().ok()))
        .map(|ip| ip.to_string())
        .unwrap_or_default();

    let forwarder_ip = headers
        .get_all(FORWARDED)
        .iter()
        .find_map(|hv| {
            hv.to_str()
                .ok()
                .and_then(|s| ForwardedHeaderValue::from_forwarded(s).ok())
                .and_then(|f| {
                    f.iter()
                        .filter_map(|fs| fs.forwarded_for.as_ref())
                        .find_map(|ff| match ff {
                            Identifier::SocketAddr(a) => Some(a.ip()),
                            Identifier::IpAddr(ip) => Some(*ip),
                            _ => None,
                        })
                })
        })
        .map(|ip| ip.to_string())
        .unwrap_or_default();

    let real_ip = headers
        .get(X_REAL_IP)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.parse::<IpAddr>().ok())
        .map(|ip| ip.to_string())
        .unwrap_or_default();

    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|hv| hv.to_str().ok())
        .map(|ua| ua.to_string())
        .unwrap_or_default();

    format!(
        "{};{};{};{};{}",
        ip, x_forward_for_ip, forwarder_ip, real_ip, user_agent
    )
}

pub(crate) fn get_cookies(headers: &HeaderMap) -> CookieJar {
    let mut jar = CookieJar::new();

    let cookie_iter = headers
        .get_all(COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());

    for cookie in cookie_iter {
        jar.add_original(cookie);
    }

    jar
}

pub(crate) trait CookiesExt {
    fn get_cookie(&self, name: &str, key: Option<&Key>, message: String)
        -> Option<Cookie<'static>>;
    fn add_cookie(&mut self, cookie: Cookie<'static>, key: &Option<Key>, message: String);
}

impl CookiesExt for CookieJar {
    fn get_cookie(
        &self,
        name: &str,
        key: Option<&Key>,
        message: String,
    ) -> Option<Cookie<'static>> {
        self.get(name).cloned()
    }

    fn add_cookie(&mut self, cookie: Cookie<'static>, key: &Option<Key>, message: String) {
        self.add(cookie);
    }
}

pub async fn get_headers_and_key(cookies: CookieJar, ip_user_agent: &str) -> Option<i64> {
    let value = cookies
        .get_cookie(
            "test",
            Some(&Key::from(&(0..64).collect::<Vec<u8>>())),
            ip_user_agent.to_owned(),
        )
        .and_then(|c| c.to_string().parse::<i64>().ok());

    value
}

fn set_cookies(jar: CookieJar, headers: &mut HeaderMap) {
    for cookie in jar.delta() {
        if let Ok(header_value) = cookie.encoded().to_string().parse() {
            headers.append(SET_COOKIE, header_value);
        }
    }
}

fn create_access_cookie<'a>(value: String) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(("app_access_token", value))
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(cookie::SameSite::Strict);

    // cookie_builder = cookie_builder.domain("");

    let expires = Duration::minutes(5);
    cookie_builder = cookie_builder.expires(Some(
        (std::time::SystemTime::now()
            + std::time::Duration::new(
                expires.num_seconds() as u64,
                (expires.num_nanoseconds().unwrap_or(0) % 1_000_000_000) as u32,
            ))
        .into(),
    ));

    cookie_builder.build()
}

fn create_refresh_cookie<'a>(value: String) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(("app_refresh_token", value))
        .path("/")
        .secure(false)
        .http_only(false)
        .same_site(cookie::SameSite::Strict);

    // cookie_builder = cookie_builder.domain("");

    let expires = Duration::days(1);
    cookie_builder = cookie_builder.expires(Some(
        (std::time::SystemTime::now()
            + std::time::Duration::new(
                expires.num_seconds() as u64,
                (expires.num_nanoseconds().unwrap_or(0) % 1_000_000_000) as u32,
            ))
        .into(),
    ));

    cookie_builder.build()
}

pub(crate) fn set_headers(headers: &mut HeaderMap, ip_user_agent: &str) {
    let mut cookies = CookieJar::new();

    cookies.add_cookie(
        create_access_cookie(String::from("1")),
        &None,
        ip_user_agent.to_owned(),
    );

    cookies.add_cookie(
        create_refresh_cookie(String::from("2")),
        &None,
        ip_user_agent.to_owned(),
    );

    set_cookies(cookies, headers);
}
