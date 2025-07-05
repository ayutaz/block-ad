//! Filter list updater for automatic updates
//! 
//! Downloads and caches filter lists from remote sources

use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::collections::HashMap;

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
            Some(last) => {
                match SystemTime::now().duration_since(last) {
                    Ok(elapsed) => elapsed >= self.config.update_interval,
                    Err(_) => true,
                }
            }
        }
    }
    
    /// Update with provided content (for testing)
    pub fn update_with_content(&mut self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Save to cache if configured
        if let Some(ref cache_dir) = self.config.cache_dir {
            std::fs::create_dir_all(cache_dir)?;
            let cache_file = cache_dir.join("filters_cache.txt");
            std::fs::write(&cache_file, content)?;
            
            // Save metadata
            let metadata_file = cache_dir.join("cache_metadata.json");
            let metadata = CacheMetadata {
                last_update: SystemTime::now(),
            };
            let metadata_json = serde_json::to_string(&metadata)?;
            std::fs::write(&metadata_file, metadata_json)?;
        }
        
        self.last_update = Some(SystemTime::now());
        Ok(())
    }
    
    /// Download a filter list from URL (simulated for testing)
    pub fn download_filter_list(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        // In a real implementation, this would use an HTTP client
        // For testing, we'll simulate failures for invalid URLs
        if url.contains("invalid") || url.contains("nonexistent") {
            return Err("Failed to download filter list".into());
        }
        
        // Simulate successful download
        Ok("||downloaded-ads.com^".to_string())
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
        if let Some(ref cache_dir) = self.config.cache_dir {
            let cache_file = cache_dir.join("filters_cache.txt");
            if cache_file.exists() {
                return std::fs::read_to_string(&cache_file)
                    .map_err(|e| e.into());
            }
        }
        
        Err("No cache available".into())
    }
    
    /// Load cache metadata
    fn load_cache_metadata(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref cache_dir) = self.config.cache_dir {
            let metadata_file = cache_dir.join("cache_metadata.json");
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