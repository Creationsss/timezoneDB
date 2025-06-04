use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;

use crate::db::AppState;

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_healthy = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();

    let redis_healthy = state.redis.get_connection().await.is_ok();

    let status = if db_healthy && redis_healthy {
        "healthy"
    } else {
        "unhealthy"
    };
    let status_code = if status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        Json(serde_json::json!({
            "status": status,
            "database": db_healthy,
            "redis": redis_healthy,
            "timestamp": chrono::Utc::now()
        })),
    )
}
