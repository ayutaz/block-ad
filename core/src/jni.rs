//! JNI bindings for Android
//!
//! This module provides JNI-compatible function signatures that wrap the FFI functions

#![cfg(target_os = "android")]

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::ffi::CString;

use crate::ffi;

#[no_mangle]
pub extern "system" fn Java_com_adblock_AdBlockEngine_nativeCreate(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    ffi::adblock_engine_create() as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_adblock_AdBlockEngine_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    ffi::adblock_engine_destroy(handle as *mut std::ffi::c_void);
}

#[no_mangle]
pub extern "system" fn Java_com_adblock_AdBlockEngine_nativeShouldBlock(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    url: JString,
) -> jboolean {
    let engine = handle as *mut std::ffi::c_void;
    if engine.is_null() {
        return JNI_FALSE;
    }

    let url_str = match env.get_string(url) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    let url_cstr = match CString::new(url_str.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    let should_block = ffi::adblock_engine_should_block(engine, url_cstr.as_ptr());
    if should_block {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_adblock_AdBlockEngine_nativeLoadFilterList(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    filter_list: JString,
) -> jboolean {
    let engine = handle as *mut std::ffi::c_void;
    if engine.is_null() {
        return JNI_FALSE;
    }

    let filter_str = match env.get_string(filter_list) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    let filter_cstr = match CString::new(filter_str.to_string_lossy().as_bytes()) {
        Ok(s) => s,
        Err(_) => return JNI_FALSE,
    };

    let success = ffi::adblock_engine_load_filter_list(engine, filter_cstr.as_ptr());
    if success {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}

#[no_mangle]
pub extern "system" fn Java_com_adblock_AdBlockEngine_nativeGetStats(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let engine = handle as *mut std::ffi::c_void;
    if engine.is_null() {
        return std::ptr::null_mut();
    }

    let stats_ptr = ffi::adblock_engine_get_stats(engine);
    if stats_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let stats_cstr = unsafe { std::ffi::CStr::from_ptr(stats_ptr) };
    let result = match env.new_string(stats_cstr.to_string_lossy()) {
        Ok(s) => s.into_inner(),
        Err(_) => std::ptr::null_mut(),
    };

    unsafe { ffi::adblock_free_string(stats_ptr as *mut std::os::raw::c_char) };
    result
}

#[no_mangle]
pub extern "system" fn Java_com_adblock_AdBlockEngine_nativeResetStats(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jboolean {
    let engine = handle as *mut std::ffi::c_void;
    if engine.is_null() {
        return JNI_FALSE;
    }

    let success = ffi::adblock_engine_reset_stats(engine);
    if success {
        JNI_TRUE
    } else {
        JNI_FALSE
    }
}
