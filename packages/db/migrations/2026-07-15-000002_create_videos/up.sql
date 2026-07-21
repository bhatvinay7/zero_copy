CREATE TABLE IF NOT EXISTS videos (
    id           SERIAL PRIMARY KEY,
    user_id      INT         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename     TEXT        NOT NULL,
    original_url TEXT        NOT NULL,
    status       TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
