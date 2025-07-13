import Foundation

/// Performance metrics for the ad blocking engine
public struct PerformanceMetrics: Codable {
    let totalRequests: Int64
    let blockedRequests: Int64
    let allowedRequests: Int64
    let avgProcessingTimeNs: Int64
    let maxProcessingTimeNs: Int64
    let minProcessingTimeNs: Int64
    let filterCount: Int
    let memoryUsageBytes: Int
    let parseErrors: Int64
    let matchErrors: Int64
    let cacheHits: Int64
    let cacheMisses: Int64
    let cacheSize: Int
    let blockRate: Double
    let cacheHitRate: Double
    
    private enum CodingKeys: String, CodingKey {
        case totalRequests = "total_requests"
        case blockedRequests = "blocked_requests"
        case allowedRequests = "allowed_requests"
        case avgProcessingTimeNs = "avg_processing_time_ns"
        case maxProcessingTimeNs = "max_processing_time_ns"
        case minProcessingTimeNs = "min_processing_time_ns"
        case filterCount = "filter_count"
        case memoryUsageBytes = "memory_usage_bytes"
        case parseErrors = "parse_errors"
        case matchErrors = "match_errors"
        case cacheHits = "cache_hits"
        case cacheMisses = "cache_misses"
        case cacheSize = "cache_size"
        case blockRate = "block_rate"
        case cacheHitRate = "cache_hit_rate"
    }
    
    /// Get average processing time in microseconds
    var avgProcessingTimeMicros: Double {
        Double(avgProcessingTimeNs) / 1000.0
    }
    
    /// Get max processing time in microseconds
    var maxProcessingTimeMicros: Double {
        Double(maxProcessingTimeNs) / 1000.0
    }
    
    /// Get min processing time in microseconds
    var minProcessingTimeMicros: Double {
        Double(minProcessingTimeNs) / 1000.0
    }
    
    /// Get memory usage in MB
    var memoryUsageMB: Double {
        Double(memoryUsageBytes) / (1024.0 * 1024.0)
    }
    
    /// Format block rate as percentage string
    var formattedBlockRate: String {
        String(format: "%.1f%%", blockRate)
    }
    
    /// Format cache hit rate as percentage string
    var formattedCacheHitRate: String {
        String(format: "%.1f%%", cacheHitRate)
    }
    
    /// Create from JSON string
    static func fromJSON(_ json: String) -> PerformanceMetrics? {
        guard let data = json.data(using: .utf8) else { return nil }
        
        let decoder = JSONDecoder()
        return try? decoder.decode(PerformanceMetrics.self, from: data)
    }
}