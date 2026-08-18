-- HiNotes Desktop SQLite Schema

-- User settings and configuration
CREATE TABLE IF NOT EXISTS user_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Notes table
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT,
    folder_id TEXT,
    audio_url TEXT,
    duration TEXT,
    rating INTEGER CHECK(rating >= 1 AND rating <= 5),
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    synced_at DATETIME,
    FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
);

-- Whisper notes (quick voice notes)
CREATE TABLE IF NOT EXISTS whisper_notes (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    audio_url TEXT,
    created_at DATETIME NOT NULL,
    synced_at DATETIME
);

-- Folders
CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    synced_at DATETIME
);

-- To-do items
CREATE TABLE IF NOT EXISTS todos (
    id TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    due_date DATETIME,
    state TEXT CHECK(state IN ('open', 'closed')) NOT NULL DEFAULT 'open',
    smart_label TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    synced_at DATETIME
);

-- Calendar events
CREATE TABLE IF NOT EXISTS calendar_events (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    source TEXT CHECK(source IN ('google_calendar', 'hinotes')) NOT NULL,
    meeting_url TEXT,
    created_at DATETIME NOT NULL,
    synced_at DATETIME
);

-- Devices (HiDoc P1)
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT CHECK(status IN ('connected', 'disconnected')) NOT NULL,
    last_sync DATETIME,
    created_at DATETIME NOT NULL
);

-- Templates
CREATE TABLE IF NOT EXISTS templates (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    is_favorite BOOLEAN NOT NULL DEFAULT 0,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    synced_at DATETIME
);

-- Smart labels
CREATE TABLE IF NOT EXISTS smart_labels (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at DATETIME NOT NULL
);

-- Custom vocabulary
CREATE TABLE IF NOT EXISTS vocabulary (
    id TEXT PRIMARY KEY,
    word TEXT NOT NULL UNIQUE,
    pronunciation TEXT,
    created_at DATETIME NOT NULL
);

-- Sync metadata
CREATE TABLE IF NOT EXISTS sync_metadata (
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    last_synced DATETIME NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (entity_type, entity_id)
);

-- Pending operations queue (for offline mode)
CREATE TABLE IF NOT EXISTS pending_operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT CHECK(operation_type IN ('create', 'update', 'delete')) NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0
);

-- Audio cache metadata
CREATE TABLE IF NOT EXISTS audio_cache (
    note_id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    last_accessed DATETIME NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

-- Share links for public note sharing
CREATE TABLE IF NOT EXISTS share_links (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at DATETIME,
    created_at DATETIME NOT NULL,
    last_accessed_at DATETIME,
    access_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

-- Subscriptions (RevenueCat)
CREATE TABLE IF NOT EXISTS subscriptions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_id TEXT NOT NULL,
    status TEXT CHECK(status IN ('active', 'expired', 'canceled', 'trial')) NOT NULL,
    expires_at DATETIME,
    purchased_at DATETIME,
    canceled_at DATETIME,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Subscription events (audit trail)
CREATE TABLE IF NOT EXISTS subscription_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subscription_id INTEGER,
    event_type TEXT CHECK(event_type IN ('activated', 'expired', 'renewed', 'canceled')) NOT NULL,
    product_id TEXT NOT NULL,
    expires_at DATETIME,
    occurred_at DATETIME NOT NULL,
    FOREIGN KEY (subscription_id) REFERENCES subscriptions(id) ON DELETE SET NULL
);

-- Speaker profiles for diarization
CREATE TABLE IF NOT EXISTS speakers (
    id TEXT PRIMARY KEY,
    name TEXT,
    voice_signature TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Speaker segments in recordings
CREATE TABLE IF NOT EXISTS speaker_segments (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    speaker_id TEXT NOT NULL,
    start_time REAL NOT NULL,
    end_time REAL NOT NULL,
    confidence REAL NOT NULL CHECK(confidence >= 0.0 AND confidence <= 1.0),
    created_at DATETIME NOT NULL,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (speaker_id) REFERENCES speakers(id) ON DELETE CASCADE
);

-- Translations cache for storing translated text
CREATE TABLE IF NOT EXISTS translations (
    id TEXT PRIMARY KEY,
    source_text TEXT NOT NULL,
    source_lang TEXT NOT NULL,
    target_lang TEXT NOT NULL,
    translated_text TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    last_accessed DATETIME NOT NULL,
    access_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(source_text, source_lang, target_lang)
);

-- Referral codes table
CREATE TABLE IF NOT EXISTS referral_codes (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    code TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL,
    expires_at DATETIME,
    is_active BOOLEAN NOT NULL DEFAULT 1
);

-- Referral usage tracking
CREATE TABLE IF NOT EXISTS referral_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code_id TEXT NOT NULL,
    referred_user_id TEXT NOT NULL,
    referrer_user_id TEXT NOT NULL,
    applied_at DATETIME NOT NULL,
    reward_points INTEGER NOT NULL DEFAULT 0,
    reward_credits INTEGER,
    reward_subscription_days INTEGER,
    FOREIGN KEY (code_id) REFERENCES referral_codes(id) ON DELETE CASCADE,
    UNIQUE(referred_user_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder_id);
CREATE INDEX IF NOT EXISTS idx_notes_created ON notes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_todos_state ON todos(state);
CREATE INDEX IF NOT EXISTS idx_todos_due_date ON todos(due_date);
CREATE INDEX IF NOT EXISTS idx_calendar_start ON calendar_events(start_time);
CREATE INDEX IF NOT EXISTS idx_pending_ops_created ON pending_operations(created_at);
CREATE INDEX IF NOT EXISTS idx_audio_cache_accessed ON audio_cache(last_accessed);
CREATE INDEX IF NOT EXISTS idx_share_links_note ON share_links(note_id);
CREATE INDEX IF NOT EXISTS idx_share_links_token ON share_links(token);
CREATE INDEX IF NOT EXISTS idx_translations_lookup ON translations(source_text, source_lang, target_lang);
CREATE INDEX IF NOT EXISTS idx_translations_last_accessed ON translations(last_accessed);
CREATE INDEX IF NOT EXISTS idx_translations_access_count ON translations(access_count DESC);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_subscriptions_expires ON subscriptions(expires_at);
CREATE INDEX IF NOT EXISTS idx_subscription_events_type ON subscription_events(event_type);
CREATE INDEX IF NOT EXISTS idx_subscription_events_occurred ON subscription_events(occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_speaker_segments_note ON speaker_segments(note_id);
CREATE INDEX IF NOT EXISTS idx_speaker_segments_speaker ON speaker_segments(speaker_id);
CREATE INDEX IF NOT EXISTS idx_speaker_segments_time ON speaker_segments(start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_referral_codes_user ON referral_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_referral_codes_code ON referral_codes(code);
CREATE INDEX IF NOT EXISTS idx_referral_usage_referred ON referral_usage(referred_user_id);
CREATE INDEX IF NOT EXISTS idx_referral_usage_referrer ON referral_usage(referrer_user_id);
CREATE INDEX IF NOT EXISTS idx_referral_usage_code ON referral_usage(code_id);
