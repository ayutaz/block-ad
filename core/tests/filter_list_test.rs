//! Filter List Loading Tests - TDD implementation
//! 
//! Test loading and parsing of EasyList-format filter rules

use adblock_core::{FilterEngine, FilterListLoader};

#[test]
fn should_load_filter_list_from_string() {
    // Given: A filter list in EasyList format
    let filter_list = r#"
! Title: Test Filter List
! Version: 1.0.0
! Homepage: https://example.com

! Basic domain blocks
||doubleclick.net^
||googleadservices.com^

! Path-based blocks
/ads/*
/banner/*

! Wildcard patterns
*://ad.*
*.advertising.*

! Element hiding rules (CSS selectors)
##.ad-container
##.sponsored-content
example.com##.sidebar-ad

! Exception rules (whitelist)
@@||example.com/ads/acceptable-ads.js
@@||trusted-site.com^

! Comments should be ignored
! This is a comment
"#;

    // When: Loading the filter list
    let loader = FilterListLoader::new();
    let rules = loader.parse_filter_list(filter_list).unwrap();
    
    // Then: Rules should be correctly parsed
    assert!(rules.len() > 0);
    assert!(rules.iter().any(|r| r.contains("doubleclick.net")));
    assert!(rules.iter().any(|r| r.contains("/ads/*")));
}

#[test]
fn should_create_filter_engine_from_list() {
    // Given: A simple filter list
    let filter_list = r#"
||ads.example.com^
||tracker.com^
*/tracking/*
"#;
    
    // When: Creating a filter engine from the list
    let engine = FilterEngine::from_filter_list(filter_list).unwrap();
    
    // Then: Engine should block according to the rules
    assert!(engine.should_block("https://ads.example.com/banner").should_block);
    assert!(engine.should_block("https://tracker.com/pixel").should_block);
    assert!(engine.should_block("https://site.com/tracking/user").should_block);
    assert!(!engine.should_block("https://example.com/content").should_block);
}

#[test]
fn should_handle_exception_rules() {
    // Given: A filter list with exception rules
    let filter_list = r#"
||ads.com^
@@||ads.com/acceptable/*
"#;
    
    // When: Creating a filter engine
    let engine = FilterEngine::from_filter_list(filter_list).unwrap();
    
    // Then: Exception rules should override blocking rules
    assert!(engine.should_block("https://ads.com/banner").should_block);
    assert!(!engine.should_block("https://ads.com/acceptable/content").should_block);
}

#[test]
fn should_load_filter_list_from_file() {
    // Given: A path to a filter list file
    use std::fs;
    use std::io::Write;
    
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("test_filters.txt");
    
    let mut file = fs::File::create(&file_path).unwrap();
    writeln!(file, "||badsite.com^").unwrap();
    writeln!(file, "||malware.com^").unwrap();
    
    // When: Loading from file
    let engine = FilterEngine::from_file(&file_path).unwrap();
    
    // Then: Rules should be loaded
    assert!(engine.should_block("https://badsite.com/").should_block);
    assert!(engine.should_block("https://malware.com/script").should_block);
    
    // Cleanup
    fs::remove_file(&file_path).ok();
}

#[test]
fn should_handle_malformed_rules_gracefully() {
    // Given: A filter list with some malformed rules
    let filter_list = r#"
||valid-domain.com^
[invalid rule format]
||another-valid.com^
!!!invalid
/valid-path/*
"#;
    
    // When: Parsing the list
    let loader = FilterListLoader::new();
    let rules = loader.parse_filter_list(filter_list).unwrap();
    
    // Then: Valid rules should be parsed, invalid ones ignored
    assert!(rules.iter().any(|r| r.contains("valid-domain.com")));
    assert!(rules.iter().any(|r| r.contains("another-valid.com")));
    assert!(rules.iter().any(|r| r.contains("/valid-path/*")));
    assert!(!rules.iter().any(|r| r.contains("[invalid")));
}

#[test]
fn should_support_css_element_hiding() {
    // Given: A filter list with CSS element hiding rules
    let filter_list = r#"
##.advertisement
example.com##.banner
~example.com##.sidebar-ad
"#;
    
    // When: Parsing CSS rules
    let loader = FilterListLoader::new();
    let css_rules = loader.get_css_rules(filter_list, "example.com").unwrap();
    
    // Then: Appropriate CSS rules should be returned
    assert!(css_rules.iter().any(|r| r == ".advertisement"));
    assert!(css_rules.iter().any(|r| r == ".banner"));
    assert!(!css_rules.iter().any(|r| r == ".sidebar-ad")); // excluded by ~example.com
}