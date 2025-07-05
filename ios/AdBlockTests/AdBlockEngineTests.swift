import XCTest
@testable import AdBlock

/// Unit tests for AdBlockEngine Swift wrapper
/// TDD Red phase - these tests will fail initially
class AdBlockEngineTests: XCTestCase {
    
    var engine: AdBlockEngine!
    
    override func setUp() {
        super.setUp()
        engine = AdBlockEngine()
    }
    
    override func tearDown() {
        engine = nil
        super.tearDown()
    }
    
    func testEngineCreationAndDestruction() {
        // Given: A new engine instance
        let newEngine = AdBlockEngine()
        
        // Then: Engine should be initialized
        XCTAssertTrue(newEngine.isInitialized)
        
        // When: Engine goes out of scope, it should be cleaned up
        // (handled by deinit)
    }
    
    func testURLBlocking() {
        // Given: Engine with filter rules
        XCTAssertTrue(engine.loadFilterList("||doubleclick.net^"))
        
        // When: Checking URLs
        let shouldBlockAd = engine.shouldBlock("https://doubleclick.net/ads")
        let shouldBlockSafe = engine.shouldBlock("https://example.com")
        
        // Then: Should block ad URL but not safe URL
        XCTAssertTrue(shouldBlockAd)
        XCTAssertFalse(shouldBlockSafe)
    }
    
    func testFilterListLoading() {
        // Given: A filter list
        let filterList = """
        ||ads.example.com^
        ||tracker.com^
        */banner/*
        """
        
        // When: Loading the filter list
        let result = engine.loadFilterList(filterList)
        
        // Then: Loading should succeed
        XCTAssertTrue(result)
        
        // And: Rules should be active
        XCTAssertTrue(engine.shouldBlock("https://ads.example.com/img"))
        XCTAssertTrue(engine.shouldBlock("https://site.com/banner/ad.jpg"))
    }
    
    func testStatistics() {
        // Given: Engine with some activity
        engine.loadFilterList("||ads.com^")
        _ = engine.shouldBlock("https://ads.com/banner")
        _ = engine.shouldBlock("https://safe.com")
        
        // When: Getting statistics
        let stats = engine.getStatistics()
        
        // Then: Statistics should be accurate
        XCTAssertEqual(stats.blockedCount, 1)
        XCTAssertEqual(stats.allowedCount, 1)
        XCTAssertGreaterThan(stats.blockRate, 0)
    }
    
    func testConcurrentAccess() {
        // Given: Engine with rules
        engine.loadFilterList("||ads.com^")
        
        // When: Multiple threads access the engine
        let expectation = self.expectation(description: "Concurrent access")
        expectation.expectedFulfillmentCount = 10
        
        let queue = DispatchQueue(label: "test", attributes: .concurrent)
        
        for i in 0..<10 {
            queue.async {
                let result = self.engine.shouldBlock("https://ads.com/thread\(i)")
                XCTAssertTrue(result)
                expectation.fulfill()
            }
        }
        
        // Then: All operations should complete successfully
        waitForExpectations(timeout: 5.0)
    }
    
    func testFilterListUpdate() {
        // Given: Initial filter list
        engine.loadFilterList("||oldads.com^")
        XCTAssertTrue(engine.shouldBlock("https://oldads.com"))
        
        // When: Updating with new filter list
        engine.loadFilterList("||newads.com^")
        
        // Then: New rules should be active
        XCTAssertTrue(engine.shouldBlock("https://newads.com"))
    }
}