CREATE TABLE IF NOT EXISTS genres (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS stories (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    genre_id TEXT NOT NULL REFERENCES genres(id),
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'complete')),
    is_sample INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS story_parts (
    id TEXT PRIMARY KEY,
    story_id TEXT NOT NULL REFERENCES stories(id) ON DELETE CASCADE,
    part_number INTEGER NOT NULL,
    content TEXT NOT NULL,
    audio_path TEXT,
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK(status IN ('draft','text_ready','audio_processing','audio_ready','audio_failed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(story_id, part_number)
);

CREATE TABLE IF NOT EXISTS audio_jobs (
    id TEXT PRIMARY KEY,
    story_part_id TEXT NOT NULL REFERENCES story_parts(id) ON DELETE CASCADE,
    voice_path TEXT,
    music_path TEXT,
    final_path TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK(status IN ('pending','voice_generating','music_generating','mixing','complete','failed')),
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
