//! Integration Tests - FilterEngine with Statistics
//! 
//! Test the integration between filtering and statistics tracking

use adblock_core::{AdBlockCore, Config};

#[test]
fn should_track_statistics_when_blocking() {
    // Given: An AdBlockCore instance with default config
    let config = Config::default();
    let mut core = AdBlockCore::new(config).expect("Failed to create core");
    
    // When: Checking URLs that should be blocked and allowed
    let test_cases = vec![
        ("https://doubleclick.net/ad", true, 1024),
        ("https://example.com", false, 512),
        ("https://googleads.com/track", true, 2048),
        ("https://github.com", false, 1024),
    ];
    
    for (url, should_block, size) in test_cases {
        let decision = core.check_url(url, size);
        assert_eq!(decision.should_block, should_block, "URL: {}", url);
    }
    
    // Then: Statistics should be correctly tracked
    let stats = core.get_statistics();
    assert_eq!(stats.total_blocked(), 2);
    assert_eq!(stats.total_allowed(), 2);
    assert_eq!(stats.data_saved(), 3072); // 1024 + 2048
}

#[test]
fn should_track_domain_specific_statistics() {
    // Given: An AdBlockCore instance
    let mut core = AdBlockCore::new(Config::default()).unwrap();
    
    // When: Multiple requests from same domains
    core.check_url("https://ads.doubleclick.net/1", 100);
    core.check_url("https://ads.doubleclick.net/2", 200);
    core.check_url("https://tracker.facebook.com/tr", 150);
    
    // Then: Domain statistics should be aggregated
    let stats = core.get_statistics();
    let top_domains = stats.top_blocked_domains(10);
    
    // Should have blocked domains in statistics
    assert!(top_domains.iter().any(|d| d.domain.contains("doubleclick")));
    assert!(top_domains.iter().any(|d| d.domain.contains("facebook")));
}

#[test]
fn should_handle_pattern_matching_with_statistics() {
    // Given: An AdBlockCore with pattern rules
    let mut core = AdBlockCore::with_patterns(vec![
        "*/ads/*".to_string(),
        "||analytics.^".to_string(),
    ]).unwrap();
    
    // When: Checking URLs matching patterns
    core.check_url("https://example.com/ads/banner.jpg", 500);
    core.check_url("https://analytics.google.com/collect", 300);
    core.check_url("https://example.com/content/image.jpg", 1000);
    
    // Then: Pattern-matched URLs should be tracked
    let stats = core.get_statistics();
    assert_eq!(stats.total_blocked(), 2);
    assert_eq!(stats.total_allowed(), 1);
    assert_eq!(stats.data_saved(), 800); // 500 + 300
}

#[test]
fn should_provide_recent_blocking_history() {
    // Given: An AdBlockCore instance
    let mut core = AdBlockCore::new(Config::default()).unwrap();
    
    // When: Making several requests
    core.check_url("https://ad1.com", 100);
    core.check_url("https://example.com", 200);
    core.check_url("https://ad2.com", 300);
    
    // Then: Recent events should be available
    let stats = core.get_statistics();
    let recent = stats.recent_events(5);
    
    assert_eq!(recent.len(), 3);
    // Recent events should be in reverse chronological order
    assert_eq!(recent[0].domain, "ad2.com");
    assert_eq!(recent[1].domain, "example.com");
    assert_eq!(recent[2].domain, "ad1.com");
}