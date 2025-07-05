import Foundation

/// Swift wrapper for the Rust ad blocking engine
/// Provides thread-safe access to the underlying Rust engine
public final class AdBlockEngine {
    private let engineHandle: OpaquePointer
    private let queue = DispatchQueue(label: "com.adblock.engine", attributes: .concurrent)
    
    /// Check if engine is initialized
    public var isInitialized: Bool {
        return true
    }
    
    /// Initialize the engine
    /// - Throws: EngineError if initialization fails
    public init() throws {
        guard let handle = adblock_engine_create() else {
            throw EngineError.initializationFailed
        }
        self.engineHandle = handle
    }
    
    deinit {
        adblock_engine_destroy(engineHandle)
    }
    
    /// Check if a URL should be blocked
    /// - Parameter url: The URL to check
    /// - Returns: true if the URL should be blocked
    public func shouldBlock(_ url: String) -> Bool {
        return queue.sync {
            return adblock_engine_should_block(engineHandle, url)
        }
    }
    
    /// Load filter rules from a string
    /// - Parameter filterList: Filter rules in EasyList format
    /// - Returns: true if loading succeeded
    @discardableResult
    public func loadFilterList(_ filterList: String) -> Bool {
        return queue.sync(flags: .barrier) {
            return adblock_engine_load_filter_list(engineHandle, filterList)
        }
    }
    
    /// Get current statistics
    /// - Returns: Statistics object with blocking metrics
    public func getStatistics() -> Statistics {
        return queue.sync {
            var blocked: UInt64 = 0
            var allowed: UInt64 = 0
            var saved: UInt64 = 0
            
            adblock_engine_get_stats(engineHandle, &blocked, &allowed, &saved)
            
            return Statistics(
                blockedCount: Int(blocked),
                allowedCount: Int(allowed),
                dataSaved: Int(saved)
            )
        }
    }
}

/// Errors that can occur during engine operations
public enum EngineError: Error {
    case initializationFailed
}