use crate::db::AppState;
use crate::types::JsonMessage;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use headers::{Cookie, HeaderMapExt};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
}

pub async fn validate_session(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<DiscordUser, (StatusCode, Json<JsonMessage>)> {
    let Some(cookie_header) = headers.typed_get::<Cookie>() else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Missing session cookie".into(),
            }),
        ));
    };

    let Some(session_id) = cookie_header.get("session") else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Missing session ID".into(),
            }),
        ));
    };

    let mut redis_conn = state.redis.get_connection().await.map_err(|e| {
        error!("Failed to get Redis connection: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Database connection error".into(),
            }),
        )
    })?;

    let key = format!("session:{}", session_id);
    let json: redis::RedisResult<String> = redis_conn.as_mut().get(&key).await;

    let Ok(json) = json else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Session not found".into(),
            }),
        ));
    };

    let Ok(user) = serde_json::from_str::<DiscordUser>(&json) else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Invalid user session".into(),
            }),
        ));
    };

    Ok(user)
}

fn create_session_cookie(session_id: &str) -> Result<HeaderValue, (StatusCode, Json<JsonMessage>)> {
    format!(
        "session={}; Max-Age=3600; Path=/; SameSite=None; Secure; HttpOnly",
        session_id
    )
    .parse()
    .map_err(|e| {
        error!("Failed to create cookie header: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Failed to create session".into(),
            }),
        )
    })
}

fn create_logout_cookie() -> Result<HeaderValue, (StatusCode, Json<JsonMessage>)> {
    "session=; Max-Age=0; Path=/; SameSite=None; Secure; HttpOnly"
        .parse()
        .map_err(|e| {
            error!("Failed to create logout cookie header: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Logout failed".into(),
                }),
            )
        })
}

#[instrument(skip(state), fields(user_id))]
pub async fn get_user_from_session(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<DiscordUser, (StatusCode, Json<JsonMessage>)> {
    let user = validate_session(headers, state).await?;
    tracing::Span::current().record("user_id", &user.id);
    Ok(user)
}

#[instrument(skip(state))]
pub async fn start_oauth(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let client_id = &state.config.discord.client_id;
    let redirect_uri = &state.config.discord.redirect_uri;

    let mut url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify",
        client_id, redirect_uri
    );

    if let Some(redirect) = params.get("redirect") {
        url.push_str(&format!("&state={}", urlencoding::encode(redirect)));
    }

    info!("Starting OAuth flow");
    (StatusCode::FOUND, [(axum::http::header::LOCATION, url)]).into_response()
}

