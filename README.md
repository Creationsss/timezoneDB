# timezone-db

A simple Bun-powered API service for managing and retrieving user timezones. It supports both local and Discord OAuth-authenticated timezone storage, backed by PostgreSQL and Redis.

## Features

- Store user timezones via `/set` endpoint (requires Discord OAuth)
- Retrieve timezones by user ID via `/get`
- Cookie-based session handling using Redis
- Built-in CORS support
- Dockerized with PostgreSQL and DragonflyDB

## Requirements

- [Bun](https://bun.sh/)
- Docker & Docker Compose
- `.env` file with required environment variables

## Environment Variables

Create a `.env` file with the following:

```
HOST=0.0.0.0
PORT=3000

PGHOST=postgres
PGPORT=5432
PGUSERNAME=postgres
PGPASSWORD=postgres
PGDATABASE=timezone

REDIS_URL=redis://dragonfly:6379

CLIENT_ID=your_discord_client_id
CLIENT_SECRET=your_discord_client_secret
REDIRECT_URI=https://your.domain/auth/discord/callback
```

## Setup

### Build and Run with Docker

```bash
docker compose up --build
```

### Development Mode

```bash
bun dev
```

## API Endpoints

### `GET /get?id=<discord_user_id>`

Returns stored timezone and username for the given user ID.

### `GET /set?timezone=<iana_timezone>`

Stores timezone for the authenticated user. Requires Discord OAuth session.

### `GET /me`

Returns Discord profile info for the current session.

### `GET /auth/discord`

Starts OAuth2 authentication flow.

### `GET /auth/discord/callback`

Handles OAuth2 redirect and sets a session cookie.

## License

[BSD-3-Clause](LICENSE)
