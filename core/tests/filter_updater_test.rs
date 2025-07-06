//! Filter Updater Tests - TDD implementation
//!
//! Test automatic filter list updates from remote sources

use adblock_core::{FilterUpdater, UpdateConfig};
use std::time::Duration;

#[test]
fn should_download_filter_list_from_url() {
    // Given: A filter updater with a test URL
    let config = UpdateConfig {
        urls: vec!["https://example.com/filters.txt".to_string()],
        update_interval: Duration::from_secs(3600),
        cache_dir: None,
    };

    let mut updater = FilterUpdater::new(config).unwrap();

    // When: Checking if update is needed
    let needs_update = updater.needs_update();

    // Then: Should need update on first run
    assert!(needs_update);

    // When: Updating filters (this would normally download from URL)
    // For testing, we'll use a mock response
    let mock_filter_content = r#"
! Test Filter List
||ads.example.com^
||tracker.com^
"#;

    let result = updater.update_with_content(mock_filter_content);

    // Then: Update should succeed
    assert!(result.is_ok());
}

#[test]
fn should_cache_downloaded_filter_lists() {
    // Given: A filter updater with cache directory
    let temp_dir = std::env::temp_dir().join("adblock_test_cache");
    std::fs::create_dir_all(&temp_dir).ok();

    let config = UpdateConfig {
        urls: vec!["https://example.com/filters.txt".to_string()],
        update_interval: Duration::from_secs(3600),
        cache_dir: Some(temp_dir.clone()),
    };

    let mut updater = FilterUpdater::new(config).unwrap();

    // When: Updating with content
    let filter_content = "||ads.example.com^";
    updater.update_with_content(filter_content).unwrap();

    // Then: Cache file should exist
    let cache_file = temp_dir.join("filters_cache.txt");
    assert!(cache_file.exists());

    // And: Cache content should match
    let cached_content = std::fs::read_to_string(&cache_file).unwrap();
    assert!(cached_content.contains("ads.example.com"));

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn should_respect_update_interval() {
    // Given: A filter updater that was recently updated
    let config = UpdateConfig {
        urls: vec!["https://example.com/filters.txt".to_string()],
        update_interval: Duration::from_secs(3600), // 1 hour
        cache_dir: None,
    };

    let mut updater = FilterUpdater::new(config).unwrap();

    // When: Updating once
    updater.update_with_content("||ads.com^").unwrap();

    // Then: Should not need update immediately after
    assert!(!updater.needs_update());
}

#[test]
fn should_merge_multiple_filter_lists() {
    // Given: Multiple filter list URLs
    let config = UpdateConfig {
        urls: vec![
            "https://example.com/filters1.txt".to_string(),
            "https://example.com/filters2.txt".to_string(),
        ],
        update_interval: Duration::from_secs(3600),
        cache_dir: None,
    };

    let updater = FilterUpdater::new(config).unwrap();

    // When: Updating with multiple filter lists
    let filters1 = "||ads1.com^";
    let filters2 = "||ads2.com^";

    let merged = updater.merge_filter_lists(vec![filters1, filters2]);

    // Then: Merged list should contain both
    assert!(merged.contains("ads1.com"));
    assert!(merged.contains("ads2.com"));
}

#[test]
fn should_handle_download_failures_gracefully() {
    // Given: A filter updater with invalid URL
    let config = UpdateConfig {
        urls: vec!["https://invalid.example.com/nonexistent".to_string()],
        update_interval: Duration::from_secs(3600),
        cache_dir: None,
    };

    let updater = FilterUpdater::new(config).unwrap();

    // When: Attempting to download (simulated failure)
    let result = updater.download_filter_list("https://invalid.example.com/nonexistent");

    // Then: Should return error but not panic
    assert!(result.is_err());
}

#[test]
fn should_use_cached_filters_when_download_fails() {
    // Given: A filter updater with cache
    let temp_dir = std::env::temp_dir().join("adblock_fallback_cache");
    std::fs::create_dir_all(&temp_dir).ok();

    // Pre-populate cache
    let cache_file = temp_dir.join("filters_cache.txt");
    std::fs::write(&cache_file, "||cached-ads.com^").unwrap();

    let config = UpdateConfig {
        urls: vec!["https://invalid.example.com/filters.txt".to_string()],
        update_interval: Duration::from_secs(3600),
        cache_dir: Some(temp_dir.clone()),
    };

    let updater = FilterUpdater::new(config).unwrap();

    // When: Loading from cache
    let filters = updater.load_from_cache().unwrap();

    // Then: Should load cached filters
    assert!(filters.contains("cached-ads.com"));

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}
