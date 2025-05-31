use crate::db::AppState;
use axum::{Router, routing::get};

pub mod auth;
mod timezone;

pub fn all() -> Router<AppState> {
    Router::new()
        .route("/get", get(timezone::get_timezone))
        .route("/set", get(timezone::set_timezone))
        .route("/delete", get(timezone::delete_timezone))
        .route("/list", get(timezone::list_timezones))
        .route("/auth/discord", get(auth::start_oauth))
        .route("/auth/discord/callback", get(auth::handle_callback))
        .route("/me", get(auth::me))
}
