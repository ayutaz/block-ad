//! AdBlock Core - High-performance ad blocking engine
//!
//! This crate provides the core filtering engine for the AdBlock application,
//! supporting both Android and iOS platforms through FFI bindings.

#![allow(non_snake_case)]

pub mod ffi;
pub mod filter_engine;
pub mod filter_list;
pub mod filter_updater;
pub mod network;
pub mod rules;
pub mod statistics;
pub mod utils;

pub use filter_engine::{BlockDecision, FilterEngine};
pub use filter_list::FilterListLoader;
pub use filter_updater::{FilterUpdater, UpdateConfig};
pub use statistics::{BlockEvent, DomainStats, Statistics};

/// Core configuration for the ad blocking engine
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// Enable verbose logging
    pub debug: bool,
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Rule update interval in seconds
    pub update_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            debug: false,
            max_memory_mb: 30,
            update_interval: 86400, // 24 hours
        }
    }
}

/// Main entry point for the ad blocking engine
pub struct AdBlockCore {
    engine: std::sync::Arc<FilterEngine>,
    statistics: std::sync::Mutex<Statistics>,
    #[allow(dead_code)]
    config: Config,
}

impl AdBlockCore {
    /// Create a new instance with the given configuration
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = FilterEngine::new(&config)?;

        Ok(Self {
            engine: std::sync::Arc::new(engine),
            statistics: std::sync::Mutex::new(Statistics::new()),
            config,
        })
    }

    /// Create a new instance with custom patterns
    pub fn with_patterns(patterns: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = FilterEngine::new_with_patterns(patterns);

        Ok(Self {
            engine: std::sync::Arc::new(engine),
            statistics: std::sync::Mutex::new(Statistics::new()),
            config: Config::default(),
        })
    }

    /// Create a new instance from a filter list
    pub fn from_filter_list(filter_list: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = FilterEngine::from_filter_list(filter_list)?;

        Ok(Self {
            engine: std::sync::Arc::new(engine),
            statistics: std::sync::Mutex::new(Statistics::new()),
            config: Config::default(),
        })
    }

    /// Check if a URL should be blocked and track statistics
    pub fn check_url(&mut self, url: &str, size: u64) -> BlockDecision {
        let decision = self.engine.should_block(url);

        // Extract domain from URL for statistics
        let domain = utils::extract_domain(url);

        // Track statistics
        self.track_decision(&decision, &domain, size);

        decision
    }

    /// Track the blocking decision in statistics
    fn track_decision(&self, decision: &BlockDecision, domain: &str, size: u64) {
        if let Ok(mut stats) = self.statistics.lock() {
            if decision.should_block {
                stats.record_blocked(domain, size);
            } else {
                stats.record_allowed(domain, size);
            }
        }
    }

    /// Get a copy of current statistics
    pub fn get_statistics(&self) -> Statistics {
        self.statistics
            .lock()
            .map(|stats| stats.clone())
            .unwrap_or_else(|_| Statistics::new())
    }

    /// Get a reference to the filter engine
    pub fn engine(&self) -> &FilterEngine {
        &self.engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.max_memory_mb, 30);
        assert_eq!(config.update_interval, 86400);
        assert!(!config.debug);
    }
}
