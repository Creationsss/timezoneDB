use crate::db::AppState;
use crate::routes::v1::auth::validate_session;
use crate::types::{GetQuery, JsonMessage, MinimalUserInfo, SetQuery, TimezoneResponse, UserInfo};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Form, Json,
};
use chrono_tz::Tz;
use sqlx::Row;
use std::collections::HashMap;
use tracing::error;

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
        Err(e) => {
            error!("Failed to fetch timezone: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Database error".into(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn list_timezones(State(state): State<AppState>) -> impl IntoResponse {
    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT user_id, username, timezone FROM timezones",
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(data) => {
            let mut result = HashMap::with_capacity(data.len());
            for (user_id, username, timezone) in data {
                result.insert(user_id, MinimalUserInfo { username, timezone });
            }
            (StatusCode::OK, Json(result)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch timezone list: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Failed to fetch list".into(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn delete_timezone(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user = match validate_session(&headers, &state).await {
        Ok(user) => user,
        Err(err) => return err.into_response(),
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
        Err(e) => {
            error!("Failed to delete timezone: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Delete failed".into(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn set_timezone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(query): Form<SetQuery>,
) -> impl IntoResponse {
    let user = match validate_session(&headers, &state).await {
        Ok(user) => user,
        Err(err) => return err.into_response(),
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
        SET username = EXCLUDED.username, timezone = EXCLUDED.timezone, updated_at = NOW()
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
        Err(e) => {
            error!("Failed to save timezone: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Database error".into(),
                }),
            )
                .into_response()
        }
    }
}
