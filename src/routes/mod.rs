use crate::constants;
use crate::db::AppState;
use axum::{http::StatusCode, response::Html, response::IntoResponse, routing::get, Router};
use std::fs;
use tower_http::services::ServeDir;

mod favicon;
pub mod v1;

async fn index_page() -> Html<String> {
    Html(
        fs::read_to_string(constants::INDEX_PAGE)
            .unwrap_or_else(|_| "<h1>404 Not Found</h1>".to_string()),
    )
}

async fn privacy_page() -> Html<String> {
    Html(
        fs::read_to_string(constants::PRIVACY_PAGE)
            .unwrap_or_else(|_| "<h1>404 Not Found</h1>".to_string()),
    )
}

async fn not_found_page() -> impl IntoResponse {
    let body = fs::read_to_string(constants::NOT_FOUND_PAGE)
        .unwrap_or_else(|_| "<h1>404 Not Found</h1>".to_string());
    (StatusCode::NOT_FOUND, Html(body))
}

pub fn all() -> Router<AppState> {
    Router::new()
        .route("/favicon.ico", get(favicon::favicon))
        .route("/", get(index_page))
        .route("/privacy", get(privacy_page))
        .nest("/v1", v1::routes())
        .merge(v1::routes())
        .nest_service("/public", ServeDir::new(constants::PUBLIC_DIR))
        .fallback(get(not_found_page))
}
