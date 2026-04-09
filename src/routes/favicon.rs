use crate::constants;
use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::Response;
use std::fs;

pub async fn favicon() -> Response {
    match fs::read(constants::FAVICON_PATH) {
        Ok(content) => match Response::builder()
            .header(header::CONTENT_TYPE, "image/x-icon")
            .body(Body::from(content))
        {
            Ok(response) => response,
            Err(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal Server Error"))
                .unwrap_or_else(|_| Response::new(Body::from("Error"))),
        },
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("404 Not Found"))
            .unwrap_or_else(|_| Response::new(Body::from("Not Found"))),
    }
}
