package com.adblock.analytics

import android.content.Context
import android.content.SharedPreferences
import android.util.Log
import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.util.*
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong

/**
 * Privacy-focused analytics for AdBlock
 * Only collects anonymous usage data to improve the app
 */
class Analytics(private val context: Context) {
    companion object {
        private const val TAG = "Analytics"
        private const val PREFS_NAME = "adblock_analytics"
        private const val KEY_ANONYMOUS_ID = "anonymous_id"
        private const val KEY_ENABLED = "analytics_enabled"
        private const val MAX_EVENTS = 1000
        
        @Volatile
        private var instance: Analytics? = null
        
        fun getInstance(context: Context): Analytics {
            return instance ?: synchronized(this) {
                instance ?: Analytics(context.applicationContext).also {
                    instance = it
                }
            }
        }
    }
    
    private val prefs: SharedPreferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    private val events = mutableListOf<AnalyticsEvent>()
    private val metrics = ConcurrentHashMap<String, MetricValue>()
    private val coroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val json = Json { ignoreUnknownKeys = true }
    
    private var sessionId = UUID.randomUUID().toString()
    private var sessionStartTime = System.currentTimeMillis()
    private var lastActivityTime = System.currentTimeMillis()
    
    val anonymousId: String = prefs.getString(KEY_ANONYMOUS_ID, null) ?: run {
        val id = UUID.randomUUID().toString()
        prefs.edit().putString(KEY_ANONYMOUS_ID, id).apply()
        id
    }
    
    var isEnabled: Boolean
        get() = prefs.getBoolean(KEY_ENABLED, true)
        set(value) {
            prefs.edit().putBoolean(KEY_ENABLED, value).apply()
            if (!value) {
                clearAllData()
            }
        }
    
    /**
     * Track an event
     */
    @JvmOverloads
    fun trackEvent(
        name: String,
        category: EventCategory,
        properties: Map<String, Any> = emptyMap()
    ) {
        if (!isEnabled) return
        
        synchronized(events) {
            if (events.size >= MAX_EVENTS) {
                events.removeAt(0)
            }
            
            events.add(AnalyticsEvent(
                name = name,
                category = category,
                properties = properties,
                timestamp = System.currentTimeMillis(),
                sessionId = sessionId
            ))
        }
        
        lastActivityTime = System.currentTimeMillis()
    }
    
    /**
     * Track a simple action
     */
    fun trackAction(action: String) {
        trackEvent(action, EventCategory.ACTION)
    }
    
    /**
     * Track feature usage
     */
    fun trackFeature(feature: String, properties: Map<String, Any> = emptyMap()) {
        trackEvent(feature, EventCategory.FEATURE, properties)
    }
    
    /**
     * Track performance metric
     */
    fun trackPerformance(metric: String, value: Double) {
        trackEvent(metric, EventCategory.PERFORMANCE, mapOf("value" to value))
        recordMetric(metric, value)
    }
    
    /**
     * Track error (non-crash)
     */
    fun trackError(error: String, errorType: String) {
        trackEvent(error, EventCategory.ERROR, mapOf("error_type" to errorType))
    }
    
    /**
     * Record a metric value
     */
    fun recordMetric(name: String, value: Double) {
        if (!isEnabled) return
        
        metrics.compute(name) { _, existing ->
            when (existing) {
                is MetricValue.Count -> MetricValue.Count(existing.value + 1)
                is MetricValue.Sum -> MetricValue.Sum(existing.value + value)
                is MetricValue.Average -> MetricValue.Average(
                    sum = existing.sum + value,
                    count = existing.count + 1
                )
                is MetricValue.Distribution -> {
                    val values = existing.values.toMutableList()
                    values.add(value)
                    if (values.size > 1000) {
                        values.subList(0, 500).clear()
                    }
                    MetricValue.Distribution(values)
                }
                null -> MetricValue.Sum(value)
            }
        }
    }
    
    /**
     * Increment a counter
     */
    fun incrementCounter(name: String) {
        metrics.compute(name) { _, existing ->
            when (existing) {
                is MetricValue.Count -> MetricValue.Count(existing.value + 1)
                else -> MetricValue.Count(1)
            }
        }
    }
    
    /**
     * Start a new session
     */
    fun startSession() {
        sessionId = UUID.randomUUID().toString()
        sessionStartTime = System.currentTimeMillis()
        lastActivityTime = System.currentTimeMillis()
        
        trackEvent("session_start", EventCategory.LIFECYCLE)
    }
    
