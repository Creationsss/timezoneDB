pub const SESSION_COOKIE_NAME: &str = "session";
pub const SESSION_TTL_SECONDS: u64 = 3600;
pub const SESSION_KEY_PREFIX: &str = "session:";

pub const DISCORD_OAUTH_AUTHORIZE_URL: &str = "https://discord.com/oauth2/authorize";
pub const DISCORD_OAUTH_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
pub const DISCORD_USER_API_URL: &str = "https://discord.com/api/users/@me";
pub const DISCORD_DOMAINS: &[&str] = &[
    "https://discord.com",
    "https://discordapp.com",
    "https://ptb.discord.com",
    "https://canary.discord.com",
];

pub const HTTP_CLIENT_TIMEOUT_SECONDS: u64 = 30;
pub const CORS_ALLOWED_METHODS: &str = "GET, POST, DELETE, OPTIONS";
pub const CORS_ALLOWED_HEADERS: &str = "Content-Type, Authorization";
pub const TIMEZONE_LIST_CACHE_CONTROL: &str = "public, max-age=86400";

pub const DB_IDLE_TIMEOUT_SECONDS: u64 = 600;
pub const DB_MAX_LIFETIME_SECONDS: u64 = 1800;
pub const MIGRATIONS_DIR: &str = "migrations";

pub const DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: &str = "3000";
pub const DEFAULT_DB_MAX_CONNECTIONS: &str = "10";
pub const DEFAULT_DB_CONNECT_TIMEOUT: &str = "30";
pub const DEFAULT_REDIS_POOL_SIZE: &str = "5";
pub const DEFAULT_REDIS_CONNECT_TIMEOUT: &str = "10";

pub const PUBLIC_DIR: &str = "public";
pub const INDEX_PAGE: &str = "public/index.html";
pub const PRIVACY_PAGE: &str = "public/privacy.html";
pub const FAVICON_PATH: &str = "public/favicon.ico";
pub const NOT_FOUND_PAGE: &str = "public/404.html";

pub const TOP_TIMEZONES_LIMIT: usize = 10;
