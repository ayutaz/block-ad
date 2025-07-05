//! AdBlock Core - High-performance ad blocking engine
//!
//! This crate provides the core filtering engine for the AdBlock application,
//! supporting both Android and iOS platforms through FFI bindings.

#![allow(non_snake_case)]

pub mod filter_engine;
pub mod ffi;
pub mod network;
pub mod rules;

pub use filter_engine::FilterEngine;

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
    #[allow(dead_code)]
    config: Config,
}

impl AdBlockCore {
    /// Create a new instance with the given configuration
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = FilterEngine::new(&config)?;
        
        Ok(Self {
            engine: std::sync::Arc::new(engine),
            config,
        })
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