    /**
     * End the current session
     */
    fun endSession() {
        val duration = (lastActivityTime - sessionStartTime) / 1000
        trackEvent("session_end", EventCategory.LIFECYCLE, mapOf("duration_seconds" to duration))
    }
    
    /**
     * Get analytics summary
     */
    fun getSummary(): AnalyticsSummary {
        val eventsByCategory = events.groupingBy { it.category }.eachCount()
        val metricSummaries = metrics.mapValues { (_, value) ->
            when (value) {
                is MetricValue.Count -> "Count: ${value.value}"
                is MetricValue.Sum -> "Sum: %.2f".format(value.value)
                is MetricValue.Average -> {
                    val avg = if (value.count > 0) value.sum / value.count else 0.0
                    "Avg: %.2f (n=${value.count})".format(avg)
                }
                is MetricValue.Distribution -> {
                    if (value.values.isEmpty()) {
                        "No data"
                    } else {
                        val min = value.values.minOrNull() ?: 0.0
                        val max = value.values.maxOrNull() ?: 0.0
                        val avg = value.values.average()
                        "Min: %.2f, Max: %.2f, Avg: %.2f".format(min, max, avg)
                    }
                }
            }
        }
        
        return AnalyticsSummary(
            totalEvents = events.size,
            eventsByCategory = eventsByCategory,
            metrics = metricSummaries,
            sessionDuration = (System.currentTimeMillis() - sessionStartTime) / 1000
        )
    }
    
    /**
     * Export events for analysis
     */
    fun exportEvents(limit: Int = 100): List<AnalyticsEvent> {
        return synchronized(events) {
            events.takeLast(limit).toList()
        }
    }
    
    /**
     * Clear all analytics data
     */
    fun clearAllData() {
        synchronized(events) {
            events.clear()
        }
        metrics.clear()
    }
    
    // Pre-defined event tracking methods
    
    fun trackAppLaunch(launchTimeMs: Long) {
        trackEvent("app_launch", EventCategory.LIFECYCLE, mapOf("launch_time_ms" to launchTimeMs))
    }
    
    fun trackVpnConnected(connectionTimeMs: Long) {
        trackEvent("vpn_connected", EventCategory.ACTION, mapOf("connection_time_ms" to connectionTimeMs))
        incrementCounter("vpn_connections")
    }
    
    fun trackVpnDisconnected(reason: String) {
        trackEvent("vpn_disconnected", EventCategory.ACTION, mapOf("reason" to reason))
    }
    
    fun trackFilterUpdated(rulesCount: Int, durationMs: Long) {
        trackEvent("filter_updated", EventCategory.ACTION, mapOf(
            "rules_count" to rulesCount,
            "duration_ms" to durationMs
        ))
    }
    
    fun trackCustomRuleAdded(ruleType: String) {
        trackEvent("custom_rule_added", EventCategory.FEATURE, mapOf("rule_type" to ruleType))
        incrementCounter("custom_rules")
    }
    
    fun trackAdBlocked(sizeBytes: Long) {
        // Don't track domains for privacy
        incrementCounter("ads_blocked")
        recordMetric("bytes_saved", sizeBytes.toDouble())
    }
    
    fun trackPerformanceWarning(metric: String, value: Double) {
        trackEvent("performance_warning", EventCategory.PERFORMANCE, mapOf(
            "metric" to metric,
            "value" to value
        ))
    }
}

/**
 * Analytics event
 */
@Serializable
data class AnalyticsEvent(
    val name: String,
    val category: EventCategory,
    val properties: Map<String, Any>,
    val timestamp: Long,
    val sessionId: String
)

/**
 * Event categories
 */
@Serializable
enum class EventCategory {
    LIFECYCLE,
    ACTION,
    PERFORMANCE,
    ERROR,
    FEATURE
}

/**
 * Metric value types
 */
sealed class MetricValue {
    data class Count(val value: Long) : MetricValue()
    data class Sum(val value: Double) : MetricValue()
    data class Average(val sum: Double, val count: Long) : MetricValue()
    data class Distribution(val values: List<Double>) : MetricValue()
}

/**
 * Analytics summary
 */
data class AnalyticsSummary(
    val totalEvents: Int,
    val eventsByCategory: Map<EventCategory, Int>,
    val metrics: Map<String, String>,
    val sessionDuration: Long
)