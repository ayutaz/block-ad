package com.adblock

/**
 * Mock implementation of AdBlockEngine for unit tests
 * This avoids the need to load native libraries in unit tests
 */
class MockAdBlockEngine : AdBlockEngine {
    private var blockingEnabled = true
    private val blockedUrls = mutableSetOf<String>()
    private val allowedUrls = mutableSetOf<String>()
    
    override fun shouldBlock(url: String): Boolean {
        return if (blockingEnabled) {
            url.contains("doubleclick") || 
            url.contains("googleadservices") ||
            url.contains("googlesyndication") ||
            url.contains("google-analytics") ||
            url.contains("facebook.com/tr") ||
            url.contains("amazon-adsystem")
        } else {
            false
        }
    }
    
    override fun loadFilterList(filterList: String): Boolean {
        // Simulate successful loading
        return true
    }
    
    override fun getStatistics(): Statistics {
        return Statistics(
            blockedCount = blockedUrls.size,
            allowedCount = allowedUrls.size,
            dataSaved = blockedUrls.size * 1024 // Simulate 1KB per blocked request
        )
    }
    
    override fun resetStatistics(): Boolean {
        blockedUrls.clear()
        allowedUrls.clear()
        return true
    }
    
    override fun destroy() {
        // No-op for mock
    }
    
    fun setBlockingEnabled(enabled: Boolean) {
        blockingEnabled = enabled
    }
    
    fun addBlockedUrl(url: String) {
        blockedUrls.add(url)
    }
    
    fun addAllowedUrl(url: String) {
        allowedUrls.add(url)
    }
}