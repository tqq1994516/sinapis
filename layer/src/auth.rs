use axum::{body::Body, http::{header::{COOKIE, SET_COOKIE}, HeaderValue, Request, Response, StatusCode}, middleware::Next, response::Response};
use cookie::{time::Duration as TimeDuration, Cookie};
use entity::entities::middleware::Claims;
use chrono::{prelude::*, Duration};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::{str::FromStr, time::Duration as CoreDuration};
use sea_orm::DatabaseConnection;

lazy_static::lazy_static! {
    static ref PRIVATE_KEY: &[u8] = include_bytes!("../../private.pem");
    static ref PUBLIC_KEY: &[u8] = include_bytes!("../../private.pem");
    static ref AUDIENCE: String = String::from("browser");
    static ref ISSUER: String = String::from("auth");
    static ref ACCESS_TOKEN_NAME: String = String::from("access_token");
    static ref REFRESH_TOKEN_NAME: String = String::from("refresh_token");
}

#[derive(Debug)]
struct TokenSchema {
    access_token: Option<Claims>,
    refresh_token: Option<Claims>,
}

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    match req.uri().path().contains("/login") {
        true => {
            let pg_connect = req.extensions().get::<DatabaseConnection>().ok_or(StatusCode::FORBIDDEN)?;
            let mut res = next.run(req).await;
            if res.status() == StatusCode::ACCEPTED {
                let user: String = res.body().into();
                let now = Utc::now();
                let access_token_dur = Duration::minutes(5);
                let refresh_token_dur = Duration::hours(8);
                let access_claims = Claims {
                    aud: AUDIENCE,
                    exp: now + access_token_dur,
                    iat: now,
                    iss: ISSUER,
                    nbf: now + access_token_dur,
                    sub: user,
                };
                let refresh_claims = Claims {
                    aud: AUDIENCE,
                    exp: now + refresh_token_dur,
                    iat: now,
                    iss: ISSUER,
                    nbf: now + refresh_token_dur,
                    sub: user,
                };
                let access_token = encode(
                    &Header::new(Algorithm::EdDSA),
                    &access_claims,
                    &EncodingKey::from_ed_pem(&PRIVATE_KEY).unwrap(),
                )
                .unwrap();
                let refresh_token = encode(
                    &Header::new(Algorithm::EdDSA),
                    &access_claims,
                    &EncodingKey::from_ed_pem(&PRIVATE_KEY).unwrap(),
                )
                .unwrap();
                
                let access_token_cookie = Cookie::build((&ACCESS_TOKEN_NAME, &access_token))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .max_age(TimeDuration::seconds(access_token_dur.num_seconds()))
                    .build();
                let refresh_token_cookie = Cookie::build((&REFRESH_TOKEN_NAME, &refresh_token))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .max_age(TimeDuration::seconds(refresh_token_dur.num_minutes()))
                    .build();
                res.headers_mut()
                    .append(
                        SET_COOKIE,
                        HeaderValue::from_str(&access_token_cookie.to_string())
                            .unwrap()
                );
                res.headers_mut()
                    .append(
                        SET_COOKIE,
                        HeaderValue::from_str(&refresh_token_cookie.to_string())
                            .unwrap()
                )
            }
            Ok(res)
        },
        false => {
            let token = req.headers()
                .get(COOKIE)
                .map(|cookie| {
                    let mut validation = Validation::new(Algorithm::EdDSA);
                    validation.set_audience(&[&AUDIENCE]);
                    validation.set_issuer(&[&ISSUER]);
                    let cookie_str = cookie.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
                    let mut access_token = None;
                    let mut refresh_token = None;
                    for token in cookie_str.split(";").collect::<Vec<&str>>() {
                        let token_jar = Cookie::from_str(token.trim()).map_err(|_| StatusCode::UNAUTHORIZED)?;
                        if token_jar.name() == &ACCESS_TOKEN_NAME {
                            access_token = Some(decode(
                                token_jar.value(),
                                &DecodingKey::from_ed_pem(&[&PUBLIC_KEY]),
                                &validation
                            ).map_err(|_| StatusCode::UNAUTHORIZED)?
                            .claims);
                        } else if token_jar.name() == &REFRESH_TOKEN_NAME {
                            refresh_token = Some(decode(
                                token_jar.value(),
                                &DecodingKey::from_ed_pem(&[&PUBLIC_KEY]),
                                &validation
                            ).map_err(|_| StatusCode::UNAUTHORIZED)?
                            .claims);
                        }
                    }
                    TokenSchema {
                        access_token,
                        refresh_token,
                    }
                })
                .ok_or(StatusCode::UNAUTHORIZED)?;
            
            match token. {
                true => Ok(next.run(req).await),
                false => Ok(StatusCode::FORBIDDEN),
            }
        },
    }
}