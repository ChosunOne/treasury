CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE TABLE cursor_key (
        id SERIAL PRIMARY KEY,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        expires_at TIMESTAMPTZ,
        key_data BYTEA NOT NULL DEFAULT gen_random_bytes(32)
);
CREATE INDEX ix_cursor_key_expires_at ON cursor_key (expires_at);
