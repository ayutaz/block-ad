//
//  FFI.swift
//  AdBlock
//
//  FFI bindings for Rust functions
//

import Foundation

// Function declarations for Rust FFI
@_silgen_name("adblock_engine_create")
func adblock_engine_create() -> OpaquePointer?

@_silgen_name("adblock_engine_destroy")
func adblock_engine_destroy(_ engine: OpaquePointer)

@_silgen_name("adblock_engine_should_block")
func adblock_engine_should_block(_ engine: OpaquePointer, _ url: UnsafePointer<CChar>) -> Bool

@_silgen_name("adblock_engine_load_filter_list")
func adblock_engine_load_filter_list(_ engine: OpaquePointer, _ filterList: UnsafePointer<CChar>) -> Bool

@_silgen_name("adblock_engine_get_stats")
func adblock_engine_get_stats(_ engine: OpaquePointer) -> UnsafeMutablePointer<CChar>?

@_silgen_name("adblock_free_string")
func adblock_free_string(_ s: UnsafeMutablePointer<CChar>)