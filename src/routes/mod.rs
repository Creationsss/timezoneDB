use crate::constants;
use crate::db::AppState;
use crate::routes::v1::auth::session_id;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use tracing::error;

mod favicon;
pub mod v1;

async fn read_page(path: &'static str) -> Option<String> {
    tokio::fs::read_to_string(path)
        .await
        .inspect_err(|e| error!("Failed to read page {}: {}", path, e))
        .ok()
}

fn page_unavailable() -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(constants::PAGE_LOAD_ERROR_BODY),
    )
        .into_response()
}

async fn static_page(path: &'static str) -> Response {
    read_page(path)
        .await
        .map_or_else(page_unavailable, |body| Html(body).into_response())
}

async fn index_page(headers: HeaderMap) -> Response {
    let Some(mut body) = read_page(constants::INDEX_PAGE).await else {
        return page_unavailable();
    };

    if session_id(&headers).is_some() {
        let pending = format!("{} auth-pending", constants::INDEX_BODY_CLASS);
        body = body.replacen(constants::INDEX_BODY_CLASS, &pending, 1);
    }

    Html(body).into_response()
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
        .route("/", get(index_page))
        .route("/privacy", get(|| static_page(constants::PRIVACY_PAGE)))
        .route("/docs", get(|| static_page(constants::DOCS_PAGE)))
        .route(
            "/integrations",
            get(|| static_page(constants::INTEGRATIONS_PAGE)),
        )
        .nest("/v1", v1::routes())
        .merge(v1::routes())
        .nest_service("/public", ServeDir::new(constants::PUBLIC_DIR))
        .fallback(not_found_page)
}
