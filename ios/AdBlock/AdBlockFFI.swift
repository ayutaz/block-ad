import Foundation

// FFI function declarations for the Rust library

@_silgen_name("adblock_engine_create")
func adblock_engine_create() -> UnsafeMutableRawPointer?

@_silgen_name("adblock_engine_destroy")
func adblock_engine_destroy(_ engine: UnsafeMutableRawPointer)

@_silgen_name("adblock_engine_should_block")
func adblock_engine_should_block(_ engine: UnsafeMutableRawPointer, _ url: UnsafePointer<CChar>) -> Bool

@_silgen_name("adblock_engine_load_filter_list")
func adblock_engine_load_filter_list(_ engine: UnsafeMutableRawPointer, _ filterList: UnsafePointer<CChar>) -> Bool

@_silgen_name("adblock_engine_get_stats")
func adblock_engine_get_stats(_ engine: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>?

@_silgen_name("adblock_engine_reset_stats")
func adblock_engine_reset_stats(_ engine: UnsafeMutableRawPointer) -> Bool

@_silgen_name("adblock_engine_get_metrics")
func adblock_engine_get_metrics(_ engine: UnsafeMutableRawPointer) -> UnsafeMutablePointer<CChar>?

@_silgen_name("adblock_free_string")
func adblock_free_string(_ string: UnsafeMutablePointer<CChar>)