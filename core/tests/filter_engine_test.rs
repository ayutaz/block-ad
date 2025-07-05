//! Filter Engine Tests - TDD implementation
//! 
//! Starting with the most basic functionality:
//! Blocking requests from known ad domains

use adblock_core::filter_engine::{FilterEngine, BlockDecision};

#[test]
fn should_block_doubleclick_domain() {
    // Given: A filter engine with default rules
    let engine = FilterEngine::new_with_defaults();
    
    // When: Checking a URL from doubleclick.net
    let decision = engine.should_block("https://doubleclick.net/ad");
    
    // Then: The request should be blocked
    assert!(decision.should_block);
    assert_eq!(decision.reason, Some("Matched ad domain: doubleclick.net".to_string()));
}

#[test]
fn should_not_block_normal_domain() {
    // Given: A filter engine with default rules
    let engine = FilterEngine::new_with_defaults();
    
    // When: Checking a normal website URL
    let decision = engine.should_block("https://example.com");
    
    // Then: The request should not be blocked
    assert!(!decision.should_block);
    assert_eq!(decision.reason, None);
}

#[test]
fn should_block_multiple_ad_domains() {
    // Given: A filter engine with default rules
    let engine = FilterEngine::new_with_defaults();
    
    // When: Checking various ad network URLs
    let test_cases = vec![
        ("https://googleadservices.com/pagead", true),
        ("https://googlesyndication.com/safeframe", true),
        ("https://facebook.com/tr", true),
        ("https://amazon-adsystem.com/widgets", true),
        ("https://google.com/search", false), // Normal Google search should not be blocked
    ];
    
    // Then: Ad domains should be blocked, normal domains should not
    for (url, should_be_blocked) in test_cases {
        let decision = engine.should_block(url);
        assert_eq!(
            decision.should_block, 
            should_be_blocked, 
            "URL {} should {} be blocked", 
            url, 
            if should_be_blocked { "" } else { "not" }
        );
    }
}