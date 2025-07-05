import Foundation

/// Swift wrapper for the Rust ad blocking engine
public class AdBlockEngine {
    private var engineHandle: OpaquePointer?
    private let queue = DispatchQueue(label: "com.adblock.engine", attributes: .concurrent)
    
    /// Check if engine is initialized
    public var isInitialized: Bool {
        return engineHandle != nil
    }
    
    /// Initialize the engine
    public init() {
        self.engineHandle = adblock_engine_create()
    }
    
    deinit {
        if let handle = engineHandle {
            adblock_engine_destroy(handle)
        }
    }
    
    /// Check if a URL should be blocked
    /// - Parameter url: The URL to check
    /// - Returns: true if the URL should be blocked
    public func shouldBlock(_ url: String) -> Bool {
        guard let handle = engineHandle else { return false }
        
        return queue.sync {
            return adblock_engine_should_block(handle, url)
        }
    }
    
    /// Load filter rules from a string
    /// - Parameter filterList: Filter rules in EasyList format
    /// - Returns: true if loading succeeded
    public func loadFilterList(_ filterList: String) -> Bool {
        guard let handle = engineHandle else { return false }
        
        return queue.sync(flags: .barrier) {
            return adblock_engine_load_filter_list(handle, filterList)
        }
    }
    
    /// Get current statistics
    /// - Returns: Statistics object with blocking metrics
    public func getStatistics() -> Statistics {
        guard let handle = engineHandle else {
            return Statistics(blockedCount: 0, allowedCount: 0, dataSaved: 0)
        }
        
        return queue.sync {
            var blocked: UInt64 = 0
            var allowed: UInt64 = 0
            var saved: UInt64 = 0
            
            adblock_engine_get_stats(handle, &blocked, &allowed, &saved)
            
            return Statistics(
                blockedCount: Int(blocked),
                allowedCount: Int(allowed),
                dataSaved: Int(saved)
            )
        }
    }
}