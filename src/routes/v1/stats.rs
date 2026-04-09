use crate::constants;
use crate::db::AppState;
use crate::types::{JsonMessage, StatsResponse, TimezoneCount};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::Row;
use std::collections::HashMap;
use tracing::error;

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

                if top_zones.len() < constants::TOP_TIMEZONES_LIMIT {
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

    let recent_registrations_result = sqlx::query(&format!(
        "SELECT COUNT(*) as count FROM timezones WHERE created_at > NOW() - INTERVAL '{} days'",
        constants::RECENT_REGISTRATIONS_DAYS
    ))
    .fetch_one(&state.db)
    .await;

    let recent_registrations = match recent_registrations_result {
        Ok(row) => row.get::<i64, _>("count"),
        Err(e) => {
            error!("Failed to get recent registrations: {}", e);
            0
        }
    };

    let unique_timezones_result =
        sqlx::query("SELECT COUNT(DISTINCT timezone) as count FROM timezones")
            .fetch_one(&state.db)
            .await;

    let unique_timezones = match unique_timezones_result {
        Ok(row) => row.get::<i64, _>("count"),
        Err(e) => {
            error!("Failed to get unique timezones count: {}", e);
            timezone_distribution.len() as i64
        }
    };
    let top_timezone = top_timezones
        .first()
        .map(|tz| tz.timezone.clone())
        .unwrap_or_default();

    let stats = StatsResponse {
        total_users,
        total_timezones: total_users,
        timezone_distribution,
        top_timezones,
        unique_timezones,
        top_timezone,
        recent_registrations,
    };

    (StatusCode::OK, Json(stats)).into_response()
}
