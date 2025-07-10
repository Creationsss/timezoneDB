use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::Response;
use std::fs;

pub async fn favicon() -> Response {
    match fs::read("public/favicon.ico") {
        Ok(content) => Response::builder()
            .header(header::CONTENT_TYPE, "image/x-icon")
            .body(Body::from(content))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap(),
    }
}
