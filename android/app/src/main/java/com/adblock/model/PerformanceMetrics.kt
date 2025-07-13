package com.adblock.model

import com.google.gson.Gson
import com.google.gson.annotations.SerializedName

/**
 * Performance metrics data class
 */
data class PerformanceMetrics(
    @SerializedName("total_requests")
    val totalRequests: Long = 0,
    
    @SerializedName("blocked_requests")
    val blockedRequests: Long = 0,
    
    @SerializedName("allowed_requests")
    val allowedRequests: Long = 0,
    
    @SerializedName("avg_processing_time_ns")
    val avgProcessingTimeNs: Long = 0,
    
    @SerializedName("max_processing_time_ns")
    val maxProcessingTimeNs: Long = 0,
    
    @SerializedName("min_processing_time_ns")
    val minProcessingTimeNs: Long = 0,
    
    @SerializedName("filter_count")
    val filterCount: Int = 0,
    
    @SerializedName("memory_usage_bytes")
    val memoryUsageBytes: Int = 0,
    
    @SerializedName("parse_errors")
    val parseErrors: Long = 0,
    
    @SerializedName("match_errors")
    val matchErrors: Long = 0,
    
    @SerializedName("cache_hits")
    val cacheHits: Long = 0,
    
    @SerializedName("cache_misses")
    val cacheMisses: Long = 0,
    
    @SerializedName("cache_size")
    val cacheSize: Int = 0,
    
    @SerializedName("block_rate")
    val blockRate: Double = 0.0,
    
    @SerializedName("cache_hit_rate")
    val cacheHitRate: Double = 0.0
) {
    
    /**
     * Get average processing time in microseconds
     */
    fun getAvgProcessingTimeMicros(): Double {
        return avgProcessingTimeNs / 1000.0
    }
    
    /**
     * Get max processing time in microseconds
     */
    fun getMaxProcessingTimeMicros(): Double {
        return maxProcessingTimeNs / 1000.0
    }
    
    /**
     * Get min processing time in microseconds
     */
    fun getMinProcessingTimeMicros(): Double {
        return minProcessingTimeNs / 1000.0
    }
    
    /**
     * Get memory usage in MB
     */
    fun getMemoryUsageMB(): Double {
        return memoryUsageBytes / (1024.0 * 1024.0)
    }
    
    /**
     * Format block rate as percentage string
     */
    fun formatBlockRate(): String {
        return String.format("%.1f%%", blockRate)
    }
    
    /**
     * Format cache hit rate as percentage string
     */
    fun formatCacheHitRate(): String {
        return String.format("%.1f%%", cacheHitRate)
    }
    
    companion object {
        /**
         * Parse metrics from JSON string
         */
        fun fromJson(json: String): PerformanceMetrics? {
            return try {
                Gson().fromJson(json, PerformanceMetrics::class.java)
            } catch (e: Exception) {
                null
            }
        }
    }
}