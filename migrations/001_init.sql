-- Videos table
CREATE TABLE IF NOT EXISTS videos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(32) NOT NULL,
    platform_id VARCHAR(255) NOT NULL,
    title TEXT NOT NULL,
    uploader VARCHAR(255) NOT NULL,
    uploader_id VARCHAR(255) NOT NULL,
    duration INT NOT NULL DEFAULT 0,
    upload_date TIMESTAMPTZ,
    subtitles_path TEXT,
    transcription_path TEXT,
    heatmap_path TEXT,
    thumbnail_path TEXT,
    thumbnail_url TEXT,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(platform, platform_id)
);

-- Lives table
CREATE TABLE IF NOT EXISTS lives (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(32) NOT NULL,
    platform_id VARCHAR(255) NOT NULL,
    title TEXT NOT NULL,
    uploader VARCHAR(255) NOT NULL,
    uploader_id VARCHAR(255) NOT NULL,
    duration INT,
    live_date TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    transcription_path TEXT,
    thumbnail_path TEXT,
    thumbnail_url TEXT,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(platform, platform_id)
);

-- Channels to watch (global registry, shared across clients)
CREATE TABLE IF NOT EXISTS channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(32) NOT NULL,
    platform_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    watcher_count INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(platform, platform_id)
);

-- Per-client channel subscriptions
CREATE TABLE IF NOT EXISTS watched_channels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES api_keys(id),
    platform VARCHAR(32) NOT NULL,
    platform_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(client_id, platform, platform_id)
);

-- Cookies stored per platform
CREATE TABLE IF NOT EXISTS cookies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(32) NOT NULL UNIQUE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- API keys for authentication
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_videos_platform ON videos(platform);
CREATE INDEX IF NOT EXISTS idx_videos_uploader ON videos(uploader);
CREATE INDEX IF NOT EXISTS idx_videos_upload_date ON videos(upload_date DESC);
CREATE INDEX IF NOT EXISTS idx_lives_platform ON lives(platform);
CREATE INDEX IF NOT EXISTS idx_lives_is_active ON lives(is_active);
CREATE INDEX IF NOT EXISTS idx_lives_live_date ON lives(live_date DESC);
CREATE INDEX IF NOT EXISTS idx_channels_platform ON channels(platform);