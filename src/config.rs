use std::env;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub discord: DiscordConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connect_timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connect_timeout_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct DiscordConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid value for {var}: {value} - {reason}")]
    InvalidValue {
        var: String,
        value: String,
        reason: String,
    },
    #[error("Parse error for {var}: {source}")]
    ParseError {
        var: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let server = ServerConfig::from_env()?;
        let database = DatabaseConfig::from_env()?;
        let redis = RedisConfig::from_env()?;
        let discord = DiscordConfig::from_env()?;

        Ok(Config {
            server,
            database,
            redis,
            discord,
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.discord.redirect_uri.starts_with("http") {
            return Err(ConfigError::InvalidValue {
                var: "REDIRECT_URI".to_string(),
                value: self.discord.redirect_uri.clone(),
                reason: "Must start with http:// or https://".to_string(),
            });
        }

        if !self.database.url.starts_with("postgres://")
            && !self.database.url.starts_with("postgresql://")
        {
            return Err(ConfigError::InvalidValue {
                var: "DATABASE_URL".to_string(),
                value: "***hidden***".to_string(),
                reason: "Must be a valid PostgreSQL connection string".to_string(),
            });
        }

        if !self.redis.url.starts_with("redis://") && !self.redis.url.starts_with("rediss://") {
            return Err(ConfigError::InvalidValue {
                var: "REDIS_URL".to_string(),
                value: "***hidden***".to_string(),
                reason: "Must be a valid Redis connection string".to_string(),
            });
        }

        Ok(())
    }
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = get_env_or("HOST", "0.0.0.0")?
            .parse::<IpAddr>()
            .map_err(|e| ConfigError::ParseError {
                var: "HOST".to_string(),
                source: Box::new(e),
            })?;

        let port =
            get_env_or("PORT", "3000")?
                .parse::<u16>()
                .map_err(|e| ConfigError::ParseError {
                    var: "PORT".to_string(),
                    source: Box::new(e),
                })?;

        let bind_address = SocketAddr::new(host, port);

        Ok(ServerConfig { bind_address })
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let url = get_env_required("DATABASE_URL")?;

        let max_connections = get_env_or("DB_MAX_CONNECTIONS", "10")?
            .parse::<u32>()
            .map_err(|e| ConfigError::ParseError {
                var: "DB_MAX_CONNECTIONS".to_string(),
                source: Box::new(e),
            })?;

        let connect_timeout_seconds = get_env_or("DB_CONNECT_TIMEOUT", "30")?
            .parse::<u64>()
            .map_err(|e| ConfigError::ParseError {
                var: "DB_CONNECT_TIMEOUT".to_string(),
                source: Box::new(e),
            })?;

        Ok(DatabaseConfig {
            url,
            max_connections,
            connect_timeout_seconds,
        })
    }
}

impl RedisConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let url = get_env_required("REDIS_URL")?;

        let pool_size = get_env_or("REDIS_POOL_SIZE", "5")?
            .parse::<u32>()
            .map_err(|e| ConfigError::ParseError {
                var: "REDIS_POOL_SIZE".to_string(),
                source: Box::new(e),
            })?;

        let connect_timeout_seconds = get_env_or("REDIS_CONNECT_TIMEOUT", "10")?
            .parse::<u64>()
            .map_err(|e| ConfigError::ParseError {
                var: "REDIS_CONNECT_TIMEOUT".to_string(),
                source: Box::new(e),
            })?;

        Ok(RedisConfig {
            url,
            pool_size,
            connect_timeout_seconds,
        })
    }
}

impl DiscordConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let client_id = get_env_required("CLIENT_ID")?;
        let client_secret = get_env_required("CLIENT_SECRET")?;
        let redirect_uri = get_env_required("REDIRECT_URI")?;

        Ok(DiscordConfig {
            client_id,
            client_secret,
            redirect_uri,
        })
    }
}

fn get_env_required(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::MissingEnvVar(key.to_string()))
}

fn get_env_or(key: &str, default: &str) -> Result<String, ConfigError> {
    Ok(env::var(key).unwrap_or_else(|_| default.to_string()))
}
