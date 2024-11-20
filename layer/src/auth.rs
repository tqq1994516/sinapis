use axum::{
    body::{to_bytes, Body},
    http::{
        header::{COOKIE, SET_COOKIE},
        response::Parts,
        HeaderValue, Request, StatusCode,
    },
    middleware::Next,
    response::Response,
};
use bb8_redis::{
    bb8::Pool,
    RedisConnectionManager,
};
use redis::{cmd, AsyncCommands, JsonAsyncCommands};
use serde_json::json;
use chrono::{prelude::*, Duration};
use cookie::{time::Duration as TimeDuration, Cookie};
use entity::entities::{auth::Account, middleware::Claims};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::str::FromStr;

use idgen::NextId;

lazy_static::lazy_static! {
    static ref PRIVATE_KEY: &'static [u8; 64] = include_bytes!("../../private.pem");
    static ref PUBLIC_KEY: &'static [u8; 64] = include_bytes!("../../private.pem");
    static ref AUDIENCE: String = String::from("browser");
    static ref ISSUER: String = String::from("auth");
    static ref ACCESS_TOKEN_NAME: String = String::from("access_token");
    static ref REFRESH_TOKEN_NAME: String = String::from("refresh_token");
    static ref REDIS_JSON_ROOT_PATH: String = String::from("$");
}



#[derive(Debug)]
struct TokenSchema {
    access_token: Option<Claims>,
    refresh_token: Option<Claims>,
}

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let redis_pool = req
        .extensions()
        .get::<Pool<RedisConnectionManager>>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .clone();
    let mut redis_connect = redis_pool.get().await.ok().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    match req.uri().path().contains("/login") {
        true => {
            let res = next.run(req).await;
            let (parts, body) = res.into_parts();
            if parts.status == StatusCode::ACCEPTED {
                let user_res = to_bytes(body, usize::MAX)
                    .await
                    .ok()
                    .ok_or(StatusCode::UNAUTHORIZED)?;
                let user_id = String::from_utf8(user_res.to_vec())
                    .ok()
                    .ok_or(StatusCode::UNAUTHORIZED)?
                    .parse::<i64>()
                    .ok()
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

                let now = Utc::now();
                let snow_id = NextId();

                let mut new_parts = Parts::from(parts);
                let (access_token, access_duration) =
                    get_access_cookie_info(now, snow_id, user_id)?;
                let (refresh_token, refresh_duration, refresh_exp) =
                    get_refresh_cookie_info(now, snow_id, user_id)?;
                new_parts.headers.append(
                    SET_COOKIE,
                    HeaderValue::from_str(
                        &Cookie::build((&*ACCESS_TOKEN_NAME, &access_token))
                            .path("/")
                            .http_only(true)
                            .max_age(access_duration)
                            .build()
                            .to_string(),
                    )
                    .ok()
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
                );
                new_parts.headers.append(
                    SET_COOKIE,
                    HeaderValue::from_str(
                        &Cookie::build((&*ACCESS_TOKEN_NAME, &refresh_token))
                            .path("/")
                            .http_only(true)
                            .max_age(refresh_duration)
                            .build()
                            .to_string(),
                    )
                    .ok()
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
                );

                let user_id = 1;
                let account_raw: String = redis_connect
                    .json_get(&user_id.to_string(), &*REDIS_JSON_ROOT_PATH)
                    .await
                    .ok()
                    .ok_or(StatusCode::FORBIDDEN)?;
                let account = serde_json::from_str::<Vec<Account>>(&account_raw)
                    .ok()
                    .ok_or(StatusCode::FORBIDDEN)?;

                if !account.is_empty() {
                    redis_connect
                        .json_del(&user_id.to_string(), &*REDIS_JSON_ROOT_PATH)
                        .await
                        .ok()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                };

                redis_connect
                    .json_set(&user_id.to_string(), &*REDIS_JSON_ROOT_PATH, &json!(Account {user_id, snow_id, refresh_token, exp: refresh_exp }))
                    .await
                    .ok()
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

                redis_connect
                    .expire_at(&user_id.to_string(), refresh_exp.timestamp())
                    .await
                    .ok()
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

                Ok(Response::from_parts(new_parts, Body::from(user_res)))
            } else {
                Ok(Response::from_parts(parts, body))
            }
        }
        false => {
            let coookies = req.headers().get(COOKIE).ok_or(StatusCode::UNAUTHORIZED)?;

            let mut validation = Validation::new(Algorithm::EdDSA);
            validation.set_audience(&[AUDIENCE.clone()]);
            validation.set_issuer(&[ISSUER.clone()]);
            let cookie_str = coookies.to_str().ok().ok_or(StatusCode::UNAUTHORIZED)?;
            let mut access_token = None;
            let mut refresh_token = None;
            for token in cookie_str.split(";").collect::<Vec<&str>>() {
                let token_jar = Cookie::from_str(token.trim())
                    .ok()
                    .ok_or(StatusCode::FORBIDDEN)?;
                if token_jar.name() == *ACCESS_TOKEN_NAME {
                    access_token = Some(
                        decode(
                            token_jar.value(),
                            &DecodingKey::from_ed_pem(*PUBLIC_KEY)
                                .ok()
                                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
                            &validation,
                        )
                        .ok()
                        .ok_or(StatusCode::UNAUTHORIZED)?
                        .claims,
                    );
                } else if token_jar.name() == *REFRESH_TOKEN_NAME {
                    refresh_token = Some(
                        decode(
                            token_jar.value(),
                            &DecodingKey::from_ed_pem(*PUBLIC_KEY)
                                .ok()
                                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
                            &validation,
                        )
                        .ok()
                        .ok_or(StatusCode::UNAUTHORIZED)?
                        .claims,
                    );
                }
            }

            let token = TokenSchema {
                access_token,
                refresh_token,
            };

            match (token.access_token, token.refresh_token) {
                (None, None) => Err(StatusCode::UNAUTHORIZED),
                (None, Some(ref_t)) => {
                    let account_raw: String = redis_connect
                        .json_get(&ref_t.sub.to_string(), &*REDIS_JSON_ROOT_PATH)
                        .await
                        .ok()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                    let account = serde_json::from_str::<Vec<Account>>(&account_raw)
                        .ok()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

                    match account.is_empty() {
                        false => {
                            let now = Utc::now();
                            let res = next.run(req).await;
                            let (parts, body) = res.into_parts();
                            let mut new_parts = Parts::from(parts);
                            let (access_token, access_duration) =
                                get_access_cookie_info(now, ref_t.snow_id, ref_t.sub)?;
                            new_parts.headers.append(
                                SET_COOKIE,
                                HeaderValue::from_str(
                                    &Cookie::build((&*ACCESS_TOKEN_NAME, &access_token))
                                        .path("/")
                                        .http_only(true)
                                        .max_age(access_duration)
                                        .build()
                                        .to_string(),
                                )
                                .ok()
                                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
                            );
                            Ok(Response::from_parts(new_parts, Body::from(body)))
                        }
                        true => Err(StatusCode::UNAUTHORIZED)
                    }
                }
                (Some(acc_t), None) => {
                    let account_raw: String = redis_connect
                        .json_get(&acc_t.sub.to_string(), &*REDIS_JSON_ROOT_PATH)
                        .await
                        .ok()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                    let account = serde_json::from_str::<Vec<Account>>(&account_raw)
                        .ok()
                        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

                    match account.is_empty() {
                        false => {
                            let now = Utc::now();
                            let res = next.run(req).await;
                            let (parts, body) = res.into_parts();
                            let mut new_parts = Parts::from(parts);
                            let (refresh_token, refresh_duration, refresh_exp) =
                                get_refresh_cookie_info(now, acc_t.snow_id, acc_t.sub)?;
                            new_parts.headers.append(
                                SET_COOKIE,
                                HeaderValue::from_str(
                                    &Cookie::build((&*REFRESH_TOKEN_NAME, &refresh_token))
                                        .path("/")
                                        .http_only(true)
                                        .max_age(refresh_duration)
                                        .build()
                                        .to_string(),
                                )
                                .ok()
                                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
                            );
                            redis_connect
                                .json_set(&acc_t.sub.to_string(), &*REDIS_JSON_ROOT_PATH, &json!(Account {user_id: acc_t.sub, snow_id: acc_t.snow_id, refresh_token, exp: refresh_exp }))
                                .await
                                .ok()
                                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
            
                            redis_connect
                                .expire_at(&acc_t.sub.to_string(), refresh_exp.timestamp())
                                .await
                                .ok()
                                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                            Ok(Response::from_parts(new_parts, Body::from(body)))
                        },
                        true => Err(StatusCode::UNAUTHORIZED),
                    }
                }
                (Some(acc_t), Some(ref_t)) => Ok(next.run(req).await),
            }
        }
    }
}

