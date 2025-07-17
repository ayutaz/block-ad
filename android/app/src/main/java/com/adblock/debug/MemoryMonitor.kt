package com.adblock.debug

import android.app.ActivityManager
import android.content.Context
import android.os.Debug
import android.os.Handler
import android.os.Looper
import android.util.Log
import com.adblock.AdBlockApplication
import java.text.DecimalFormat
import kotlin.concurrent.timer

/**
 * Memory usage monitor for debugging and optimization
 */
class MemoryMonitor(private val context: Context) {
    
    companion object {
        private const val TAG = "MemoryMonitor"
        private const val MONITORING_INTERVAL_MS = 5000L // 5 seconds
        private const val MB = 1024 * 1024
        
        @Volatile
        private var instance: MemoryMonitor? = null
        
        fun getInstance(context: Context): MemoryMonitor {
            return instance ?: synchronized(this) {
                instance ?: MemoryMonitor(context.applicationContext).also {
                    instance = it
                }
            }
        }
    }
    
    private var isMonitoring = false
    private val handler = Handler(Looper.getMainLooper())
    private val decimalFormat = DecimalFormat("#.##")
    
    data class MemoryInfo(
        val totalMemoryMB: Float,
        val availableMemoryMB: Float,
        val usedMemoryMB: Float,
        val appMemoryMB: Float,
        val nativeHeapMB: Float,
        val javaHeapMB: Float,
        val percentUsed: Float
    )
    
    /**
     * Start monitoring memory usage
     */
    fun startMonitoring(callback: (MemoryInfo) -> Unit) {
        if (isMonitoring) return
        
        isMonitoring = true
        Log.d(TAG, "Starting memory monitoring")
        
        timer(period = MONITORING_INTERVAL_MS) {
            if (!isMonitoring) {
                cancel()
                return@timer
            }
            
            val memoryInfo = getMemoryInfo()
            handler.post {
                callback(memoryInfo)
                
                // Log if memory usage is high
                if (memoryInfo.appMemoryMB > 30) {
                    Log.w(TAG, "High memory usage detected: ${memoryInfo.appMemoryMB}MB")
                    
                    // Report to analytics
                    AdBlockApplication.instance.analytics.trackPerformanceWarning(
                        "high_memory_usage",
                        memoryInfo.appMemoryMB.toDouble()
                    )
                }
            }
        }
    }
    
    /**
     * Stop monitoring memory usage
     */
    fun stopMonitoring() {
        isMonitoring = false
        Log.d(TAG, "Stopped memory monitoring")
    }
    
    /**
     * Get current memory information
     */
    fun getMemoryInfo(): MemoryInfo {
        val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val memoryInfo = ActivityManager.MemoryInfo()
        activityManager.getMemoryInfo(memoryInfo)
        
        // System memory
        val totalMemory = memoryInfo.totalMem / MB.toFloat()
        val availableMemory = memoryInfo.availMem / MB.toFloat()
        val usedMemory = totalMemory - availableMemory
        
        // App memory
        val runtime = Runtime.getRuntime()
        val javaHeap = (runtime.totalMemory() - runtime.freeMemory()) / MB.toFloat()
        val javaHeapMax = runtime.maxMemory() / MB.toFloat()
        
        // Native heap
        val nativeHeap = Debug.getNativeHeapAllocatedSize() / MB.toFloat()
        
        // Total app memory
        val appMemory = javaHeap + nativeHeap
        
        // Percentage
        val percentUsed = (usedMemory / totalMemory) * 100
        
        return MemoryInfo(
            totalMemoryMB = roundToTwoDecimals(totalMemory),
            availableMemoryMB = roundToTwoDecimals(availableMemory),
            usedMemoryMB = roundToTwoDecimals(usedMemory),
            appMemoryMB = roundToTwoDecimals(appMemory),
            nativeHeapMB = roundToTwoDecimals(nativeHeap),
            javaHeapMB = roundToTwoDecimals(javaHeap),
            percentUsed = roundToTwoDecimals(percentUsed)
        )
    }
    
    /**
     * Force garbage collection and log memory stats
     */
    fun forceGCAndLog() {
        val before = getMemoryInfo()
        
        // Force garbage collection
        System.gc()
        System.runFinalization()
        System.gc()
        
        // Wait a bit for GC to complete
        Thread.sleep(100)
        
        val after = getMemoryInfo()
        
        Log.d(TAG, """
            Memory cleanup results:
            Before GC: ${before.appMemoryMB}MB (Java: ${before.javaHeapMB}MB, Native: ${before.nativeHeapMB}MB)
            After GC:  ${after.appMemoryMB}MB (Java: ${after.javaHeapMB}MB, Native: ${after.nativeHeapMB}MB)
            Freed: ${before.appMemoryMB - after.appMemoryMB}MB
        """.trimIndent())
    }
    
    /**
     * Check if app is under memory pressure
     */
    fun isUnderMemoryPressure(): Boolean {
        val memoryInfo = getMemoryInfo()
        return memoryInfo.appMemoryMB > 35 || memoryInfo.percentUsed > 90
    }
    
    /**
     * Get detailed memory report
     */
    fun getDetailedReport(): String {
        val info = getMemoryInfo()
        val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        
        return buildString {
            appendLine("=== Memory Report ===")
            appendLine("System Memory:")
            appendLine("  Total: ${info.totalMemoryMB}MB")
            appendLine("  Available: ${info.availableMemoryMB}MB")
            appendLine("  Used: ${info.usedMemoryMB}MB (${info.percentUsed}%)")
            appendLine()
            appendLine("App Memory:")
            appendLine("  Total: ${info.appMemoryMB}MB")
            appendLine("  Java Heap: ${info.javaHeapMB}MB")
            appendLine("  Native Heap: ${info.nativeHeapMB}MB")
            appendLine()
            appendLine("Memory Class:")
            appendLine("  Normal: ${activityManager.memoryClass}MB")
            appendLine("  Large: ${activityManager.largeMemoryClass}MB")
            appendLine()
            appendLine("Low Memory: ${isLowMemory()}")
            appendLine("==================")
        }
    }
    
    /**
     * Check if system is in low memory state
     */
    fun isLowMemory(): Boolean {
        val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val memoryInfo = ActivityManager.MemoryInfo()
        activityManager.getMemoryInfo(memoryInfo)
        return memoryInfo.lowMemory
    }
    
    private fun roundToTwoDecimals(value: Float): Float {
        return decimalFormat.format(value).toFloat()
    }
}

/**
 * Extension function for easy memory logging
 */
fun logMemoryUsage(tag: String = "MemoryUsage") {
    val monitor = MemoryMonitor.getInstance(AdBlockApplication.instance)
    val info = monitor.getMemoryInfo()
    Log.d(tag, "App Memory: ${info.appMemoryMB}MB (Java: ${info.javaHeapMB}MB, Native: ${info.nativeHeapMB}MB)")
}