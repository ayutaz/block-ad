import XCTest
import NetworkExtension
@testable import AdBlock

/// Unit tests for Network Extension
/// TDD Red phase
class NetworkExtensionTests: XCTestCase {
    
    var provider: AdBlockPacketTunnelProvider!
    
    override func setUp() {
        super.setUp()
        provider = try! AdBlockPacketTunnelProvider()
    }
    
    override func tearDown() {
        provider = nil
        super.tearDown()
    }
    
    func testProviderInitialization() {
        // Given: A new provider instance
        // Then: Provider should be initialized
        XCTAssertNotNil(provider)
        XCTAssertNotNil(provider.engine)
    }
    
    func testPacketFiltering() {
        // Given: Provider with filter rules
        provider.loadFilterRules("||ads.com^")
        
        // When: Processing a packet to ads.com
        let adPacket = MockPacket(host: "ads.com", port: 443)
        let shouldBlockAd = provider.shouldBlockPacket(adPacket)
        
        // When: Processing a packet to safe site
        let safePacket = MockPacket(host: "example.com", port: 443)
        let shouldBlockSafe = provider.shouldBlockPacket(safePacket)
        
        // Then: Should block ad packet but not safe packet
        XCTAssertTrue(shouldBlockAd)
        XCTAssertFalse(shouldBlockSafe)
    }
    
    func testStatisticsTracking() {
        // Given: Provider with filter rules
        provider.loadFilterRules("||ads.com^")
        
        // When: Processing multiple packets
        provider.shouldBlockPacket(MockPacket(host: "ads.com", port: 443))
        provider.shouldBlockPacket(MockPacket(host: "safe.com", port: 443))
        provider.shouldBlockPacket(MockPacket(host: "ads.com", port: 80))
        
        // Then: Statistics should be accurate
        let stats = provider.getStatistics()
        XCTAssertEqual(stats.blockedCount, 2)
        XCTAssertEqual(stats.allowedCount, 1)
    }
    
    func testTunnelConfiguration() {
        // Given: Provider configuration
        let config = provider.createTunnelConfiguration()
        
        // Then: Configuration should be valid
        XCTAssertNotNil(config.serverAddress)
        XCTAssertFalse(config.dnsSettings?.servers.isEmpty ?? true)
        XCTAssertNotNil(config.ipv4Settings)
    }
}

/// Mock packet for testing
struct MockPacket {
    let host: String
    let port: Int
}

/// Statistics struct
struct Statistics {
    let blockedCount: Int
    let allowedCount: Int
    let dataSaved: Int
    
    var blockRate: Double {
        let total = blockedCount + allowedCount
        return total > 0 ? Double(blockedCount) / Double(total) : 0.0
    }
}