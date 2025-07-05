//! Statistics Tests - TDD implementation
//! 
//! Track blocking statistics and provide insights

use adblock_core::{Statistics, BlockEvent};
use std::time::{Duration, SystemTime};

#[test]
fn should_track_basic_statistics() {
    // Given: A new statistics instance
    let mut stats = Statistics::new();
    
    // When: Recording blocked and allowed requests
    stats.record_blocked("doubleclick.net", 1024);
    stats.record_blocked("googleads.com", 2048);
    stats.record_allowed("example.com", 512);
    
    // Then: Statistics should be correctly tracked
    assert_eq!(stats.total_blocked(), 2);
    assert_eq!(stats.total_allowed(), 1);
    assert_eq!(stats.data_saved(), 3072); // 1024 + 2048
}

#[test]
fn should_track_domain_statistics() {
    // Given: A statistics instance with multiple events
    let mut stats = Statistics::new();
    
    // When: Recording multiple blocks for same domains
    stats.record_blocked("ads.com", 100);
    stats.record_blocked("ads.com", 200);
    stats.record_blocked("tracker.com", 150);
    stats.record_blocked("ads.com", 300);
    
    // Then: Domain statistics should be aggregated
    let top_domains = stats.top_blocked_domains(10);
    
    assert_eq!(top_domains.len(), 2);
    assert_eq!(top_domains[0].domain, "ads.com");
    assert_eq!(top_domains[0].count, 3);
    assert_eq!(top_domains[0].data_saved, 600); // 100 + 200 + 300
    
    assert_eq!(top_domains[1].domain, "tracker.com");
    assert_eq!(top_domains[1].count, 1);
    assert_eq!(top_domains[1].data_saved, 150);
}

#[test]
fn should_get_recent_block_events() {
    // Given: A statistics instance
    let mut stats = Statistics::new();
    
    // When: Recording several events
    stats.record_blocked("ad1.com", 100);
    std::thread::sleep(Duration::from_millis(10));
    stats.record_blocked("ad2.com", 200);
    std::thread::sleep(Duration::from_millis(10));
    stats.record_allowed("good.com", 300);
    
    // Then: Recent events should be retrievable
    let recent = stats.recent_events(2);
    
    assert_eq!(recent.len(), 2);
    // Most recent first
    assert_eq!(recent[0].domain, "good.com");
    assert!(!recent[0].blocked);
    assert_eq!(recent[1].domain, "ad2.com");
    assert!(recent[1].blocked);
}

#[test]
fn should_calculate_block_rate() {
    // Given: A statistics instance with mixed events
    let mut stats = Statistics::new();
    
    // When: Recording a mix of blocked and allowed
    for _ in 0..80 {
        stats.record_blocked("ad.com", 100);
    }
    for _ in 0..20 {
        stats.record_allowed("site.com", 100);
    }
    
    // Then: Block rate should be calculated correctly
    assert_eq!(stats.block_rate(), 0.8); // 80%
}

#[test]
fn should_reset_statistics() {
    // Given: A statistics instance with data
    let mut stats = Statistics::new();
    stats.record_blocked("ad.com", 1000);
    stats.record_allowed("site.com", 500);
    
    // When: Resetting statistics
    stats.reset();
    
    // Then: All counters should be zero
    assert_eq!(stats.total_blocked(), 0);
    assert_eq!(stats.total_allowed(), 0);
    assert_eq!(stats.data_saved(), 0);
    assert_eq!(stats.recent_events(10).len(), 0);
}