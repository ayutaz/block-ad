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

/// Statistics tracker for the ad blocker
#[derive(Debug, Default)]
pub struct Statistics {
    blocked_count: u64,
    allowed_count: u64,
    data_saved: u64,
    domain_stats: HashMap<String, (u64, u64)>, // (count, data_saved)
    recent_events: Vec<BlockEvent>,
}

impl Statistics {
    /// Create a new statistics instance
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a blocked request
    pub fn record_blocked(&mut self, domain: &str, size: u64) {
        self.blocked_count += 1;
        self.data_saved += size;
        
        // Update domain stats
        let entry = self.domain_stats.entry(domain.to_string()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += size;
        
        // Add to recent events
        self.recent_events.push(BlockEvent {
            timestamp: SystemTime::now(),
            domain: domain.to_string(),
            blocked: true,
            size,
        });
        
        // Keep only last 1000 events
        if self.recent_events.len() > 1000 {
            self.recent_events.remove(0);
        }
    }
    
    /// Record an allowed request
    pub fn record_allowed(&mut self, domain: &str, size: u64) {
        self.allowed_count += 1;
        
        // Add to recent events
        self.recent_events.push(BlockEvent {
            timestamp: SystemTime::now(),
            domain: domain.to_string(),
            blocked: false,
            size,
        });
        
        // Keep only last 1000 events
        if self.recent_events.len() > 1000 {
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
            .map(|(domain, (count, data))| DomainStats {
                domain: domain.clone(),
                count: *count,
                data_saved: *data,
            })
            .collect();
        
        // Sort by count (descending)
        domains.sort_by(|a, b| b.count.cmp(&a.count));
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