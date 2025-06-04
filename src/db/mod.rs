pub mod postgres;
pub mod redis_helper;

use crate::config::Config;
pub use redis_helper::RedisPool;

pub type Db = sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub redis: RedisPool,
    pub config: Config,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("redis", &self.redis)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
