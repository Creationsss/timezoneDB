use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;

pub async fn connect() -> PgPool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is required");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS timezones (
            user_id TEXT PRIMARY KEY,
            username TEXT NOT NULL,
            timezone TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create timezones table");

    pool
}
