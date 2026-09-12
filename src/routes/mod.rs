use crate::constants;
use crate::db::AppState;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::fs;
use tower_http::services::ServeDir;
use tracing::error;

mod favicon;
pub mod v1;

async fn static_page(path: &'static str) -> Response {
    match fs::read_to_string(path) {
        Ok(body) => Html(body).into_response(),
        Err(e) => {
            error!("Failed to read page {}: {}", path, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(constants::PAGE_LOAD_ERROR_BODY),
            )
                .into_response()
        }
    }
}

async fn not_found_page() -> Response {
    let mut response = static_page(constants::NOT_FOUND_PAGE).await;

    if response.status() == StatusCode::OK {
        *response.status_mut() = StatusCode::NOT_FOUND;
    }

    response
}

pub fn all() -> Router<AppState> {
    Router::new()
        .route("/favicon.ico", get(favicon::favicon))
        .route("/", get(|| static_page(constants::INDEX_PAGE)))
        .route("/privacy", get(|| static_page(constants::PRIVACY_PAGE)))
        .route("/docs", get(|| static_page(constants::DOCS_PAGE)))
        .nest("/v1", v1::routes())
        .merge(v1::routes())
        .nest_service("/public", ServeDir::new(constants::PUBLIC_DIR))
        .fallback(not_found_page)
}
