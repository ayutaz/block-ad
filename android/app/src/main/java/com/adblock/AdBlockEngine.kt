package com.adblock

import androidx.annotation.Keep
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.read
import kotlin.concurrent.write

/**
 * JNI wrapper for the Rust AdBlock engine
 * Provides thread-safe access to the native engine
 */
@Keep
class AdBlockEngine {
    // Native engine handle
    private var engineHandle: Long = 0
    private val lock = ReentrantReadWriteLock()
    
    init {
        engineHandle = nativeCreate()
        if (engineHandle == 0L) {
            throw RuntimeException("Failed to create native AdBlock engine")
        }
    }
    
    /**
     * Check if the engine is initialized
     */
    fun isInitialized(): Boolean = lock.read {
        engineHandle != 0L
    }
    
    /**
     * Check if a URL should be blocked
     */
    fun shouldBlock(url: String): Boolean = lock.read {
        if (engineHandle == 0L) return false
        nativeShouldBlock(engineHandle, url)
    }
    
    /**
     * Load a filter list
     */
    fun loadFilterList(filterList: String): Boolean = lock.write {
        if (engineHandle == 0L) return false
        nativeLoadFilterList(engineHandle, filterList)
    }
    
    /**
     * Load filters (alias for loadFilterList)
     */
    fun loadFilters(filters: String): Boolean = loadFilterList(filters)
    
    /**
     * Get statistics
     */
    fun getStatistics(): Statistics = lock.read {
        if (engineHandle == 0L) {
            return Statistics(0, 0, 0)
        }
        
        val json = nativeGetStats(engineHandle) ?: return Statistics(0, 0, 0)
        
        // Parse simple JSON manually to avoid dependencies
        val blockedCount = extractJsonLong(json, "blocked_count") ?: 0
        val allowedCount = extractJsonLong(json, "allowed_count") ?: 0
        val dataSaved = extractJsonLong(json, "data_saved") ?: 0
        
        return Statistics(blockedCount.toInt(), allowedCount.toInt(), dataSaved.toInt())
    }
    
    /**
     * Reset statistics
     */
    fun resetStatistics(): Boolean = lock.write {
        if (engineHandle == 0L) return false
        nativeResetStats(engineHandle)
    }
    
    /**
     * Destroy the engine and free native resources
     */
    fun destroy() = lock.write {
        if (engineHandle != 0L) {
            nativeDestroy(engineHandle)
            engineHandle = 0
        }
    }
    
    @Throws(Throwable::class)
    protected fun finalize() {
        destroy()
    }
    
    /**
     * Simple JSON parser for numeric values
     */
    private fun extractJsonLong(json: String, key: String): Long? {
        val pattern = """"$key"\s*:\s*(\d+)""".toRegex()
        return pattern.find(json)?.groupValues?.get(1)?.toLongOrNull()
    }
    
    /**
     * Get performance metrics
     */
    fun getPerformanceMetrics(): String? = lock.read {
        if (engineHandle == 0L) return null
        nativeGetMetrics(engineHandle)
    }
    
    companion object {
        private const val LIBRARY_NAME = "adblock_core"
        
        @Volatile
        private var INSTANCE: AdBlockEngine? = null
        
        init {
            try {
                System.loadLibrary(LIBRARY_NAME)
            } catch (e: UnsatisfiedLinkError) {
                throw RuntimeException("Failed to load native library: $LIBRARY_NAME", e)
            }
        }
        
        /**
         * Get singleton instance
         */
        fun getInstance(): AdBlockEngine {
            return INSTANCE ?: synchronized(this) {
                INSTANCE ?: AdBlockEngine().also { INSTANCE = it }
            }
        }
    }
    
    // Native methods
    @Keep
    private external fun nativeCreate(): Long
    
    @Keep
    private external fun nativeDestroy(handle: Long)
    
    @Keep
    private external fun nativeShouldBlock(handle: Long, url: String): Boolean
    
    @Keep
    private external fun nativeLoadFilterList(handle: Long, filterList: String): Boolean
    
    @Keep
    private external fun nativeGetStats(handle: Long): String?
    
    @Keep
    private external fun nativeResetStats(handle: Long): Boolean
    
    @Keep
    private external fun nativeGetMetrics(handle: Long): String?
}

