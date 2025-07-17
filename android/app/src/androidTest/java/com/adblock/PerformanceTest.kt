package com.adblock

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.adblock.debug.MemoryMonitor
import com.adblock.core.AdBlockEngine
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.system.measureTimeMillis

/**
 * Performance tests to verify optimization targets
 */
@RunWith(AndroidJUnit4::class)
class PerformanceTest {
    
    private lateinit var context: android.content.Context
    private lateinit var memoryMonitor: MemoryMonitor
    private lateinit var engine: AdBlockEngine
    
    @Before
    fun setup() {
        context = InstrumentationRegistry.getInstrumentation().targetContext
        memoryMonitor = MemoryMonitor.getInstance(context)
        engine = AdBlockEngine.getInstance(context)
    }
    
    @Test
    fun testMemoryUsageUnder30MB() {
        // Initialize engine with default filters
        engine.initialize()
        
        // Wait for initialization
        Thread.sleep(2000)
        
        // Force GC to get accurate measurement
        memoryMonitor.forceGCAndLog()
        
        // Measure memory
        val memoryInfo = memoryMonitor.getMemoryInfo()
        
        println("Memory usage: ${memoryInfo.appMemoryMB}MB")
        println(memoryMonitor.getDetailedReport())
        
        // Verify memory usage is under 30MB
        assertTrue(
            "Memory usage (${memoryInfo.appMemoryMB}MB) exceeds 30MB target",
            memoryInfo.appMemoryMB <= 30f
        )
    }
    
    @Test
    fun testFilterLoadingPerformance() {
        val loadTime = measureTimeMillis {
            engine.initialize()
        }
        
        println("Filter loading time: ${loadTime}ms")
        
        // Should load within 2 seconds
        assertTrue(
            "Filter loading time (${loadTime}ms) exceeds 2000ms target",
            loadTime < 2000
        )
    }
    
    @Test
    fun testUrlCheckingPerformance() {
        engine.initialize()
        Thread.sleep(1000) // Wait for initialization
        
        val testUrls = listOf(
            "https://doubleclick.net/ads/banner.jpg",
            "https://google-analytics.com/track.js",
            "https://facebook.com/tr",
            "https://example.com/content.html",
            "https://youtube.com/watch?v=123",
            "https://ads.twitter.com/pixel",
            "https://amazon-adsystem.com/banner",
            "https://googlesyndication.com/pagead/js",
            "https://secure.website.com/page",
            "https://cdn.content.com/image.jpg"
        )
        
        // Warm up
        testUrls.forEach { engine.shouldBlockUrl(it) }
        
        // Measure
        val times = mutableListOf<Long>()
        repeat(100) {
            testUrls.forEach { url ->
                val time = measureTimeMillis {
                    engine.shouldBlockUrl(url)
                }
                times.add(time)
            }
        }
        
        val avgTime = times.average()
        val maxTime = times.maxOrNull() ?: 0
        
        println("URL checking performance:")
        println("  Average: ${avgTime}ms")
        println("  Max: ${maxTime}ms")
        println("  Total checks: ${times.size}")
        
        // Average should be under 1ms
        assertTrue(
            "Average URL check time (${avgTime}ms) exceeds 1ms target",
            avgTime < 1.0
        )
        
        // Max should be under 5ms
        assertTrue(
            "Max URL check time (${maxTime}ms) exceeds 5ms limit",
            maxTime < 5
        )
    }
    
    @Test
    fun testMemoryUnderLoad() {
        engine.initialize()
        Thread.sleep(1000)
        
        val initialMemory = memoryMonitor.getMemoryInfo().appMemoryMB
        println("Initial memory: ${initialMemory}MB")
        
        // Simulate heavy load
        val domains = List(10000) { "domain$it.com" }
        val urls = domains.flatMap { domain ->
            listOf(
                "https://$domain/ads/banner.jpg",
                "https://$domain/track.js",
                "https://$domain/pixel.gif"
            )
        }
        
        // Process URLs
        urls.forEach { url ->
            engine.shouldBlockUrl(url)
        }
        
        // Check memory after load
        val loadedMemory = memoryMonitor.getMemoryInfo().appMemoryMB
        println("Memory under load: ${loadedMemory}MB")
        
        // Memory should still be under 35MB even under load
        assertTrue(
            "Memory under load (${loadedMemory}MB) exceeds 35MB limit",
            loadedMemory <= 35f
        )
        
        // Clean up and check memory recovery
        memoryMonitor.forceGCAndLog()
        val recoveredMemory = memoryMonitor.getMemoryInfo().appMemoryMB
        println("Memory after GC: ${recoveredMemory}MB")
        
        // Should recover to near initial levels
        assertTrue(
            "Memory did not recover after GC",
            recoveredMemory <= initialMemory + 5f
        )
    }
    
    @Test
    fun testCustomRulesPerformance() {
        engine.initialize()
        Thread.sleep(1000)
        
        // Add custom rules
        val customRules = List(1000) { "||custom$it.com^" }
        
        val addTime = measureTimeMillis {
            customRules.forEach { rule ->
                engine.addCustomRule(rule)
            }
        }
        
        println("Time to add 1000 custom rules: ${addTime}ms")
        
        // Should complete within reasonable time
        assertTrue(
            "Adding custom rules took too long (${addTime}ms)",
            addTime < 5000
        )
        
        // Test performance with custom rules
        val testUrl = "https://custom500.com/ads.js"
        val checkTime = measureTimeMillis {
            repeat(1000) {
                engine.shouldBlockUrl(testUrl)
            }
        }
        
        val avgCheckTime = checkTime / 1000.0
        println("Average check time with custom rules: ${avgCheckTime}ms")
        
        // Should still be fast
        assertTrue(
            "URL checking with custom rules too slow (${avgCheckTime}ms)",
            avgCheckTime < 2.0
        )
    }
    
    @Test
    fun testConcurrentAccess() {
        engine.initialize()
        Thread.sleep(1000)
        
        val threads = 10
        val checksPerThread = 1000
        val errors = mutableListOf<Exception>()
        
        val time = measureTimeMillis {
            val threadList = List(threads) { threadIndex ->
                Thread {
                    try {
                        repeat(checksPerThread) { i ->
                            val url = "https://site$threadIndex-$i.com/ads.js"
                            engine.shouldBlockUrl(url)
                        }
                    } catch (e: Exception) {
                        errors.add(e)
                    }
                }
            }
            
            threadList.forEach { it.start() }
            threadList.forEach { it.join() }
        }
        
        println("Concurrent access test:")
        println("  Threads: $threads")
        println("  Total checks: ${threads * checksPerThread}")
        println("  Time: ${time}ms")
        println("  Errors: ${errors.size}")
        
        // Should complete without errors
        assertTrue("Concurrent access caused errors", errors.isEmpty())
        
        // Should complete in reasonable time
        assertTrue(
            "Concurrent access too slow (${time}ms)",
            time < 5000
        )
    }
}