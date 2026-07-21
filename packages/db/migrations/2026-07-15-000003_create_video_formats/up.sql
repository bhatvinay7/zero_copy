CREATE TABLE IF NOT EXISTS video_formats (
    id         SERIAL PRIMARY KEY,
    video_id   INT         NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
    resolution TEXT        NOT NULL,
    url        TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
