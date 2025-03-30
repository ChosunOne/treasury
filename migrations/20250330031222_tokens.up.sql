CREATE TABLE csrf_token (
        token TEXT NOT NULL PRIMARY KEY UNIQUE
);

CREATE TABLE auth_token (
        id SERIAL NOT NULL PRIMARY KEY,
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        access_secret TEXT NOT NULL,
        refresh_secret TEXT NOT NULL
);
