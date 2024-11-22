use axum::{
    body::{to_bytes, Body},
    extract::{Extension, Request, State},
    http::{
        header::{COOKIE, SET_COOKIE},
        response::Parts,
        HeaderMap, HeaderValue, StatusCode, Uri,
    },
    middleware::Next,
    response::{IntoResponse, Response},
};
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use chrono::{prelude::*, Duration};
use cookie::{time::Duration as TimeDuration, Cookie};
use entity::entities::{auth::Account, middleware::Claims};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use redis::{AsyncCommands, JsonAsyncCommands};
use serde_json::{json, Value};
use std::str::FromStr;

use entity::entities::state::OpenApiState;
use idgen::NextId;

lazy_static::lazy_static! {
    static ref PRIVATE_KEY: EncodingKey = EncodingKey::from_ed_pem(include_bytes!("../../private.pem")).unwrap();
    static ref PUBLIC_KEY: DecodingKey = DecodingKey::from_ed_pem(include_bytes!("../../private.pem")).unwrap();
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

pub async fn auth_middleware(
    open_api_state: State<OpenApiState>,
    redis_pool: Extension<Pool<RedisConnectionManager>>,
    uri: Uri,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {

    let mut redis_connect = match redis_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("redis pool connect failed.{}", e),
            )
                .into_response()
        }
    };

    match open_api_state
        .0
        .openapi
        .iter()
        .any(|re| re.is_match(uri.path()))
    {
        true => {
            let res = next.run(request).await;
            let (parts, body) = res.into_parts();
            if parts.status == StatusCode::ACCEPTED {
                let user_res = match to_bytes(body, usize::MAX).await {
                    Ok(user) => user,
                    Err(_) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "body parse failed.")
                            .into_response()
                    }
                };
                let user_id_str = match String::from_utf8(user_res.to_vec()) {
                    Ok(user) => user,
                    Err(_) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "user id parse failed.")
                            .into_response()
                    }
                };

                let user_id = match user_id_str.parse::<i64>() {
                    Ok(user_id) => user_id,
                    Err(_) => {
                        return (StatusCode::INTERNAL_SERVER_ERROR, "user id parse failed.")
                            .into_response()
                    }
                };

                let now = Utc::now();
                let snow_id = NextId();

                let mut new_parts = Parts::from(parts);
                let (access_token, access_duration) =
                    match get_access_cookie_info(now, snow_id, user_id) {
                        Ok(result) => result,
                        Err(resp) => return resp,
                    };
                let (refresh_token, refresh_duration, refresh_exp) =
                    match get_refresh_cookie_info(now, snow_id, user_id) {
                        Ok(result) => result,
                        Err(resp) => return resp,
                    };

                let access_cookie = match HeaderValue::from_str(
                    &Cookie::build((&*ACCESS_TOKEN_NAME, &access_token))
                        .path("/")
                        .http_only(true)
                        .max_age(access_duration)
                        .build()
                        .to_string(),
                ) {
                    Ok(acc_c) => acc_c,
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "access cookie build failed.",
                        )
                            .into_response()
                    }
                };

                let refresh_cookie = match HeaderValue::from_str(
                    &Cookie::build((&*REFRESH_TOKEN_NAME, &refresh_token))
                        .path("/")
                        .http_only(true)
                        .max_age(refresh_duration)
                        .build()
                        .to_string(),
                ) {
                    Ok(ref_c) => ref_c,
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "refresh cookie build failed.",
                        )
                            .into_response()
                    }
                };

                new_parts.headers.append(SET_COOKIE, access_cookie);
                new_parts.headers.append(SET_COOKIE, refresh_cookie);

                let user_id = 1;
                if let Ok(account_raw) = redis_connect
                    .json_get::<&str, &str, String>(&user_id.to_string(), &*REDIS_JSON_ROOT_PATH)
                    .await
                {
                    let account = match serde_json::from_str::<Vec<Account>>(&account_raw) {
                        Ok(account) => account,
                        Err(_) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "parse redis info failed.",
                            )
                                .into_response()
                        }
                    };

                    if !account.is_empty() {
                        if let Err(_) = redis_connect
                            .json_del::<&str, &str, String>(
                                &user_id.to_string(),
                                &*REDIS_JSON_ROOT_PATH,
                            )
                            .await
                        {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "delete redis info failed.",
                            )
                                .into_response();
                        };
                    };
                };

                if let Err(_) = redis_connect
                    .json_set::<&str, &str, Value, String>(
                        &user_id.to_string(),
                        &*REDIS_JSON_ROOT_PATH,
                        &json!(Account {
                            user_id,
                            snow_id,
                            refresh_token,
                            exp: refresh_exp
                        }),
                    )
                    .await
                {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "set redis info failed.")
                        .into_response();
                }

                if let Err(_) = redis_connect
                    .expire_at::<&str, i64>(&user_id.to_string(), refresh_exp.timestamp())
                    .await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "expire redis info failed.",
                    )
                        .into_response();
                };
                Response::from_parts(new_parts, Body::from(user_res))
            } else {
                Response::from_parts(parts, body)
            }
        }
        false => {
            let cookies = match headers.get(COOKIE) {
                Some(cookies) => cookies,
                None => return (StatusCode::UNAUTHORIZED, "Unauthorized.").into_response(),
            };

            let mut validation = Validation::new(Algorithm::EdDSA);
            validation.set_audience(&[AUDIENCE.clone()]);
            validation.set_issuer(&[ISSUER.clone()]);
            let cookie_str = match cookies.to_str() {
                Ok(cookie) => cookie,
                Err(_) => {
                    return (StatusCode::UNAUTHORIZED, "Unauthorized, cookie error.")
                        .into_response()
                }
            };
            let mut access_token = None;
            let mut refresh_token = None;

            for token in cookie_str.split(";").collect::<Vec<&str>>() {
                let token_jar = match Cookie::from_str(token.trim()) {
                    Ok(cookie) => cookie,
                    Err(_) => {
                        return (
                            StatusCode::UNAUTHORIZED,
                            "Unauthorized, cookie split error.",
                        )
                            .into_response()
                    }
                };

                if token_jar.name() == *ACCESS_TOKEN_NAME {
                    access_token =
                        match decode::<Claims>(token_jar.value(), &*PUBLIC_KEY, &validation) {
                            Ok(token) => Some(token.claims),
                            Err(_) => {
                                return (
                                    StatusCode::UNAUTHORIZED,
                                    "Unauthorized, access token parse error.",
                                )
                                    .into_response()
                            }
                        };
                } else if token_jar.name() == *REFRESH_TOKEN_NAME {
                    refresh_token =
                        match decode::<Claims>(token_jar.value(), &*PUBLIC_KEY, &validation) {
                            Ok(token) => Some(token.claims),
                            Err(_) => {
                                return (
                                    StatusCode::UNAUTHORIZED,
                                    "Unauthorized, refresh token parse error.",
                                )
                                    .into_response()
                            }
                        };
                }
            }

            let token = TokenSchema {
                access_token,
                refresh_token,
            };

            match (token.access_token, token.refresh_token) {
                (None, None) => (StatusCode::UNAUTHORIZED, "Unauthorized.").into_response(),
                (None, Some(ref_t)) => {
                    match redis_connect
                        .json_get::<&str, &str, String>(
                            &ref_t.sub.to_string(),
                            &*REDIS_JSON_ROOT_PATH,
                        )
                        .await
                    {
                        Err(_) => (StatusCode::UNAUTHORIZED, "Unauthorized.").into_response(),
                        Ok(account) => {
                            let account = match serde_json::from_str::<Vec<Account>>(&account) {
                                Ok(account) => account,
                                Err(_) => {
                                    return (StatusCode::INTERNAL_SERVER_ERROR, "Unauthorized.")
                                        .into_response()
                                }
                            };

                            match account.is_empty() {
                                true => (StatusCode::UNAUTHORIZED, "Unauthorized.").into_response(),
                                false => {
                                    let now = Utc::now();
                                    if account.first().unwrap().exp.timestamp() < now.timestamp() {
                                        return (StatusCode::UNAUTHORIZED, "Unauthorized.")
                                            .into_response();
                                    };
                                    let res = next.run(request).await;
                                    let (parts, body) = res.into_parts();
                                    let mut new_parts = Parts::from(parts);
                                    let (access_token, access_duration) =
                                        match get_access_cookie_info(now, ref_t.snow_id, ref_t.sub)
                                        {
                                            Ok(result) => result,
                                            Err(resp) => return resp,
                                        };

                                    let access_cookie = match HeaderValue::from_str(
                                        &Cookie::build((&*ACCESS_TOKEN_NAME, &access_token))
                                            .path("/")
                                            .http_only(true)
                                            .max_age(access_duration)
                                            .build()
                                            .to_string(),
                                    ) {
                                        Ok(acc_c) => acc_c,
                                        Err(_) => {
                                            return (
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                "access cookie build failed.",
                                            )
                                                .into_response()
                                        }
                                    };

                                    new_parts.headers.append(SET_COOKIE, access_cookie);
                                    Response::from_parts(new_parts, Body::from(body))
                                }
                            }
                        }
                    }
                }
                (Some(acc_t), None) => {
                    match redis_connect
                        .json_get::<&str, &str, String>(
                            &acc_t.sub.to_string(),
                            &*REDIS_JSON_ROOT_PATH,
                        )
                        .await
                    {
                        Err(_) => (StatusCode::UNAUTHORIZED, "Unauthorized.").into_response(),
                        Ok(account) => {
                            let account = match serde_json::from_str::<Vec<Account>>(&account) {
                                Ok(account) => account,
                                Err(_) => {
                                    return (StatusCode::INTERNAL_SERVER_ERROR, "Unauthorized.")
                                        .into_response()
                                }
                            };

                            match account.is_empty() {
                                true => (StatusCode::UNAUTHORIZED, "Unauthorized.").into_response(),
                                false => {
                                    let now = Utc::now();
                                    if account.first().unwrap().exp.timestamp() < now.timestamp() {
                                        return (StatusCode::UNAUTHORIZED, "Unauthorized.")
                                            .into_response();
                                    };
                                    let res = next.run(request).await;
                                    let (parts, body) = res.into_parts();
                                    let mut new_parts = Parts::from(parts);
                                    let (refresh_token, refresh_duration, refresh_exp) =
                                        match get_refresh_cookie_info(now, acc_t.snow_id, acc_t.sub)
                                        {
                                            Ok(result) => result,
                                            Err(resp) => return resp,
                                        };

                                    let refresh_cookie = match HeaderValue::from_str(
                                        &Cookie::build((&*REFRESH_TOKEN_NAME, &refresh_token))
                                            .path("/")
                                            .http_only(true)
                                            .max_age(refresh_duration)
                                            .build()
                                            .to_string(),
                                    ) {
                                        Ok(acc_c) => acc_c,
                                        Err(_) => {
                                            return (
                                                StatusCode::INTERNAL_SERVER_ERROR,
                                                "access cookie build failed.",
                                            )
                                                .into_response()
                                        }
                                    };
                                    new_parts.headers.append(SET_COOKIE, refresh_cookie);

                                    if let Err(_) = redis_connect
                                        .json_set::<&str, &str, Value, String>(
                                            &acc_t.sub.to_string(),
                                            &*REDIS_JSON_ROOT_PATH,
                                            &json!(Account {
                                                user_id: acc_t.sub,
                                                snow_id: acc_t.snow_id,
                                                refresh_token,
                                                exp: refresh_exp
                                            }),
                                        )
                                        .await
                                    {
                                        return (
                                            StatusCode::INTERNAL_SERVER_ERROR,
                                            "set redis info failed.",
                                        )
                                            .into_response();
                                    }

                                    if let Err(_) = redis_connect
                                        .expire_at::<&str, i64>(
                                            &acc_t.sub.to_string(),
                                            refresh_exp.timestamp(),
                                        )
                                        .await
                                    {
                                        return (
                                            StatusCode::INTERNAL_SERVER_ERROR,
                                            "expire redis info failed.",
                                        )
                                            .into_response();
                                    };
                                    Response::from_parts(new_parts, Body::from(body))
                                }
                            }
                        }
                    }
                }
                (Some(acc_t), Some(_)) => {
                    match redis_connect
                        .json_get::<&str, &str, String>(
                            &acc_t.sub.to_string(),
                            &*REDIS_JSON_ROOT_PATH,
                        )
                        .await
                    {
                        Err(_) => (StatusCode::UNAUTHORIZED, "Unauthorized.").into_response(),
                        Ok(account) => {
                            let account = match serde_json::from_str::<Vec<Account>>(&account) {
                                Ok(account) => account,
                                Err(_) => {
                                    return (StatusCode::INTERNAL_SERVER_ERROR, "Unauthorized.")
                                        .into_response()
                                }
                            };

                            match account.is_empty() {
                                true => (StatusCode::UNAUTHORIZED, "Unauthorized.").into_response(),
                                false => {
                                    let now = Utc::now();
                                    if account.first().unwrap().exp.timestamp() < now.timestamp() {
                                        return (StatusCode::UNAUTHORIZED, "Unauthorized.")
                                            .into_response();
                                    };
                                    next.run(request).await
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_access_cookie_info(
    now: DateTime<Utc>,
    snow_id: i64,
    user_id: i64,
) -> Result<(String, TimeDuration), Response> {
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
        &*PRIVATE_KEY,
    )
    .ok()
    .ok_or(
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "access token parse failed.",
        )
            .into_response(),
    )?;
    Ok((
        access_token,
        TimeDuration::seconds(access_token_dur.num_seconds()),
    ))
}

fn get_refresh_cookie_info(
    now: DateTime<Utc>,
    snow_id: i64,
    user_id: i64,
) -> Result<(String, TimeDuration, DateTime<Utc>), Response> {
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
        &*PRIVATE_KEY,
    )
    .ok()
    .ok_or(
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "refresh token parse failed.",
        )
            .into_response(),
    )?;

    Ok((
        refresh_token,
        TimeDuration::seconds(refresh_token_dur.num_minutes()),
        now + refresh_token_dur,
    ))
}
