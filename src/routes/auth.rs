use crate::db::AppState;

use crate::types::JsonMessage;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use headers::{Cookie, HeaderMapExt};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::{collections::HashMap, env};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    user: DiscordUser,
    session: String,
}

pub async fn get_user_from_session(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<DiscordUser, impl IntoResponse> {
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

    let mut redis = state.redis.clone();
    let key = format!("session:{}", session_id);
    let Ok(json) = redis.get::<_, String>(&key).await else {
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

pub async fn start_oauth(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let client_id = env::var("CLIENT_ID").unwrap_or_default();
    let redirect_uri = env::var("REDIRECT_URI").unwrap_or_default();

    let mut url = format!(
		"https://discord.com/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope=identify",
		client_id, redirect_uri
	);

    if let Some(redirect) = params.get("redirect") {
        url.push_str(&format!("&state={}", urlencoding::encode(redirect)));
    }

    (StatusCode::FOUND, [(axum::http::header::LOCATION, url)]).into_response()
}

pub async fn handle_callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> impl IntoResponse {
    let client_id = env::var("CLIENT_ID").unwrap();
    let client_secret = env::var("CLIENT_SECRET").unwrap();
    let redirect_uri = env::var("REDIRECT_URI").unwrap();

    let form = [
        ("client_id", client_id.as_str()),
        ("client_secret", client_secret.as_str()),
        ("grant_type", "authorization_code"),
        ("code", &query.code),
        ("redirect_uri", redirect_uri.as_str()),
    ];

    let token_res = reqwest::Client::new()
        .post("https://discord.com/api/oauth2/token")
        .form(&form)
        .send()
        .await;

    let Ok(res) = token_res else {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonMessage {
                message: "Failed to exchange token".into(),
            }),
        )
            .into_response();
    };

    let Ok(token_json) = res.json::<serde_json::Value>().await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Invalid token response".into(),
            }),
        )
            .into_response();
    };

    let Some(access_token) = token_json["access_token"].as_str() else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JsonMessage {
                message: "Access token not found".into(),
            }),
        )
            .into_response();
    };

    let user_res = reqwest::Client::new()
        .get("https://discord.com/api/users/@me")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;

    let Ok(user_res) = user_res else {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonMessage {
                message: "Failed to fetch user".into(),
            }),
        )
            .into_response();
    };

    let Ok(user) = user_res.json::<DiscordUser>().await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonMessage {
                message: "Failed to parse user".into(),
            }),
        )
            .into_response();
    };

    let session_id = Uuid::now_v7().to_string();
    let mut redis = state.redis.clone();
    let _ = redis
        .set_ex::<_, _, ()>(
            format!("session:{}", session_id),
            serde_json::to_string(&user).unwrap(),
            3600,
        )
        .await;

    if let Some(redirect_url) = &query.state {
        let redirect_target = urlencoding::decode(redirect_url)
            .map(|s| s.into_owned())
            .unwrap_or("/".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(
            "Set-Cookie",
            format!(
                "session={}; Max-Age=3600; Path=/; SameSite=None; Secure; HttpOnly",
                session_id
            )
            .parse()
            .unwrap(),
        );
        headers.insert(
            axum::http::header::LOCATION,
            redirect_target.parse().unwrap(),
        );

        (StatusCode::FOUND, headers).into_response()
    } else {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Set-Cookie",
            format!(
                "session={}; Max-Age=3600; Path=/; SameSite=None; Secure; HttpOnly",
                session_id
            )
            .parse()
            .unwrap(),
        );

        let response = AuthResponse {
            user,
            session: session_id,
        };
        (StatusCode::OK, headers, Json(response)).into_response()
    }
}

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
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(JsonMessage {
                        message: "Failed to fetch timezone".into(),
                    }),
                )
                    .into_response(),
            }
        }
        Err(err) => err.into_response(),
    }
}
