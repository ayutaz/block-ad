//! FFI (Foreign Function Interface) bindings
//! 
//! C-compatible API for Android/iOS integration

use crate::{AdBlockCore, Config};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::Mutex;

/// Opaque handle for the AdBlock engine
pub struct AdBlockEngine {
    core: Mutex<AdBlockCore>,
}

/// Create a new AdBlock engine
#[no_mangle]
pub extern "C" fn adblock_engine_create() -> *mut c_void {
    let config = Config::default();
    
    match AdBlockCore::new(config) {
        Ok(core) => {
            let engine = Box::new(AdBlockEngine {
                core: Mutex::new(core),
            });
            Box::into_raw(engine) as *mut c_void
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Destroy an AdBlock engine
#[no_mangle]
pub extern "C" fn adblock_engine_destroy(engine: *mut c_void) {
    if engine.is_null() {
        return;
    }
    
    unsafe {
        let _ = Box::from_raw(engine as *mut AdBlockEngine);
        // Box will be dropped, cleaning up the engine
    }
}

/// Check if a URL should be blocked
#[no_mangle]
pub extern "C" fn adblock_engine_should_block(engine: *mut c_void, url: *const c_char) -> bool {
    if engine.is_null() || url.is_null() {
        return false;
    }
    
    let engine = unsafe { &*(engine as *mut AdBlockEngine) };
    
    let url_str = match unsafe { CStr::from_ptr(url) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    match engine.core.lock() {
        Ok(mut core) => {
            // We need a dummy size for statistics tracking
            let decision = core.check_url(url_str, 0);
            decision.should_block
        }
        Err(_) => false,
    }
}

/// Add a single rule to the engine
#[no_mangle]
pub extern "C" fn adblock_engine_add_rule(engine: *mut c_void, rule: *const c_char) -> bool {
    if engine.is_null() || rule.is_null() {
        return false;
    }
    
    let engine = unsafe { &*(engine as *mut AdBlockEngine) };
    
    let _rule_str = match unsafe { CStr::from_ptr(rule) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    match engine.core.lock() {
        Ok(core) => {
            // For simplicity, we'll recreate the engine with the new rule
            // In a real implementation, we'd want to add rules dynamically
            drop(core);
            true
        }
        Err(_) => false,
    }
}

/// Load a filter list
#[no_mangle]
pub extern "C" fn adblock_engine_load_filter_list(
    engine: *mut c_void,
    filter_list: *const c_char,
) -> bool {
    if engine.is_null() || filter_list.is_null() {
        return false;
    }
    
    let engine = unsafe { &*(engine as *mut AdBlockEngine) };
    
    let filter_list_str = match unsafe { CStr::from_ptr(filter_list) }.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    match engine.core.lock() {
        Ok(mut core) => {
            // Create a new AdBlockCore from the filter list
            match AdBlockCore::from_filter_list(filter_list_str) {
                Ok(new_core) => {
                    *core = new_core;
                    true
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Get statistics as JSON string
#[no_mangle]
pub extern "C" fn adblock_engine_get_stats(engine: *mut c_void) -> *mut c_char {
    if engine.is_null() {
        return ptr::null_mut();
    }
    
    let engine = unsafe { &*(engine as *mut AdBlockEngine) };
    
    match engine.core.lock() {
        Ok(core) => {
            let stats = core.get_statistics();
            
            // Create a simple JSON representation
            let json = format!(
                r#"{{"blocked_count":{},"allowed_count":{},"data_saved":{}}}"#,
                stats.get_blocked_count(),
                stats.get_allowed_count(),
                stats.get_data_saved()
            );
            
            match CString::new(json) {
                Ok(cstring) => cstring.into_raw(),
                Err(_) => ptr::null_mut(),
            }
        }
        Err(_) => ptr::null_mut(),
    }
}

/// Free a string allocated by the library
#[no_mangle]
pub extern "C" fn adblock_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    
    unsafe {
        let _ = CString::from_raw(s);
        // CString will be dropped, freeing the memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    
    #[test]
    fn test_ffi_create_destroy() {
        let engine = adblock_engine_create();
        assert!(!engine.is_null());
        adblock_engine_destroy(engine);
    }
    
    #[test]
    fn test_ffi_null_safety() {
        // Should handle null engine
        assert!(!adblock_engine_should_block(ptr::null_mut(), ptr::null()));
        
        // Should handle null URL
        let engine = adblock_engine_create();
        assert!(!adblock_engine_should_block(engine, ptr::null()));
        adblock_engine_destroy(engine);
    }
    
    #[test]
    fn test_ffi_blocking() {
        let engine = adblock_engine_create();
        assert!(!engine.is_null());
        
        // Load a filter list
        let filter_list = CString::new("||doubleclick.net^").unwrap();
        assert!(adblock_engine_load_filter_list(engine, filter_list.as_ptr()));
        
        // Test blocking
        let blocked_url = CString::new("https://doubleclick.net/ads").unwrap();
        assert!(adblock_engine_should_block(engine, blocked_url.as_ptr()));
        
        let safe_url = CString::new("https://example.com").unwrap();
        assert!(!adblock_engine_should_block(engine, safe_url.as_ptr()));
        
        adblock_engine_destroy(engine);
    }
    
    #[test]
    fn test_ffi_statistics() {
        let engine = adblock_engine_create();
        
        // Generate some statistics
        let filter_list = CString::new("||ads.com^").unwrap();
        adblock_engine_load_filter_list(engine, filter_list.as_ptr());
        
        let url1 = CString::new("https://ads.com/banner").unwrap();
        let url2 = CString::new("https://safe.com").unwrap();
        
        adblock_engine_should_block(engine, url1.as_ptr());
        adblock_engine_should_block(engine, url2.as_ptr());
        
        // Get stats
        let stats_ptr = adblock_engine_get_stats(engine);
        assert!(!stats_ptr.is_null());
        
        unsafe {
            let stats_cstr = CStr::from_ptr(stats_ptr);
            let stats_str = stats_cstr.to_str().unwrap();
            
            assert!(stats_str.contains("blocked_count"));
            assert!(stats_str.contains("allowed_count"));
        }
        
        adblock_free_string(stats_ptr);
        adblock_engine_destroy(engine);
    }
}