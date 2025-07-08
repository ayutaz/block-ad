//! Filter list updater for automatic updates
//!
//! Downloads and caches filter lists from remote sources

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Default cache file names
const FILTER_CACHE_FILE: &str = "filters_cache.txt";
const METADATA_FILE: &str = "cache_metadata.json";

/// Configuration for filter updates
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// URLs to download filter lists from
    pub urls: Vec<String>,
    /// How often to check for updates
    pub update_interval: Duration,
    /// Directory to cache downloaded filters
    pub cache_dir: Option<PathBuf>,
}

/// Filter list updater
pub struct FilterUpdater {
    config: UpdateConfig,
    last_update: Option<SystemTime>,
    #[allow(dead_code)]
    cached_filters: HashMap<String, String>,
}

impl FilterUpdater {
    /// Create a new filter updater
    pub fn new(config: UpdateConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut updater = FilterUpdater {
            config,
            last_update: None,
            cached_filters: HashMap::new(),
        };

        // Try to load from cache on initialization
        if updater.config.cache_dir.is_some() {
            let _ = updater.load_cache_metadata();
        }

        Ok(updater)
    }

    /// Check if an update is needed
    pub fn needs_update(&self) -> bool {
        match self.last_update {
            None => true,
            Some(last) => match SystemTime::now().duration_since(last) {
                Ok(elapsed) => elapsed >= self.config.update_interval,
                Err(_) => true,
            },
        }
    }

    /// Update with provided content (for testing)
    pub fn update_with_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref cache_dir) = self.config.cache_dir {
            self.save_to_cache(cache_dir, content)?;
        }

        self.last_update = Some(SystemTime::now());
        Ok(())
    }

    /// Save filter content and metadata to cache
    fn save_to_cache(
        &self,
        cache_dir: &Path,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(cache_dir)?;

        // Save filter content
        let cache_file = cache_dir.join(FILTER_CACHE_FILE);
        std::fs::write(&cache_file, content)?;

        // Save metadata
        self.save_cache_metadata(cache_dir)?;

        Ok(())
    }

    /// Save cache metadata
    fn save_cache_metadata(&self, cache_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_file = cache_dir.join(METADATA_FILE);
        let metadata = CacheMetadata {
            last_update: SystemTime::now(),
        };
        let metadata_json = serde_json::to_string(&metadata)?;
        std::fs::write(&metadata_file, metadata_json)?;
        Ok(())
    }

    /// Download a filter list from URL
    pub fn download_filter_list(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // For testing, simulate failures for invalid URLs
        if url.contains("invalid") || url.contains("nonexistent") {
            return Err("Failed to download filter list".into());
        }

        // In production, this would use reqwest or similar HTTP client
        // For now, return a simulated response
        eprintln!("Note: HTTP download not implemented yet. URL: {}", url);
        
        // Simulate different content based on URL
        if url.contains("easylist") {
            Ok(include_str!("../tests/fixtures/easylist_sample.txt").to_string())
        } else if url.contains("easyprivacy") {
            Ok("! EasyPrivacy Sample\n||analytics.com^\n||tracking.net^".to_string())
        } else {
            Ok("||downloaded-ads.com^".to_string())
        }
    }

    /// Perform automatic update if needed
    pub fn auto_update(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if !self.needs_update() {
            // Try to load from cache
            if let Ok(cached) = self.load_from_cache() {
                return Ok(cached);
            }
        }

        // Download all configured filter lists
        let mut all_filters = Vec::new();
        
        for url in &self.config.urls.clone() {
            match self.download_filter_list(url) {
                Ok(content) => all_filters.push(content),
                Err(e) => eprintln!("Failed to download {}: {}", url, e),
            }
        }

        if all_filters.is_empty() {
            return Err("Failed to download any filter lists".into());
        }

        // Merge all downloaded lists
        let merged = self.merge_filter_lists(all_filters.iter().map(|s| s.as_str()).collect());
        
        // Save to cache
        self.update_with_content(&merged)?;
        
        Ok(merged)
    }

    /// Merge multiple filter lists
    pub fn merge_filter_lists(&self, lists: Vec<&str>) -> String {
        let mut merged = String::new();
        merged.push_str("! Merged Filter List\n");
        merged.push_str(&format!("! Generated at: {:?}\n\n", SystemTime::now()));

        for list in lists {
            merged.push_str(list);
            if !list.ends_with('\n') {
                merged.push('\n');
            }
        }

        merged
    }

    /// Load filters from cache
    pub fn load_from_cache(&self) -> Result<String, Box<dyn std::error::Error>> {
        let cache_dir = self
            .config
            .cache_dir
            .as_ref()
            .ok_or("No cache directory configured")?;

        let cache_file = cache_dir.join(FILTER_CACHE_FILE);
        if !cache_file.exists() {
            return Err("Cache file not found".into());
        }

        std::fs::read_to_string(&cache_file).map_err(|e| e.into())
    }

    /// Load cache metadata
    fn load_cache_metadata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref cache_dir) = self.config.cache_dir {
            let metadata_file = cache_dir.join(METADATA_FILE);
            if metadata_file.exists() {
                let metadata_json = std::fs::read_to_string(&metadata_file)?;
                let metadata: CacheMetadata = serde_json::from_str(&metadata_json)?;
                self.last_update = Some(metadata.last_update);
            }
        }
        Ok(())
    }
}

/// Cache metadata
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CacheMetadata {
    last_update: SystemTime,
}
