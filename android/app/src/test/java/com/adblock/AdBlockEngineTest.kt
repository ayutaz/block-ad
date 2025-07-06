package com.adblock

import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import java.io.File
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

/**
 * Unit tests for AdBlockEngine JNI wrapper
 */
class AdBlockEngineTest {
    
    private lateinit var engine: AdBlockEngine
    
    @Before
    fun setUp() {
        // Load native library for tests
        System.loadLibrary("adblock_core")
        engine = AdBlockEngine()
    }
    
    @After
    fun tearDown() {
        engine.destroy()
    }
    
    @Test
    fun `should create and destroy engine`() {
        // Given: A new engine instance
        val newEngine = AdBlockEngine()
        
        // Then: Engine should be initialized
        assertTrue(newEngine.isInitialized())
        
        // When: Destroying the engine
        newEngine.destroy()
        
        // Then: Engine should no longer be initialized
        assertFalse(newEngine.isInitialized())
    }
    
    @Test
    fun `should block URLs based on filter rules`() {
        // Given: Engine with filter rules
        engine.loadFilterList("||doubleclick.net^")
        
        // When: Checking URLs
        val shouldBlockAd = engine.shouldBlock("https://doubleclick.net/ads")
        val shouldBlockSafe = engine.shouldBlock("https://example.com")
        
        // Then: Should block ad URL but not safe URL
        assertTrue(shouldBlockAd)
        assertFalse(shouldBlockSafe)
    }
    
    @Test
    fun `should load filter list from string`() {
        // Given: A filter list
        val filterList = """
            ||ads.example.com^
            ||tracker.com^
            */banner/*
        """.trimIndent()
        
        // When: Loading the filter list
        val result = engine.loadFilterList(filterList)
        
        // Then: Loading should succeed
        assertTrue(result)
        
        // And: Rules should be active
        assertTrue(engine.shouldBlock("https://ads.example.com/img"))
        assertTrue(engine.shouldBlock("https://site.com/banner/ad.jpg"))
    }
    
    @Test
    fun `should get statistics`() {
        // Given: Engine with some activity
        engine.loadFilterList("||ads.com^")
        engine.shouldBlock("https://ads.com/banner")
        engine.shouldBlock("https://safe.com")
        
        // When: Getting statistics
        val stats = engine.getStatistics()
        
        // Then: Statistics should be accurate
        assertEquals(1, stats.blockedCount)
        assertEquals(1, stats.allowedCount)
        assertTrue(stats.blockRate > 0)
    }
    
    @Test
    fun `should handle concurrent access safely`() {
        // Given: Engine with rules
        engine.loadFilterList("||ads.com^")
        
        // When: Multiple threads access the engine
        val threadCount = 10
        val latch = CountDownLatch(threadCount)
        val results = mutableListOf<Boolean>()
        
        repeat(threadCount) { i ->
            Thread {
                val result = engine.shouldBlock("https://ads.com/thread$i")
                synchronized(results) {
                    results.add(result)
                }
                latch.countDown()
            }.start()
        }
        
        // Wait for all threads to complete
        assertTrue("Threads did not complete in time", latch.await(5, TimeUnit.SECONDS))
        
        // Then: All results should be consistent
        assertEquals(threadCount, results.size)
        assertTrue("All URLs should be blocked", results.all { it })
    }
    
    @Test
    fun `should update filter lists`() {
        // Given: Initial filter list
        engine.loadFilterList("||oldads.com^")
        assertTrue(engine.shouldBlock("https://oldads.com"))
        
        // When: Updating with new filter list
        engine.loadFilterList("||newads.com^")
        
        // Then: New rules should be active
        assertTrue(engine.shouldBlock("https://newads.com"))
    }
}