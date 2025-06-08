CREATE TABLE IF NOT EXISTS timezones (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    timezone TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);