#[instrument(skip(state, query), fields(user_id))]
pub async fn handle_callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> impl IntoResponse {
    let client_id = &state.config.discord.client_id;
    let client_secret = &state.config.discord.client_secret;
    let redirect_uri = &state.config.discord.redirect_uri;

    let form = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", &query.code),
        ("redirect_uri", redirect_uri.as_str()),
    ];

    let token_res = state
        .http_client
        .post("https://discord.com/api/oauth2/token")
        .form(&form)
        .send()
        .await;

    let Ok(res) = token_res else {
        error!("Failed to exchange OAuth code for token");
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonMessage {
                message: "Failed to exchange token".into(),
            }),
        )
            .into_response();
    };

    let Ok(token_json) = res.json::<serde_json::Value>().await else {
        error!("Invalid token response from Discord");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Invalid token response".into(),
            }),
        )
            .into_response();
    };

    let Some(access_token) = token_json["access_token"].as_str() else {
        error!("Access token not found in Discord response");
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Access token not found".into(),
            }),
        )
            .into_response();
    };

    let user_res = state
        .http_client
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;

    let Ok(user_res) = user_res else {
        error!("Failed to fetch user info from Discord");
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonMessage {
                message: "Failed to fetch user".into(),
            }),
        )
            .into_response();
    };

    let Ok(user) = user_res.json::<DiscordUser>().await else {
        error!("Failed to parse user info from Discord");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Failed to parse user".into(),
            }),
        )
            .into_response();
    };

    tracing::Span::current().record("user_id", &user.id);

    if let Err(e) = sqlx::query("UPDATE timezones SET username = $1 WHERE user_id = $2")
        .bind(&user.username)
        .bind(&user.id)
        .execute(&state.db)
        .await
    {
        warn!("Failed to update username for user {}: {}", user.id, e);
    }

    let session_id = Uuid::now_v7().to_string();

    let mut redis_conn = match state.redis.get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get Redis connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Database connection error".into(),
                }),
            )
                .into_response();
        }
    };

    let user_json = match serde_json::to_string(&user) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize user data: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Failed to create session".into(),
                }),
            )
                .into_response();
        }
    };

    if let Err(e) = redis_conn
        .as_mut()
        .set_ex::<_, _, ()>(format!("session:{}", session_id), user_json, 3600)
        .await
    {
        error!("Failed to store session in Redis: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Failed to create session".into(),
            }),
        )
            .into_response();
    }

    let redirect_target = match &query.state {
        Some(s) => urlencoding::decode(s)
            .map(|s| s.into_owned())
            .unwrap_or_else(|e| {
                warn!("Failed to decode state parameter '{}': {}", s, e);
                "/".to_string()
            }),
        None => {
            info!(user_id = %user.id, username = %user.username, "User logged in via API");

            let mut headers = HeaderMap::new();
            let cookie_value = match create_session_cookie(&session_id) {
                Ok(value) => value,
                Err(err) => return err.into_response(),
            };
            headers.insert("Set-Cookie", cookie_value);

            return (
                StatusCode::OK,
                headers,
                Json(json!({
                    "message": "Login successful",
                    "user": {
                        "id": user.id,
                        "username": user.username,
                        "discriminator": user.discriminator,
                        "avatar": user.avatar
                    },
                    "session_id": session_id
                })),
            )
                .into_response();
        }
    };

    let mut headers = HeaderMap::new();
    let cookie_value = match create_session_cookie(&session_id) {
        Ok(value) => value,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, headers).into_response();
        }
    };
    headers.insert("Set-Cookie", cookie_value);
    let location_value = match redirect_target.parse() {
        Ok(value) => value,
        Err(e) => {
            error!(
                "Failed to parse redirect target '{}': {}",
                redirect_target, e
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, headers).into_response();
        }
    };
    headers.insert(axum::http::header::LOCATION, location_value);

    info!(user_id = %user.id, username = %user.username, "User logged in successfully");
    (StatusCode::FOUND, headers).into_response()
}

#[instrument(skip(state))]
pub async fn me(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    match get_user_from_session(&headers, &state).await {
        Ok(user) => {
            let result = sqlx::query("SELECT timezone FROM timezones WHERE user_id = $1")
                .bind(&user.id)
                .fetch_optional(&state.db)
                .await;

            match result {
                Ok(Some(row)) => {
                    let timezone: String = row.get("timezone");
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({
                            "user": user,
                            "timezone": timezone
                        })),
                    )
                        .into_response()
                }
                Ok(None) => (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "user": user,
                        "timezone": null
                    })),
                )
                    .into_response(),
                Err(e) => {
                    error!("Database error while fetching timezone: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(JsonMessage {
                            message: "Failed to fetch timezone".into(),
                        }),
                    )
                        .into_response()
                }
            }
        }
        Err(err) => (err.0, err.1).into_response(),
    }
}

#[instrument(skip(state))]
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(cookie_header) = headers.typed_get::<Cookie>() else {
        return (
            StatusCode::OK,
            Json(JsonMessage {
                message: "Already logged out".into(),
            }),
        )
            .into_response();
    };

    let Some(session_id) = cookie_header.get("session") else {
        return (
            StatusCode::OK,
            Json(JsonMessage {
                message: "Already logged out".into(),
            }),
        )
            .into_response();
    };

    let mut redis_conn = match state.redis.get_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get Redis connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Database connection error".into(),
                }),
            )
                .into_response();
        }
    };

    let key = format!("session:{}", session_id);
    let _: redis::RedisResult<()> = redis_conn.as_mut().del(&key).await;

    let mut headers = HeaderMap::new();
    let logout_cookie = match create_logout_cookie() {
        Ok(value) => value,
        Err(err) => return err.into_response(),
    };
    headers.insert("Set-Cookie", logout_cookie);

    info!("User logged out successfully");
    (
        StatusCode::OK,
        headers,
        Json(JsonMessage {
            message: "Logged out successfully".into(),
        }),
    )
        .into_response()
}
