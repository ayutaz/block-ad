//! FFI Binding Tests - TDD implementation
//! 
//! Test C-compatible API for Android/iOS integration

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

// These functions will be implemented in ffi.rs
extern "C" {
    fn adblock_engine_create() -> *mut c_void;
    fn adblock_engine_destroy(engine: *mut c_void);
    fn adblock_engine_should_block(engine: *mut c_void, url: *const c_char) -> bool;
    fn adblock_engine_add_rule(engine: *mut c_void, rule: *const c_char) -> bool;
    fn adblock_engine_load_filter_list(engine: *mut c_void, filter_list: *const c_char) -> bool;
    fn adblock_engine_get_stats(engine: *mut c_void) -> *mut c_char;
    fn adblock_free_string(s: *mut c_char);
}

#[test]
fn should_create_and_destroy_engine() {
    unsafe {
        // Given: Create an engine
        let engine = adblock_engine_create();
        
        // Then: Engine should not be null
        assert!(!engine.is_null());
        
        // When: Destroy the engine
        adblock_engine_destroy(engine);
        // Should not crash
    }
}

#[test]
fn should_block_urls_through_ffi() {
    unsafe {
        // Given: An engine with rules
        let engine = adblock_engine_create();
        assert!(!engine.is_null());
        
        // When: Adding a rule
        let rule = CString::new("||doubleclick.net^").unwrap();
        let result = adblock_engine_add_rule(engine, rule.as_ptr());
        assert!(result);
        
        // Then: Should block matching URLs
        let test_url = CString::new("https://doubleclick.net/ads").unwrap();
        assert!(adblock_engine_should_block(engine, test_url.as_ptr()));
        
        // And: Should not block non-matching URLs
        let safe_url = CString::new("https://example.com").unwrap();
        assert!(!adblock_engine_should_block(engine, safe_url.as_ptr()));
        
        adblock_engine_destroy(engine);
    }
}

#[test]
fn should_load_filter_list_through_ffi() {
    unsafe {
        // Given: An engine
        let engine = adblock_engine_create();
        
        // When: Loading a filter list
        let filter_list = CString::new(r#"
||ads.example.com^
||tracker.com^
*/banner/*
"#).unwrap();
        
        let result = adblock_engine_load_filter_list(engine, filter_list.as_ptr());
        assert!(result);
        
        // Then: Rules should be active
        let test_url = CString::new("https://ads.example.com/img").unwrap();
        assert!(adblock_engine_should_block(engine, test_url.as_ptr()));
        
        adblock_engine_destroy(engine);
    }
}

#[test]
fn should_get_statistics_as_json() {
    unsafe {
        // Given: An engine with some activity
        let engine = adblock_engine_create();
        
        // Add rules and check URLs to generate stats
        let rule = CString::new("||ads.com^").unwrap();
        adblock_engine_add_rule(engine, rule.as_ptr());
        
        let blocked_url = CString::new("https://ads.com/banner").unwrap();
        let safe_url = CString::new("https://safe.com").unwrap();
        
        adblock_engine_should_block(engine, blocked_url.as_ptr());
        adblock_engine_should_block(engine, safe_url.as_ptr());
        
        // When: Getting statistics
        let stats_ptr = adblock_engine_get_stats(engine);
        assert!(!stats_ptr.is_null());
        
        let stats_cstr = CStr::from_ptr(stats_ptr);
        let stats_str = stats_cstr.to_str().unwrap();
        
        // Then: Should contain valid JSON
        assert!(stats_str.contains("blocked_count"));
        assert!(stats_str.contains("allowed_count"));
        
        // Clean up
        adblock_free_string(stats_ptr);
        adblock_engine_destroy(engine);
    }
}

#[test]
fn should_handle_null_safety() {
    unsafe {
        // Should handle null engine gracefully
        assert!(!adblock_engine_should_block(std::ptr::null_mut(), std::ptr::null()));
        
        // Should handle null URL
        let engine = adblock_engine_create();
        assert!(!adblock_engine_should_block(engine, std::ptr::null()));
        
        // Should handle null rule
        assert!(!adblock_engine_add_rule(engine, std::ptr::null()));
        
        adblock_engine_destroy(engine);
    }
}

#[test]
fn should_be_thread_safe() {
    use std::thread;
    
    unsafe {
        // Given: Create engines in each thread
        let mut handles = vec![];
        
        // When: Multiple threads create and use their own engines
        for i in 0..5 {
            let handle = thread::spawn(move || {
                unsafe {
                    let engine = adblock_engine_create();
                    
                    // Add a rule
                    let rule = CString::new("||ads.com^").unwrap();
                    adblock_engine_add_rule(engine, rule.as_ptr());
                    
                    // Test blocking
                    let url = CString::new(format!("https://ads.com/thread{}", i)).unwrap();
                    let result = adblock_engine_should_block(engine, url.as_ptr());
                    
                    // Clean up
                    adblock_engine_destroy(engine);
                    
                    result
                }
            });
            handles.push(handle);
        }
        
        // Then: All threads should complete successfully
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result);
        }
    }
}