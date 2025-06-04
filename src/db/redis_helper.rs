use crate::config::RedisConfig;
use redis::{aio::MultiplexedConnection, Client, RedisError};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub type RedisConnection = MultiplexedConnection;

#[derive(Clone)]
pub struct RedisPool {
    connections: Arc<Mutex<VecDeque<RedisConnection>>>,
    client: Client,
    config: RedisConfig,
}

impl std::fmt::Debug for RedisPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisPool")
            .field("config", &self.config)
            .field("pool_size", &self.config.pool_size)
            .finish()
    }
}

impl RedisPool {
    pub async fn new(config: RedisConfig) -> Result<Self, RedisError> {
        let client = Client::open(config.url.clone())?;
        let connections = Arc::new(Mutex::new(VecDeque::new()));

        let pool = RedisPool {
            connections,
            client,
            config,
        };

        pool.initialize_pool().await?;
        Ok(pool)
    }

    async fn initialize_pool(&self) -> Result<(), RedisError> {
        let mut connections = self.connections.lock().await;

        for _ in 0..self.config.pool_size {
            let conn = self.create_connection().await?;
            connections.push_back(conn);
        }

        Ok(())
    }

    async fn create_connection(&self) -> Result<RedisConnection, RedisError> {
        tokio::time::timeout(
            Duration::from_secs(self.config.connect_timeout_seconds),
            self.client.get_multiplexed_tokio_connection(),
        )
        .await
        .map_err(|_| RedisError::from((redis::ErrorKind::IoError, "Connection timeout")))?
    }

    pub async fn get_connection(&self) -> Result<PooledConnection, RedisError> {
        let mut connections = self.connections.lock().await;

        let conn = if let Some(conn) = connections.pop_front() {
            conn
        } else {
            drop(connections);
            self.create_connection().await?
        };

        Ok(PooledConnection {
            connection: Some(conn),
            pool: self.clone(),
        })
    }

    async fn return_connection(&self, conn: RedisConnection) {
        let mut connections = self.connections.lock().await;

        if connections.len() < self.config.pool_size as usize {
            connections.push_back(conn);
        }
    }
}

pub struct PooledConnection {
    connection: Option<RedisConnection>,
    pool: RedisPool,
}

impl PooledConnection {
    pub fn as_mut(&mut self) -> &mut RedisConnection {
        self.connection
            .as_mut()
            .expect("Connection already returned to pool")
    }
}

impl redis::aio::ConnectionLike for PooledConnection {
    fn req_packed_command<'a>(
        &'a mut self,
        cmd: &'a redis::Cmd,
    ) -> redis::RedisFuture<'a, redis::Value> {
        self.as_mut().req_packed_command(cmd)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a redis::Pipeline,
        offset: usize,
        count: usize,
    ) -> redis::RedisFuture<'a, Vec<redis::Value>> {
        self.as_mut().req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.connection
            .as_ref()
            .expect("Connection already returned to pool")
            .get_db()
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.return_connection(conn).await;
            });
        }
    }
}

pub async fn connect(config: &RedisConfig) -> Result<RedisPool, RedisError> {
    RedisPool::new(config.clone()).await
}
