# timezone-db

A simple Rust-powered API service for managing and retrieving user timezones.

## Features

- Store user timezones via `/set` endpoint (requires Discord OAuth)
- Retrieve timezones by user ID via `/get`
- List all saved timezones
- Cookie-based session handling using Redis
- Built-in CORS support
- Fully containerized with PostgreSQL and DragonflyDB

## Requirements

- Docker & Docker Compose
- `.env` file with required environment variables

## Environment Variables

Create a `.env` file with the following:

```env
HOST=0.0.0.0
PORT=3000

DATABASE_URL=postgres://postgres:postgres@postgres:5432/postgres
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

## API Endpoints

### `GET /get?id=<discord_user_id>`

Returns stored timezone and username for the given user ID.

### `GET /set?timezone=<iana_timezone>`

Stores timezone for the authenticated user. Requires Discord OAuth session.

### `GET /delete`

Deletes the authenticated user's timezone entry. Requires Discord OAuth session.

### `GET /list`

Returns a JSON object of all stored timezones by user ID.

### `GET /me`

Returns Discord profile info for the current session.

### `GET /auth/discord`

Starts OAuth2 authentication flow.

### `GET /auth/discord/callback`

Handles OAuth2 redirect and sets a session cookie.

## License

[BSD-3-Clause](LICENSE)
