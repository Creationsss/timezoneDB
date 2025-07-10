use crate::db::AppState;
use crate::types::JsonMessage;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use sqlx::Row;
use std::collections::HashMap;
use tracing::error;

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_users: i64,
    pub total_timezones: i64,
    pub timezone_distribution: HashMap<String, i64>,
    pub top_timezones: Vec<TimezoneCount>,
    pub unique_timezones: i64,
    pub top_timezone: String,
    pub recent_registrations: i64,
}

#[derive(Serialize)]
pub struct TimezoneCount {
    pub timezone: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct BasicStats {
    pub total_users: i64,
    pub total_timezones: i64,
    pub top_timezone: Option<String>,
}

pub async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let total_users_result = sqlx::query("SELECT COUNT(*) as count FROM timezones")
        .fetch_one(&state.db)
        .await;

    let total_users = match total_users_result {
        Ok(row) => row.get::<i64, _>("count"),
        Err(e) => {
            error!("Failed to get total users: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Failed to fetch stats".into(),
                }),
            )
                .into_response();
        }
    };

    let timezone_dist_result = sqlx::query(
        "SELECT timezone, COUNT(*) as count FROM timezones GROUP BY timezone ORDER BY count DESC",
    )
    .fetch_all(&state.db)
    .await;

    let (timezone_distribution, top_timezones) = match timezone_dist_result {
        Ok(rows) => {
            let mut distribution = HashMap::new();
            let mut top_zones = Vec::new();

            for row in rows {
                let timezone: String = row.get("timezone");
                let count: i64 = row.get("count");

                distribution.insert(timezone.clone(), count);

                if top_zones.len() < 10 {
                    top_zones.push(TimezoneCount { timezone, count });
                }
            }

            (distribution, top_zones)
        }
        Err(e) => {
            error!("Failed to get timezone distribution: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonMessage {
                    message: "Failed to fetch timezone distribution".into(),
                }),
            )
                .into_response();
        }
    };

    let recent_registrations_result = sqlx::query(
        "SELECT COUNT(*) as count FROM timezones WHERE created_at > NOW() - INTERVAL '7 days'",
    )
    .fetch_one(&state.db)
    .await;

    let recent_registrations = match recent_registrations_result {
        Ok(row) => row.get::<i64, _>("count"),
        Err(e) => {
            error!("Failed to get recent registrations: {}", e);
            0
        }
    };

    let total_timezones = timezone_distribution.len() as i64;
    let unique_timezones = timezone_distribution.len() as i64;
    let top_timezone = top_timezones
        .first()
        .map(|tz| tz.timezone.clone())
        .unwrap_or_default();

    let stats = StatsResponse {
        total_users,
        total_timezones,
        timezone_distribution,
        top_timezones,
        unique_timezones,
        top_timezone,
        recent_registrations,
    };

    (StatusCode::OK, Json(stats)).into_response()
}
