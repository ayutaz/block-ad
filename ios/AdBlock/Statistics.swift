import Foundation

/// Statistics for ad blocking activity
public struct Statistics: Codable {
    /// Number of blocked requests
    public let blockedCount: Int
    
    /// Number of allowed requests
    public let allowedCount: Int
    
    /// Estimated data saved in bytes
    public let dataSaved: Int
    
    /// Initialize statistics
    public init(blockedCount: Int, allowedCount: Int, dataSaved: Int) {
        self.blockedCount = blockedCount
        self.allowedCount = allowedCount
        self.dataSaved = dataSaved
    }
    
    /// Total number of requests processed
    public var totalRequests: Int {
        return blockedCount + allowedCount
    }
    
    /// Block rate as a percentage (0-100)
    public var blockRate: Double {
        guard totalRequests > 0 else { return 0 }
        return Double(blockedCount) / Double(totalRequests) * 100
    }
    
    /// Human-readable data saved
    public var dataSavedFormatted: String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .binary
        return formatter.string(fromByteCount: Int64(dataSaved))
    }
}