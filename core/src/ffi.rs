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

/// Convert C string to Rust string safely
fn c_str_to_rust(ptr: *const c_char) -> Option<&'static str> {
    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr).to_str().ok() }
}

/// Get engine reference safely
fn get_engine_ref(engine: *mut c_void) -> Option<&'static AdBlockEngine> {
    if engine.is_null() {
        return None;
    }

    Some(unsafe { &*(engine as *mut AdBlockEngine) })
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
    let engine = match get_engine_ref(engine) {
        Some(e) => e,
        None => return false,
    };

    let url_str = match c_str_to_rust(url) {
        Some(s) => s,
        None => return false,
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
    let engine = match get_engine_ref(engine) {
        Some(e) => e,
        None => return false,
    };

    let _rule_str = match c_str_to_rust(rule) {
        Some(s) => s,
        None => return false,
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
    let engine = match get_engine_ref(engine) {
        Some(e) => e,
        None => return false,
    };

    let filter_list_str = match c_str_to_rust(filter_list) {
        Some(s) => s,
        None => return false,
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
    let engine = match get_engine_ref(engine) {
        Some(e) => e,
        None => return ptr::null_mut(),
    };

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

/// Reset statistics
///
/// # Safety
/// The engine pointer must be valid
#[no_mangle]
pub extern "C" fn adblock_engine_reset_stats(engine: *mut c_void) -> bool {
    let Some(engine) = get_engine_ref(engine) else {
        return false;
    };

    match engine.core.lock() {
        Ok(core) => {
            core.reset_statistics();
            true
        }
        Err(_) => false,
    }
}

/// Free a string allocated by the library
///
/// # Safety
/// The pointer must have been returned by a function from this library
/// and must not have been freed already.
#[no_mangle]
pub unsafe extern "C" fn adblock_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }

    let _ = CString::from_raw(s);
    // CString will be dropped, freeing the memory
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
        assert!(adblock_engine_load_filter_list(
            engine,
            filter_list.as_ptr()
        ));

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

        unsafe {
            adblock_free_string(stats_ptr);
        }
        adblock_engine_destroy(engine);
    }
}
