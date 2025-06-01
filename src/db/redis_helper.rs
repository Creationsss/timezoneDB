use redis::aio::MultiplexedConnection;
use redis::Client;
use std::env;

pub async fn connect() -> MultiplexedConnection {
    let url = env::var("REDIS_URL").expect("REDIS_URL is required");
    let client = Client::open(url).expect("Failed to create Redis client");
    client
        .get_multiplexed_tokio_connection()
        .await
        .expect("Failed to connect to Redis")
}
