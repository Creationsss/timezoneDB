# timezone-db

A simple Rust-powered API service for managing and retrieving user timezones.

## Features

- Store user timezones via `/set` endpoint (requires Discord OAuth)
- Retrieve timezones by user ID via `/get`
- List all saved timezones
- List every valid IANA timezone via `/timezones`
- Cookie-based session handling using Redis connection pooling
- Built-in CORS support
- Structured configuration with validation
- Graceful shutdown support
- Fully containerized with PostgreSQL and DragonflyDB

## Requirements

- Docker & Docker Compose
- `.env` file with required environment variables

## Environment Variables

Create a `.env` file with the following:

```env
# Server Configuration
HOST=0.0.0.0
PORT=3000

# Database Configuration
DATABASE_URL=postgres://postgres:postgres@postgres:5432/postgres
DB_MAX_CONNECTIONS=10
DB_CONNECT_TIMEOUT=30

# Redis Configuration
REDIS_URL=redis://dragonfly:6379
REDIS_POOL_SIZE=5
REDIS_CONNECT_TIMEOUT=10

# Discord OAuth Configuration
CLIENT_ID=your_discord_client_id
CLIENT_SECRET=your_discord_client_secret
REDIRECT_URI=https://your.domain/auth/discord/callback

# Logging (optional)
RUST_LOG=info,timezone_db=debug
```

## Setup

### Build and Run with Docker

```bash
docker compose up --build
```

### Run Manually

```bash
# Make sure PostgreSQL and Redis are running
cargo run
```

## API Endpoints

Every endpoint is documented at `/docs` on the running service.

## Configuration

The application uses structured configuration with validation. All required environment variables must be provided, and the app will exit with helpful error messages if configuration is invalid.

### Optional Configuration Variables

- `DB_MAX_CONNECTIONS`: Maximum database connections (default: 10)
- `DB_CONNECT_TIMEOUT`: Database connection timeout in seconds (default: 30)
- `REDIS_POOL_SIZE`: Redis connection pool size (default: 5)
- `REDIS_CONNECT_TIMEOUT`: Redis connection timeout in seconds (default: 10)
- `RUST_LOG`: Logging level configuration