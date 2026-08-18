use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::PathBuf;

const MAX_CACHE_SIZE_BYTES: u64 = 500 * 1024 * 1024; // 500MB

/// Audio cache manager with LRU eviction
pub struct AudioCache {
    cache_dir: PathBuf,
    db_conn: Connection,
}

impl AudioCache {
    /// Create a new AudioCache instance
    pub fn new(cache_dir: PathBuf, db_conn: Connection) -> Result<Self> {
        // Ensure cache directory exists
        fs::create_dir_all(&cache_dir)
            .context(format!("Failed to create cache directory: {:?}", cache_dir))?;

        Ok(Self { cache_dir, db_conn })
    }

    /// Get platform-specific cache directory
    pub fn get_platform_cache_dir() -> Result<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").context("HOME environment variable not set")?;
            Ok(PathBuf::from(home)
                .join("Library")
                .join("Caches")
                .join("com.hidock.hinotes")
                .join("audio"))
        }

        #[cfg(target_os = "linux")]
        {
            let cache_home = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| {
                let home = std::env::var("HOME").expect("HOME not set");
                format!("{}/.cache", home)
            });
            Ok(PathBuf::from(cache_home).join("hinotes").join("audio"))
        }

        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA")
                .context("LOCALAPPDATA environment variable not set")?;
            Ok(PathBuf::from(local_app_data).join("HiNotes").join("audio"))
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            anyhow::bail!("Unsupported platform")
        }
    }

    /// Get audio file for a note, downloading if not cached
    pub async fn get_audio(&self, note_id: &str, audio_url: &str) -> Result<Vec<u8>> {
        // Check if already cached
        if let Some(cached_path) = self.get_cached_path(note_id)? {
            // Update last_accessed timestamp
            self.update_last_accessed(note_id)?;

            // Read and return cached file
            let data = fs::read(&cached_path)
                .context(format!("Failed to read cached audio: {:?}", cached_path))?;
            return Ok(data);
        }

        // Download audio file
        let data = self.download_audio(audio_url).await?;

        // Cache the downloaded audio
        self.cache_audio(note_id, &data)?;

        Ok(data)
    }

    /// Download audio from URL
    async fn download_audio(&self, audio_url: &str) -> Result<Vec<u8>> {
        let response = reqwest::get(audio_url)
            .await
            .context(format!("Failed to download audio from: {}", audio_url))?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error {}: {}", response.status(), audio_url);
        }

        let data = response
            .bytes()
            .await
            .context("Failed to read audio response body")?;

        Ok(data.to_vec())
    }

    /// Cache audio data for a note
    pub fn cache_audio(&self, note_id: &str, data: &[u8]) -> Result<()> {
        let size_bytes = data.len() as u64;

        // Ensure we have enough space
        self.ensure_space(size_bytes)?;

        // Write audio file to cache directory
        let file_path = self.cache_dir.join(format!("{}.audio", note_id));
        fs::write(&file_path, data)
            .context(format!("Failed to write audio cache: {:?}", file_path))?;

        // Store metadata in database
        let now = Utc::now().to_rfc3339();
        self.db_conn.execute(
            "INSERT OR REPLACE INTO audio_cache (note_id, file_path, size_bytes, last_accessed)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                note_id,
                file_path
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?,
                size_bytes as i64,
                now
            ],
        )?;

        Ok(())
    }

    /// Get cached file path if exists
    fn get_cached_path(&self, note_id: &str) -> Result<Option<PathBuf>> {
        let result: Option<String> = self
            .db_conn
            .query_row(
                "SELECT file_path FROM audio_cache WHERE note_id = ?1",
                params![note_id],
                |row| row.get(0),
            )
            .optional()?;

        match result {
            Some(path_str) => {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    Ok(Some(path))
                } else {
                    // File was deleted, clean up metadata
                    self.remove_cache_entry(note_id)?;
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Update last_accessed timestamp
    fn update_last_accessed(&self, note_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.db_conn.execute(
            "UPDATE audio_cache SET last_accessed = ?1 WHERE note_id = ?2",
            params![now, note_id],
        )?;
        Ok(())
    }

    /// Ensure enough space by evicting LRU entries if needed
    fn ensure_space(&self, required_bytes: u64) -> Result<()> {
        let mut current_size = self.get_cache_size()?;

        // If adding this file would exceed limit, evict oldest entries
        while current_size + required_bytes > MAX_CACHE_SIZE_BYTES {
            if !self.evict_lru()? {
                // No more entries to evict
                if current_size + required_bytes > MAX_CACHE_SIZE_BYTES {
                    anyhow::bail!(
                        "Cannot cache audio: file size {} exceeds available cache space",
                        required_bytes
                    );
                }
                break;
            }
            current_size = self.get_cache_size()?;
        }

        Ok(())
    }

    /// Evict least recently used cache entry
    pub fn evict_lru(&self) -> Result<bool> {
        // Find oldest accessed entry
        let result: Option<(String, String)> = self
            .db_conn
            .query_row(
                "SELECT note_id, file_path FROM audio_cache
                 ORDER BY last_accessed ASC
                 LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        match result {
            Some((note_id, file_path)) => {
                // Delete the file
                let path = PathBuf::from(file_path);
                if path.exists() {
                    fs::remove_file(&path)
                        .context(format!("Failed to remove cached file: {:?}", path))?;
                }

                // Remove from database
                self.remove_cache_entry(&note_id)?;

                Ok(true)
            }
            None => Ok(false), // No entries to evict
        }
    }

    /// Remove cache entry from database
    fn remove_cache_entry(&self, note_id: &str) -> Result<()> {
        self.db_conn.execute(
            "DELETE FROM audio_cache WHERE note_id = ?1",
            params![note_id],
        )?;
        Ok(())
    }

    /// Get total size of cached audio files in bytes
    pub fn get_cache_size(&self) -> Result<u64> {
        let size: i64 = self.db_conn.query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM audio_cache",
            [],
            |row| row.get(0),
        )?;

        Ok(size as u64)
    }

    /// Clear all cached audio files
    pub fn clear_cache(&self) -> Result<()> {
        // Get all cached file paths
        let mut stmt = self.db_conn.prepare("SELECT file_path FROM audio_cache")?;

        let paths: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        // Delete all cached files
        for path_str in paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                fs::remove_file(&path).ok(); // Ignore errors
            }
        }

        // Clear database entries
        self.db_conn.execute("DELETE FROM audio_cache", [])?;

        Ok(())
    }

    /// Get number of cached entries
    pub fn get_cache_count(&self) -> Result<usize> {
        let count: i64 = self
            .db_conn
            .query_row("SELECT COUNT(*) FROM audio_cache", [], |row| row.get(0))?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_cache() -> (AudioCache, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("audio");

        let db_conn = Connection::open_in_memory().unwrap();

        // Create audio_cache table
        db_conn
            .execute(
                "CREATE TABLE IF NOT EXISTS audio_cache (
                note_id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                last_accessed DATETIME NOT NULL
            )",
                [],
            )
            .unwrap();

        let cache = AudioCache::new(cache_dir, db_conn).unwrap();
        (cache, temp_dir)
    }

    #[test]
    fn test_cache_audio() {
        let (cache, _temp_dir) = setup_test_cache();

        let note_id = "test-note-1";
        let audio_data = b"fake audio data";

        cache.cache_audio(note_id, audio_data).unwrap();

        // Verify cache size
        let size = cache.get_cache_size().unwrap();
        assert_eq!(size, audio_data.len() as u64);

        // Verify cache count
        let count = cache.get_cache_count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_cached_path() {
        let (cache, _temp_dir) = setup_test_cache();

        let note_id = "test-note-1";
        let audio_data = b"fake audio data";

        cache.cache_audio(note_id, audio_data).unwrap();

        let cached_path = cache.get_cached_path(note_id).unwrap();
        assert!(cached_path.is_some());
        assert!(cached_path.unwrap().exists());
    }

    #[test]
    fn test_get_cached_path_nonexistent() {
        let (cache, _temp_dir) = setup_test_cache();

        let cached_path = cache.get_cached_path("nonexistent").unwrap();
        assert!(cached_path.is_none());
    }

    #[test]
    fn test_evict_lru() {
        let (cache, _temp_dir) = setup_test_cache();

        // Cache multiple files
        cache.cache_audio("note-1", b"data1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        cache.cache_audio("note-2", b"data2").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        cache.cache_audio("note-3", b"data3").unwrap();

        assert_eq!(cache.get_cache_count().unwrap(), 3);

        // Evict LRU (should be note-1)
        let evicted = cache.evict_lru().unwrap();
        assert!(evicted);
        assert_eq!(cache.get_cache_count().unwrap(), 2);

        // Verify note-1 is gone
        let path = cache.get_cached_path("note-1").unwrap();
        assert!(path.is_none());
    }

    #[test]
    fn test_update_last_accessed() {
        let (cache, _temp_dir) = setup_test_cache();

        // Cache files with delays
        cache.cache_audio("note-1", b"data1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        cache.cache_audio("note-2", b"data2").unwrap();

        // Update note-1's last_accessed
        std::thread::sleep(std::time::Duration::from_millis(10));
        cache.update_last_accessed("note-1").unwrap();

        // Evict LRU (should now be note-2, not note-1)
        cache.evict_lru().unwrap();
        assert!(cache.get_cached_path("note-1").unwrap().is_some());
        assert!(cache.get_cached_path("note-2").unwrap().is_none());
    }

    #[test]
    fn test_clear_cache() {
        let (cache, _temp_dir) = setup_test_cache();

        cache.cache_audio("note-1", b"data1").unwrap();
        cache.cache_audio("note-2", b"data2").unwrap();
        cache.cache_audio("note-3", b"data3").unwrap();

        assert_eq!(cache.get_cache_count().unwrap(), 3);

        cache.clear_cache().unwrap();

        assert_eq!(cache.get_cache_count().unwrap(), 0);
        assert_eq!(cache.get_cache_size().unwrap(), 0);
    }

    #[test]
    fn test_cache_size_calculation() {
        let (cache, _temp_dir) = setup_test_cache();

        let data1 = vec![0u8; 1000];
        let data2 = vec![0u8; 2000];
        let data3 = vec![0u8; 3000];

        cache.cache_audio("note-1", &data1).unwrap();
        cache.cache_audio("note-2", &data2).unwrap();
        cache.cache_audio("note-3", &data3).unwrap();

        let total_size = cache.get_cache_size().unwrap();
        assert_eq!(total_size, 6000);
    }

    #[test]
    fn test_ensure_space_evicts_when_needed() {
        let (cache, _temp_dir) = setup_test_cache();

        // Fill cache with small files
        for i in 0..10 {
            let data = vec![0u8; 1000];
            cache.cache_audio(&format!("note-{}", i), &data).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert_eq!(cache.get_cache_count().unwrap(), 10);

        // Try to cache a file that requires eviction
        // This should trigger ensure_space to evict LRU entries
        let large_data = vec![0u8; MAX_CACHE_SIZE_BYTES as usize];
        cache.cache_audio("large-note", &large_data).unwrap();

        // All previous entries should be evicted
        assert_eq!(cache.get_cache_count().unwrap(), 1);
        assert!(cache.get_cached_path("large-note").unwrap().is_some());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_get_platform_cache_dir_macos() {
        let cache_dir = AudioCache::get_platform_cache_dir().unwrap();
        assert!(cache_dir
            .to_string_lossy()
            .contains("Library/Caches/com.hidock.hinotes/audio"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_get_platform_cache_dir_linux() {
        let cache_dir = AudioCache::get_platform_cache_dir().unwrap();
        assert!(
            cache_dir.to_string_lossy().contains(".cache/hinotes/audio")
                || cache_dir.to_string_lossy().contains("hinotes/audio")
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_platform_cache_dir_windows() {
        let cache_dir = AudioCache::get_platform_cache_dir().unwrap();
        assert!(cache_dir.to_string_lossy().contains("HiNotes\\audio"));
    }
}
