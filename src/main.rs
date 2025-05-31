use axum::{Router, serve};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber;

mod db;
mod routes;
mod types;

use db::{AppState, postgres, redis_helper};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = postgres::connect().await;
    let redis = redis_helper::connect().await;
    let state = AppState { db, redis };

    let app = Router::new()
        .merge(routes::all())
        .with_state(state.clone())
        .layer(CorsLayer::permissive());

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let addr = format!("{}:{}", host, port)
        .parse::<SocketAddr>()
        .expect("Invalid HOST or PORT");

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    info!("Listening on http://{}", addr);
    if let Err(err) = serve(listener, app).await {
        error!("Server error: {}", err);
    }
}
