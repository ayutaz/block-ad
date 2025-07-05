//! Statistics tracking for ad blocking

use std::collections::HashMap;
use std::time::SystemTime;

/// A single block/allow event
#[derive(Debug, Clone)]
pub struct BlockEvent {
    pub timestamp: SystemTime,
    pub domain: String,
    pub blocked: bool,
    pub size: u64,
}

/// Domain-specific statistics
#[derive(Debug, Clone)]
pub struct DomainStats {
    pub domain: String,
    pub count: u64,
    pub data_saved: u64,
}

/// Configuration for statistics tracking
#[derive(Debug, Clone)]
pub struct StatisticsConfig {
    /// Maximum number of recent events to keep
    pub max_recent_events: usize,
}

impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            max_recent_events: 1000,
        }
    }
}

/// Statistics tracker for the ad blocker
#[derive(Debug, Clone)]
pub struct Statistics {
    blocked_count: u64,
    allowed_count: u64,
    data_saved: u64,
    domain_stats: HashMap<String, DomainStatsInternal>,
    recent_events: Vec<BlockEvent>,
    config: StatisticsConfig,
}

/// Internal domain statistics structure
#[derive(Debug, Default, Clone)]
struct DomainStatsInternal {
    count: u64,
    data_saved: u64,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            blocked_count: 0,
            allowed_count: 0,
            data_saved: 0,
            domain_stats: HashMap::new(),
            recent_events: Vec::new(),
            config: StatisticsConfig::default(),
        }
    }
}

impl Statistics {
    /// Create a new statistics instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a new statistics instance with custom configuration
    pub fn with_config(config: StatisticsConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }
    
    /// Get blocked count
    pub fn get_blocked_count(&self) -> u64 {
        self.blocked_count
    }
    
    /// Get allowed count
    pub fn get_allowed_count(&self) -> u64 {
        self.allowed_count
    }
    
    /// Get data saved
    pub fn get_data_saved(&self) -> u64 {
        self.data_saved
    }
    
    /// Record a blocked request
    pub fn record_blocked(&mut self, domain: &str, size: u64) {
        self.blocked_count += 1;
        self.data_saved += size;
        
        // Update domain stats
        let stats = self.domain_stats
            .entry(domain.to_string())
            .or_insert_with(DomainStatsInternal::default);
        stats.count += 1;
        stats.data_saved += size;
        
        // Add to recent events
        self.add_event(BlockEvent {
            timestamp: SystemTime::now(),
            domain: domain.to_string(),
            blocked: true,
            size,
        });
    }
    
    /// Record an allowed request
    pub fn record_allowed(&mut self, domain: &str, size: u64) {
        self.allowed_count += 1;
        
        // Add to recent events
        self.add_event(BlockEvent {
            timestamp: SystemTime::now(),
            domain: domain.to_string(),
            blocked: false,
            size,
        });
    }
    
    /// Add an event to recent events, maintaining size limit
    fn add_event(&mut self, event: BlockEvent) {
        self.recent_events.push(event);
        
        // Keep only the configured maximum number of events
        if self.recent_events.len() > self.config.max_recent_events {
            self.recent_events.remove(0);
        }
    }
    
    /// Get total blocked requests
    pub fn total_blocked(&self) -> u64 {
        self.blocked_count
    }
    
    /// Get total allowed requests
    pub fn total_allowed(&self) -> u64 {
        self.allowed_count
    }
    
    /// Get total data saved (in bytes)
    pub fn data_saved(&self) -> u64 {
        self.data_saved
    }
    
    /// Get top blocked domains
    pub fn top_blocked_domains(&self, limit: usize) -> Vec<DomainStats> {
        let mut domains: Vec<_> = self.domain_stats.iter()
            .map(|(domain, stats)| DomainStats {
                domain: domain.clone(),
                count: stats.count,
                data_saved: stats.data_saved,
            })
            .collect();
        
        // Sort by count (descending), then by data saved as tiebreaker
        domains.sort_by(|a, b| {
            b.count.cmp(&a.count)
                .then_with(|| b.data_saved.cmp(&a.data_saved))
        });
        
        domains.truncate(limit);
        domains
    }
    
    /// Get recent events
    pub fn recent_events(&self, limit: usize) -> Vec<BlockEvent> {
        let start = self.recent_events.len().saturating_sub(limit);
        self.recent_events[start..].iter().rev().cloned().collect()
    }
    
    /// Calculate block rate (0.0 - 1.0)
    pub fn block_rate(&self) -> f64 {
        let total = self.blocked_count + self.allowed_count;
        if total == 0 {
            0.0
        } else {
            self.blocked_count as f64 / total as f64
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        self.blocked_count = 0;
        self.allowed_count = 0;
        self.data_saved = 0;
        self.domain_stats.clear();
        self.recent_events.clear();
    }
}