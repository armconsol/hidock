use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::types::{CachedTranslation, TranslationResponse};

/// Cache for storing translations in SQLite database
pub struct TranslationCache {
    conn: Arc<Mutex<Connection>>,
}

impl TranslationCache {
    /// Create a new translation cache with database connection
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create translations table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS translations (
                id TEXT PRIMARY KEY,
                source_text TEXT NOT NULL,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                last_accessed DATETIME NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 0,
                UNIQUE(source_text, source_lang, target_lang)
            )",
            [],
        )?;

        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_translations_lookup
             ON translations(source_text, source_lang, target_lang)",
            [],
        )?;

        // Create index for cleanup by date
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_translations_last_accessed
             ON translations(last_accessed)",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Get cached translation if it exists
    pub async fn get_translation(
        &self,
        source_text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Option<TranslationResponse>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, source_text, source_lang, target_lang, translated_text,
                    created_at, last_accessed, access_count
             FROM translations
             WHERE source_text = ?1 AND source_lang = ?2 AND target_lang = ?3",
        )?;

        let result = stmt.query_row(
            params![source_text, source_lang, target_lang],
            |row| {
                Ok(CachedTranslation {
                    id: row.get(0)?,
                    source_text: row.get(1)?,
                    source_lang: row.get(2)?,
                    target_lang: row.get(3)?,
                    translated_text: row.get(4)?,
                    created_at: row.get(5)?,
                    last_accessed: row.get(6)?,
                    access_count: row.get(7)?,
                })
            },
        );

        match result {
            Ok(cached) => {
                // Update access count and last accessed time
                drop(stmt);
                self.update_access(&cached.id).await?;
                Ok(Some(TranslationResponse::from_cache(&cached)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Save a new translation to cache
    pub async fn save_translation(
        &self,
        source_text: &str,
        source_lang: &str,
        target_lang: &str,
        translated_text: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT OR REPLACE INTO translations
             (id, source_text, source_lang, target_lang, translated_text, created_at, last_accessed, access_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
            params![
                id,
                source_text,
                source_lang,
                target_lang,
                translated_text,
                now,
                now
            ],
        )?;

        Ok(())
    }

    /// Update access count and last accessed time
    async fn update_access(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE translations
             SET access_count = access_count + 1, last_accessed = ?1
             WHERE id = ?2",
            params![Utc::now(), id],
        )?;

        Ok(())
    }

    /// Clear translations older than specified days
    pub async fn clear_old_translations(&self, days: i64) -> Result<u64> {
        let conn = self.conn.lock().unwrap();
        let cutoff = Utc::now() - Duration::days(days);

        let deleted = conn.execute(
            "DELETE FROM translations WHERE last_accessed < ?1",
            params![cutoff],
        )?;

        Ok(deleted as u64)
    }

    /// Get cache statistics (total count and size)
    pub async fn get_cache_stats(&self) -> Result<(u64, u64)> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM translations", [], |row| {
            row.get(0)
        })?;

        let size: i64 = conn.query_row(
            "SELECT SUM(LENGTH(source_text) + LENGTH(translated_text)) FROM translations",
            [],
            |row| row.get(0),
        )?;

        Ok((count as u64, size as u64))
    }

    /// Clear all translations from cache
    pub async fn clear_all(&self) -> Result<u64> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM translations", [])?;
        Ok(deleted as u64)
    }

    /// Get most frequently accessed translations
    pub async fn get_popular_translations(&self, limit: i64) -> Result<Vec<CachedTranslation>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, source_text, source_lang, target_lang, translated_text,
                    created_at, last_accessed, access_count
             FROM translations
             ORDER BY access_count DESC
             LIMIT ?1",
        )?;

        let translations = stmt
            .query_map(params![limit], |row| {
                Ok(CachedTranslation {
                    id: row.get(0)?,
                    source_text: row.get(1)?,
                    source_lang: row.get(2)?,
                    target_lang: row.get(3)?,
                    translated_text: row.get(4)?,
                    created_at: row.get(5)?,
                    last_accessed: row.get(6)?,
                    access_count: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(translations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_cache() -> (TranslationCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_translations.db");
        let cache = TranslationCache::new(db_path.to_str().unwrap()).unwrap();
        (cache, temp_dir)
    }

    #[tokio::test]
    async fn test_save_and_get_translation() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("Hello", "en", "es", "Hola")
            .await
            .unwrap();

        let result = cache.get_translation("Hello", "en", "es").await.unwrap();

        assert!(result.is_some());
        let translation = result.unwrap();
        assert_eq!(translation.translated_text, "Hola");
        assert_eq!(translation.source_lang, "en");
        assert_eq!(translation.target_lang, "es");
    }

    #[tokio::test]
    async fn test_get_nonexistent_translation() {
        let (cache, _temp) = setup_test_cache();

        let result = cache
            .get_translation("Nonexistent", "en", "es")
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_access_count_increments() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("Test", "en", "es", "Prueba")
            .await
            .unwrap();

        // Access multiple times
        cache.get_translation("Test", "en", "es").await.unwrap();
        cache.get_translation("Test", "en", "es").await.unwrap();
        cache.get_translation("Test", "en", "es").await.unwrap();

        // Check access count
        let popular = cache.get_popular_translations(1).await.unwrap();
        assert_eq!(popular.len(), 1);
        assert_eq!(popular[0].access_count, 4); // 1 initial + 3 accesses
    }

    #[tokio::test]
    async fn test_clear_old_translations() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("Old", "en", "es", "Viejo")
            .await
            .unwrap();

        // Clear translations accessed in last 0 days (all)
        let deleted = cache.clear_old_translations(0).await.unwrap();

        assert_eq!(deleted, 1);

        let result = cache.get_translation("Old", "en", "es").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("Test1", "en", "es", "Prueba1")
            .await
            .unwrap();
        cache
            .save_translation("Test2", "en", "fr", "Test2")
            .await
            .unwrap();

        let (count, size) = cache.get_cache_stats().await.unwrap();

        assert_eq!(count, 2);
        assert!(size > 0);
    }

    #[tokio::test]
    async fn test_unique_constraint() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("Duplicate", "en", "es", "First")
            .await
            .unwrap();

        // This should replace the first entry
        cache
            .save_translation("Duplicate", "en", "es", "Second")
            .await
            .unwrap();

        let result = cache
            .get_translation("Duplicate", "en", "es")
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().translated_text, "Second");

        let (count, _) = cache.get_cache_stats().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_different_language_pairs() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("Hello", "en", "es", "Hola")
            .await
            .unwrap();
        cache
            .save_translation("Hello", "en", "fr", "Bonjour")
            .await
            .unwrap();
        cache
            .save_translation("Hello", "en", "de", "Hallo")
            .await
            .unwrap();

        let es = cache.get_translation("Hello", "en", "es").await.unwrap();
        let fr = cache.get_translation("Hello", "en", "fr").await.unwrap();
        let de = cache.get_translation("Hello", "en", "de").await.unwrap();

        assert_eq!(es.unwrap().translated_text, "Hola");
        assert_eq!(fr.unwrap().translated_text, "Bonjour");
        assert_eq!(de.unwrap().translated_text, "Hallo");
    }

    #[tokio::test]
    async fn test_clear_all() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("Test1", "en", "es", "Prueba1")
            .await
            .unwrap();
        cache
            .save_translation("Test2", "en", "fr", "Test2")
            .await
            .unwrap();

        let deleted = cache.clear_all().await.unwrap();
        assert_eq!(deleted, 2);

        let (count, _) = cache.get_cache_stats().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_popular_translations() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("Popular", "en", "es", "Popular")
            .await
            .unwrap();
        cache
            .save_translation("Less", "en", "es", "Menos")
            .await
            .unwrap();

        // Access first one multiple times
        for _ in 0..5 {
            cache.get_translation("Popular", "en", "es").await.unwrap();
        }

        let popular = cache.get_popular_translations(1).await.unwrap();
        assert_eq!(popular.len(), 1);
        assert_eq!(popular[0].source_text, "Popular");
        assert!(popular[0].access_count >= 5);
    }

    #[tokio::test]
    async fn test_empty_text() {
        let (cache, _temp) = setup_test_cache();

        cache
            .save_translation("", "en", "es", "")
            .await
            .unwrap();

        let result = cache.get_translation("", "en", "es").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().translated_text, "");
    }
}
