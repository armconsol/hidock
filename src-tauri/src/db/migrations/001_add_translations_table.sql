-- Add translations table for caching translated text
-- Migration: 001_add_translations_table
-- Created: 2026-08-18

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

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_translations_lookup
ON translations(source_text, source_lang, target_lang);

-- Create index for cleanup by date
CREATE INDEX IF NOT EXISTS idx_translations_last_accessed
ON translations(last_accessed);

-- Create index for popular translations
CREATE INDEX IF NOT EXISTS idx_translations_access_count
ON translations(access_count DESC);
