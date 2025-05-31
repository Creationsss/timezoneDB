pub mod postgres;
pub mod redis_helper;

pub type Db = sqlx::PgPool;
pub type Redis = redis::aio::MultiplexedConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub redis: Redis,
}
