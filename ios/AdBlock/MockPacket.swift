import Foundation

/// Mock packet structure for testing packet filtering
public struct MockPacket {
    /// Destination host
    public let host: String
    
    /// Destination port
    public let port: Int
    
    /// Initialize a mock packet
    public init(host: String, port: Int) {
        self.host = host
        self.port = port
    }
}