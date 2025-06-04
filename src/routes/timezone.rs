use crate::db::AppState;
use crate::routes::auth::DiscordUser;
use crate::types::JsonMessage;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Form, Json,
};
use chrono_tz::Tz;
use headers::{Cookie, HeaderMapExt};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use tracing::error;

#[derive(Serialize)]
pub struct TimezoneResponse {
    user: UserInfo,
    timezone: String,
}

#[derive(Serialize)]
struct MinimalUserInfo {
    username: String,
    timezone: String,
}

#[derive(Serialize)]
pub struct UserInfo {
    id: String,
    username: String,
}

#[derive(Deserialize)]
pub struct GetQuery {
    id: String,
}

#[derive(Deserialize)]
pub struct SetQuery {
    timezone: String,
}

pub async fn get_timezone(
    State(state): State<AppState>,
    Query(query): Query<GetQuery>,
) -> impl IntoResponse {
    let row = sqlx::query("SELECT username, timezone FROM timezones WHERE user_id = $1")
        .bind(&query.id)
        .fetch_optional(&state.db)
        .await;

    match row {
        Ok(Some(record)) => {
            let response = TimezoneResponse {
                user: UserInfo {
                    id: query.id,
                    username: record.get("username"),
                },
                timezone: record.get("timezone"),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(JsonMessage {
                message: "User not found".into(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Database error".into(),
            }),
        )
            .into_response(),
    }
}

pub async fn list_timezones(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query("SELECT user_id, username, timezone FROM timezones")
        .fetch_all(&state.db)
        .await;

    match rows {
        Ok(data) => {
            let mut result = HashMap::new();
            for r in data {
                result.insert(
                    r.get::<String, _>("user_id"),
                    MinimalUserInfo {
                        username: r.get("username"),
                        timezone: r.get("timezone"),
                    },
                );
            }
            (StatusCode::OK, Json(result)).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Failed to fetch list".into(),
            }),
        )
            .into_response(),
    }
}

pub async fn delete_timezone(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(cookie_header) = headers.typed_get::<Cookie>() else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Missing session cookie".into(),
            }),
        )
            .into_response();
    };

    let Some(session_id) = cookie_header.get("session") else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Missing session ID".into(),
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
    let json: redis::RedisResult<String> = redis_conn.get(&key).await;

    let Ok(json) = json else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Session not found".into(),
            }),
        )
            .into_response();
    };

    let Ok(user) = serde_json::from_str::<DiscordUser>(&json) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Invalid user session".into(),
            }),
        )
            .into_response();
    };

    let result = sqlx::query("DELETE FROM timezones WHERE user_id = $1")
        .bind(&user.id)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(JsonMessage {
                message: "Timezone deleted".into(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Delete failed".into(),
            }),
        )
            .into_response(),
    }
}

pub async fn set_timezone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(query): Form<SetQuery>,
) -> impl IntoResponse {
    let Some(cookie_header) = headers.typed_get::<Cookie>() else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Missing session cookie".into(),
            }),
        )
            .into_response();
    };

    let Some(session_id) = cookie_header.get("session") else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Missing session ID".into(),
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
    let json: redis::RedisResult<String> = redis_conn.get(&key).await;

    let Ok(json) = json else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Session not found".into(),
            }),
        )
            .into_response();
    };

    let Ok(user) = serde_json::from_str::<DiscordUser>(&json) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Invalid user session".into(),
            }),
        )
            .into_response();
    };

    let tz_input = query.timezone.trim();
    if tz_input.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonMessage {
                message: "Timezone is required".into(),
            }),
        )
            .into_response();
    }

    if tz_input.parse::<Tz>().is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonMessage {
                message: "Invalid timezone".into(),
            }),
        )
            .into_response();
    }

    let result = sqlx::query(
        r#"
        INSERT INTO timezones (user_id, username, timezone)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id) DO UPDATE
        SET username = EXCLUDED.username, timezone = EXCLUDED.timezone
        "#,
    )
    .bind(&user.id)
    .bind(&user.username)
    .bind(tz_input)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(JsonMessage {
                message: "Timezone saved".into(),
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Database error".into(),
            }),
        )
            .into_response(),
    }
}
