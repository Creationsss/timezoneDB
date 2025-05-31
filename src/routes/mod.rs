use crate::db::AppState;
use axum::{
    http::{HeaderValue, StatusCode},
    response::Response,
    routing::{get, options},
    Router,
};

pub mod auth;
mod timezone;

async fn preflight_handler() -> Response {
    let mut res = Response::new("".into());

    let headers = res.headers_mut();
    headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
    headers.insert(
        "access-control-allow-methods",
        HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    headers.insert(
        "access-control-allow-headers",
        HeaderValue::from_static("Content-Type, Authorization"),
    );
    headers.insert(
        "access-control-allow-credentials",
        HeaderValue::from_static("true"),
    );
    headers.insert("vary", HeaderValue::from_static("Origin"));

    *res.status_mut() = StatusCode::OK;

    res
}

pub fn all() -> Router<AppState> {
    Router::new()
        .route("/get", get(timezone::get_timezone))
        .route("/set", get(timezone::set_timezone))
        .route("/set", options(preflight_handler))
        .route("/delete", get(timezone::delete_timezone))
        .route("/list", get(timezone::list_timezones))
        .route("/auth/discord", get(auth::start_oauth))
        .route("/auth/discord/callback", get(auth::handle_callback))
        .route("/me", get(auth::me))
}
