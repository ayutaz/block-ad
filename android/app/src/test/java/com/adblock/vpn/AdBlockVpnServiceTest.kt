package com.adblock.vpn

import android.content.Intent
import android.net.VpnService
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.mockito.Mock
import org.mockito.junit.MockitoJUnitRunner
import org.mockito.kotlin.*
import com.adblock.AdBlockEngine
import com.adblock.Statistics

/**
 * Unit tests for AdBlockVpnService
 */
@RunWith(MockitoJUnitRunner::class)
class AdBlockVpnServiceTest {
    
    @Mock
    private lateinit var mockIntent: Intent
    
    private lateinit var service: AdBlockVpnService
    
    @Before
    fun setUp() {
        service = AdBlockVpnService()
    }
    
    @Test
    fun `should start VPN service`() {
        // Given: A VPN service instance
        // When: Starting the service
        val result = service.onStartCommand(mockIntent, 0, 1)
        
        // Then: Service should start successfully
        assertEquals(VpnService.START_STICKY, result)
        assertTrue(service.isRunning())
    }
    
    @Test
    fun `should stop VPN service`() {
        // Given: A running VPN service
        service.onStartCommand(mockIntent, 0, 1)
        
        // When: Stopping the service
        service.stop()
        
        // Then: Service should stop
        assertFalse(service.isRunning())
    }
    
    @Test
    fun `should intercept and filter network packets`() {
        // Given: A running VPN service with filter rules
        service.setFilterRules("||ads.com^")
        service.onStartCommand(mockIntent, 0, 1)
        
        // When: Processing a packet to ads.com
        val packet = createMockPacket("ads.com", 443)
        val shouldBlock = service.shouldBlockPacket(packet)
        
        // Then: Packet should be blocked
        assertTrue(shouldBlock)
    }
    
    @Test
    fun `should allow non-blocked packets`() {
        // Given: A running VPN service with filter rules
        service.setFilterRules("||ads.com^")
        service.onStartCommand(mockIntent, 0, 1)
        
        // When: Processing a packet to safe site
        val packet = createMockPacket("example.com", 443)
        val shouldBlock = service.shouldBlockPacket(packet)
        
        // Then: Packet should not be blocked
        assertFalse(shouldBlock)
    }
    
    @Test
    fun `should track statistics`() {
        // Given: A running VPN service
        service.setFilterRules("||ads.com^")
        service.onStartCommand(mockIntent, 0, 1)
        
        // When: Processing multiple packets
        service.shouldBlockPacket(createMockPacket("ads.com", 443))
        service.shouldBlockPacket(createMockPacket("safe.com", 443))
        service.shouldBlockPacket(createMockPacket("ads.com", 80))
        
        // Then: Statistics should be accurate
        val stats = service.getStatistics()
        assertEquals(2, stats.blockedCount)
        assertEquals(1, stats.allowedCount)
    }
    
    private fun createMockPacket(host: String, port: Int): NetworkPacket {
        return NetworkPacket(host, port, 1024)
    }
}