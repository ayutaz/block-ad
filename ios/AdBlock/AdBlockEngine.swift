import Foundation

/// Swift wrapper for the Rust ad blocking engine
/// Provides thread-safe access to the underlying Rust engine
public final class AdBlockEngine {
    private let engineHandle: UnsafeMutableRawPointer
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
            guard let statsPtr = adblock_engine_get_stats(engineHandle) else {
                return Statistics(blockedCount: 0, allowedCount: 0, dataSaved: 0)
            }
            
            defer {
                adblock_free_string(statsPtr)
            }
            
            let statsString = String(cString: statsPtr)
            
            guard let data = statsString.data(using: .utf8),
                  let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let blockedCount = json["blocked_count"] as? Int,
                  let allowedCount = json["allowed_count"] as? Int,
                  let dataSaved = json["data_saved"] as? Int else {
                return Statistics(blockedCount: 0, allowedCount: 0, dataSaved: 0)
            }
            
            return Statistics(
                blockedCount: blockedCount,
                allowedCount: allowedCount,
                dataSaved: dataSaved
            )
        }
    }
    
    /// Get performance metrics
    /// - Returns: PerformanceMetrics object with detailed performance data
    public func getPerformanceMetrics() -> PerformanceMetrics? {
        return queue.sync {
            guard let metricsPtr = adblock_engine_get_metrics(engineHandle) else {
                return nil
            }
            
            defer {
                adblock_free_string(metricsPtr)
            }
            
            let metricsString = String(cString: metricsPtr)
            return PerformanceMetrics.fromJSON(metricsString)
        }
    }
    
    /// Reset statistics
    public func resetStatistics() {
        queue.sync(flags: .barrier) {
            adblock_engine_reset_stats(engineHandle)
        }
    }
}

/// Errors that can occur during engine operations
public enum EngineError: Error {
    case initializationFailed
}