fn get_access_cookie_info(
    now: DateTime<Utc>,
    snow_id: i64,
    user_id: i64,
) -> Result<(String, TimeDuration), StatusCode> {
    let access_token_dur = Duration::minutes(5);
    let access_claims = Claims {
        snow_id,
        aud: AUDIENCE.to_string(),
        exp: now + access_token_dur,
        iat: now,
        iss: ISSUER.to_string(),
        nbf: now + access_token_dur,
        sub: user_id,
    };
    let access_token = encode(
        &Header::new(Algorithm::EdDSA),
        &access_claims,
        &EncodingKey::from_ed_pem(*PRIVATE_KEY)
            .ok()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .ok()
    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((
        access_token,
        TimeDuration::seconds(access_token_dur.num_seconds()),
    ))
}

fn get_refresh_cookie_info(
    now: DateTime<Utc>,
    snow_id: i64,
    user_id: i64,
) -> Result<(String, TimeDuration, DateTime<Utc>), StatusCode> {
    let refresh_token_dur = Duration::hours(8);
    let refresh_claims = Claims {
        snow_id,
        aud: AUDIENCE.to_string(),
        exp: now + refresh_token_dur,
        iat: now,
        iss: ISSUER.to_string(),
        nbf: now + refresh_token_dur,
        sub: user_id,
    };
    let refresh_token = encode(
        &Header::new(Algorithm::EdDSA),
        &refresh_claims,
        &EncodingKey::from_ed_pem(*PRIVATE_KEY)
            .ok()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .ok()
    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        refresh_token,
        TimeDuration::seconds(refresh_token_dur.num_minutes()),
        now + refresh_token_dur,
    ))
}
