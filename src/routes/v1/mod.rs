use crate::constants;
use crate::db::AppState;
use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    response::Response,
    routing::{delete, get, options, post},
    Router,
};

pub mod auth;
mod health;
mod stats;
mod timezone;

async fn preflight_handler(req: Request) -> Response {
    let mut res = Response::new("".into());

    let headers = res.headers_mut();
    if let Some(origin) = req.headers().get("origin").cloned() {
        headers.insert("access-control-allow-origin", origin);
    }
    headers.insert(
        "access-control-allow-methods",
        HeaderValue::from_static(constants::CORS_ALLOWED_METHODS),
    );
    headers.insert(
        "access-control-allow-headers",
        HeaderValue::from_static(constants::CORS_ALLOWED_HEADERS),
    );
    headers.insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );
    headers.insert("vary", HeaderValue::from_static("Origin"));

    *res.status_mut() = StatusCode::OK;

    res
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/get", get(timezone::get_timezone))
        .route("/set", post(timezone::set_timezone))
        .route("/set", options(preflight_handler))
        .route("/stats", get(stats::get_stats))
        .route("/delete", delete(timezone::delete_timezone))
        .route("/delete", options(preflight_handler))
        .route("/list", get(timezone::list_timezones))
        .route("/auth/discord", get(auth::start_oauth))
        .route("/auth/discord/callback", get(auth::handle_callback))
        .route("/me", get(auth::me))
        .route("/logout", get(auth::logout))
        .route("/health", get(health::health_check))
}
