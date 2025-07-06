//! Filter Engine Tests - TDD implementation
//!
//! Starting with the most basic functionality:
//! Blocking requests from known ad domains

use adblock_core::filter_engine::FilterEngine;

#[test]
fn should_block_doubleclick_domain() {
    // Given: A filter engine with default rules
    let engine = FilterEngine::new_with_defaults();

    // When: Checking a URL from doubleclick.net
    let decision = engine.should_block("https://doubleclick.net/ad");

    // Then: The request should be blocked
    assert!(decision.should_block);
    assert_eq!(
        decision.reason,
        Some("Matched ad domain: doubleclick.net".to_string())
    );
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

#[test]
fn should_support_wildcard_patterns() {
    // Given: A filter engine with wildcard patterns
    let engine = FilterEngine::new_with_patterns(vec![
        "*/ads/*".to_string(),
        "*.doubleclick.*".to_string(),
        "*://ad.*".to_string(),
    ]);

    // When & Then: Check various URLs against wildcard patterns

    // Should block URLs matching */ads/*
    assert!(
        engine
            .should_block("https://example.com/ads/banner.jpg")
            .should_block
    );
    assert!(
        engine
            .should_block("http://site.com/static/ads/video.mp4")
            .should_block
    );

    // Should block URLs matching *.doubleclick.*
    assert!(
        engine
            .should_block("https://stats.doubleclick.net/track")
            .should_block
    );
    assert!(
        engine
            .should_block("http://ad.doubleclick.com/pixel")
            .should_block
    );

    // Should block URLs matching *://ad.*
    assert!(
        engine
            .should_block("https://ad.example.com/banner")
            .should_block
    );
    assert!(engine.should_block("http://ad.site.org/track").should_block);

    // Should NOT block URLs that don't match patterns
    assert!(
        !engine
            .should_block("https://example.com/content")
            .should_block
    );
    assert!(
        !engine
            .should_block("https://addons.mozilla.org")
            .should_block
    ); // Contains "ad" but not matching pattern
}

#[test]
fn should_match_subdomain_patterns() {
    // Given: A filter engine with subdomain patterns
    let engine = FilterEngine::new_with_patterns(vec![
        "||doubleclick.net^".to_string(), // Match domain and all subdomains
    ]);

    // When & Then: Test subdomain matching
    assert!(engine.should_block("https://doubleclick.net/").should_block);
    assert!(
        engine
            .should_block("https://ads.doubleclick.net/pixel")
            .should_block
    );
    assert!(
        engine
            .should_block("https://stats.g.doubleclick.net/track")
            .should_block
    );

    // Should NOT block similar but different domains
    assert!(
        !engine
            .should_block("https://notdoubleclick.net/")
            .should_block
    );
    assert!(!engine.should_block("https://doubleclick.com/").should_block);
}
