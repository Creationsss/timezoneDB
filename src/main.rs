use axum::{serve, Router};
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod middleware;
mod routes;
mod types;

use config::Config;
use db::{postgres, redis_helper, AppState};
use middleware::cors::DynamicCors;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = match Config::from_env() {
        Ok(config) => {
            if let Err(e) = config.validate() {
                error!("Configuration validation failed: {}", e);
                std::process::exit(1);
            }
            config
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    info!("Starting timezone-db server");
    info!("Server will bind to: {}", config.server.bind_address);

    let db = match postgres::connect(&config.database).await {
        Ok(pool) => {
            info!("Successfully connected to PostgreSQL");
            pool
        }
        Err(e) => {
            error!("Failed to connect to PostgreSQL: {}", e);
            std::process::exit(1);
        }
    };

    let redis = match redis_helper::connect(&config.redis).await {
        Ok(pool) => {
            info!("Successfully connected to Redis");
            pool
        }
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            std::process::exit(1);
        }
    };

    let state = AppState {
        db,
        redis,
        config: config.clone(),
    };

    let app = Router::new()
        .merge(routes::all())
        .with_state(state)
        .layer(DynamicCors);

    let listener = match TcpListener::bind(config.server.bind_address).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to {}: {}", config.server.bind_address, e);
            std::process::exit(1);
        }
    };

    info!("Server listening on http://{}", config.server.bind_address);

    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        warn!("Shutdown signal received");
    };

    if let Err(err) = serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
    {
        error!("Server error: {}", err);
        std::process::exit(1);
    }

    info!("Server has shut down gracefully");
}
