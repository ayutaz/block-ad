package com.adblock.analytics

import android.content.Context
import android.os.Build
import android.util.Log
import kotlinx.coroutines.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.io.File
import java.util.*
import java.util.concurrent.ConcurrentLinkedQueue

/**
 * Privacy-respecting crash reporter for Android
 */
class CrashReporter(private val context: Context) {
    companion object {
        private const val TAG = "CrashReporter"
        private const val MAX_REPORTS = 100
        private const val CRASH_DIR = "crashes"
        
        @Volatile
        private var instance: CrashReporter? = null
        
        fun getInstance(context: Context): CrashReporter {
            return instance ?: synchronized(this) {
                instance ?: CrashReporter(context.applicationContext).also {
                    instance = it
                }
            }
        }
    }
    
    private val crashDir = File(context.filesDir, CRASH_DIR)
    private val recentCrashes = ConcurrentLinkedQueue<CrashReport>()
    private val coroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private val json = Json { 
        ignoreUnknownKeys = true
        prettyPrint = true
    }
    
    var isEnabled = true
        set(value) {
            field = value
            if (!value) {
                clearAllReports()
            }
        }
    
    init {
        crashDir.mkdirs()
        loadRecentCrashes()
        installUncaughtExceptionHandler()
    }
    
    /**
     * Report a crash
     */
    fun reportCrash(
        type: CrashType,
        message: String,
        throwable: Throwable? = null,
        context: CrashContext = CrashContext()
    ) {
        if (!isEnabled) return
        
        val report = CrashReport(
            id = UUID.randomUUID().toString(),
            timestamp = System.currentTimeMillis(),
            type = type,
            message = sanitizeMessage(message),
            stackTrace = throwable?.stackTraceToString(),
            appVersion = BuildConfig.VERSION_NAME,
            osVersion = "Android ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})",
            deviceModel = getAnonymizedDeviceModel(),
            context = context
        )
        
        // Add to memory queue
        recentCrashes.offer(report)
        while (recentCrashes.size > MAX_REPORTS) {
            recentCrashes.poll()
        }
        
        // Save to disk
        coroutineScope.launch {
            saveReport(report)
        }
        
        Log.e(TAG, "Crash reported: $type - $message", throwable)
    }
    
    /**
     * Report an exception
     */
    fun reportException(throwable: Throwable, context: CrashContext? = null) {
        reportCrash(
            CrashType.EXCEPTION,
            throwable.message ?: "Unknown exception",
            throwable,
            context ?: captureContext()
        )
    }
    
    /**
     * Report ANR (Application Not Responding)
     */
    fun reportANR(message: String) {
        reportCrash(
            CrashType.ANR,
            message,
            context = captureContext()
        )
    }
    
    /**
     * Report out of memory
     */
    fun reportOOM(availableMemoryMB: Int) {
        reportCrash(
            CrashType.OUT_OF_MEMORY,
            "Out of memory. Available: ${availableMemoryMB}MB",
            context = captureContext().copy(memoryUsageMB = availableMemoryMB)
        )
    }
    
    /**
     * Get recent crash reports
     */
    fun getRecentReports(limit: Int = 10): List<CrashReport> {
        return recentCrashes.toList()
            .sortedByDescending { it.timestamp }
            .take(limit)
    }
    
    /**
     * Get crash statistics
     */
    fun getStatistics(): CrashStatistics {
        val reports = recentCrashes.toList()
        val byType = reports.groupingBy { it.type }.eachCount()
        
        return CrashStatistics(
            totalCrashes = reports.size,
            crashesByType = byType,
            oldestCrash = reports.minByOrNull { it.timestamp }?.timestamp,
            newestCrash = reports.maxByOrNull { it.timestamp }?.timestamp
        )
    }
    
    /**
     * Clear all crash reports
     */
    fun clearAllReports() {
        recentCrashes.clear()
        coroutineScope.launch {
            crashDir.listFiles()?.forEach { it.delete() }
        }
    }
    
    /**
     * Capture current context
     */
    private fun captureContext(): CrashContext {
        val runtime = Runtime.getRuntime()
        val memoryInfo = android.app.ActivityManager.MemoryInfo()
        val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as android.app.ActivityManager
        activityManager.getMemoryInfo(memoryInfo)
        
        return CrashContext(
            filterRulesCount = null, // Would be set by caller
            memoryUsageMB = ((runtime.totalMemory() - runtime.freeMemory()) / 1024 / 1024).toInt(),
            availableMemoryMB = (memoryInfo.availMem / 1024 / 1024).toInt(),
            vpnActive = null, // Would be set by caller
            batteryLevel = getBatteryLevel(),
            screenOn = isScreenOn()
        )
    }
    
    /**
     * Install uncaught exception handler
     */
    private fun installUncaughtExceptionHandler() {
        val defaultHandler = Thread.getDefaultUncaughtExceptionHandler()
        
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            reportCrash(
                CrashType.NATIVE_CRASH,
                "Uncaught exception on thread: ${thread.name}",
                throwable,
                captureContext()
            )
            
            // Call default handler
            defaultHandler?.uncaughtException(thread, throwable)
        }
    }
    
    /**
     * Load recent crashes from disk
     */
    private fun loadRecentCrashes() {
        coroutineScope.launch {
            try {
                val files = crashDir.listFiles()
                    ?.filter { it.name.endsWith(".json") }
                    ?.sortedByDescending { it.lastModified() }
                    ?.take(MAX_REPORTS)
                    ?: return@launch
                
                files.forEach { file ->
                    try {
                        val content = file.readText()
                        val report = json.decodeFromString<CrashReport>(content)
                        recentCrashes.offer(report)
                    } catch (e: Exception) {
                        Log.e(TAG, "Failed to load crash report: ${file.name}", e)
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to load crash reports", e)
            }
        }
    }
    
    /**
     * Save report to disk
     */
    private suspend fun saveReport(report: CrashReport) {
        withContext(Dispatchers.IO) {
            try {
                val file = File(crashDir, "crash_${report.id}.json")
                file.writeText(json.encodeToString(report))
                
                // Clean up old files
                cleanupOldReports()
            } catch (e: Exception) {
                Log.e(TAG, "Failed to save crash report", e)
            }
        }
    }
    
    /**
     * Clean up old crash reports
     */
    private fun cleanupOldReports() {
        val files = crashDir.listFiles()
            ?.filter { it.name.endsWith(".json") }
            ?.sortedByDescending { it.lastModified() }
            ?: return
        
        // Keep only recent reports
        files.drop(MAX_REPORTS).forEach { it.delete() }
    }
    
    /**
     * Sanitize message to remove PII
     */
    private fun sanitizeMessage(message: String): String {
        return message
            .replace(Regex("\\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Z|a-z]{2,}\\b"), "[EMAIL]")
            .replace(Regex("\\b(?:[0-9]{1,3}\\.){3}[0-9]{1,3}\\b"), "[IP]")
            .replace(Regex("\\b\\d{3}[-.]?\\d{3}[-.]?\\d{4}\\b"), "[PHONE]")
            .take(1000)
    }
    
    /**
     * Get anonymized device model
     */
    private fun getAnonymizedDeviceModel(): String {
        return when {
            Build.MODEL.contains("Pixel", ignoreCase = true) -> "Google Pixel"
            Build.MODEL.contains("Galaxy", ignoreCase = true) -> "Samsung Galaxy"
            Build.MODEL.contains("OnePlus", ignoreCase = true) -> "OnePlus"
            Build.MODEL.contains("Xiaomi", ignoreCase = true) -> "Xiaomi"
            else -> "Android Device"
        }
    }
    
    /**
     * Get battery level
     */
    private fun getBatteryLevel(): Int? {
        return try {
            val batteryManager = context.getSystemService(Context.BATTERY_SERVICE) as android.os.BatteryManager
            batteryManager.getIntProperty(android.os.BatteryManager.BATTERY_PROPERTY_CAPACITY)
        } catch (e: Exception) {
            null
        }
    }
    
    /**
     * Check if screen is on
     */
    private fun isScreenOn(): Boolean {
        return try {
            val powerManager = context.getSystemService(Context.POWER_SERVICE) as android.os.PowerManager
            powerManager.isInteractive
        } catch (e: Exception) {
            false
        }
    }
}

/**
 * Crash report data
 */
@Serializable
data class CrashReport(
    val id: String,
    val timestamp: Long,
    val type: CrashType,
    val message: String,
    val stackTrace: String? = null,
    val appVersion: String,
    val osVersion: String,
    val deviceModel: String,
    val context: CrashContext
)

/**
 * Crash types
 */
@Serializable
enum class CrashType {
    NATIVE_CRASH,
    EXCEPTION,
    OUT_OF_MEMORY,
    ANR,
    NETWORK_ERROR,
    FILTER_ERROR,
    OTHER
}

/**
 * Crash context
 */
@Serializable
data class CrashContext(
    val filterRulesCount: Int? = null,
    val memoryUsageMB: Int? = null,
    val availableMemoryMB: Int? = null,
    val vpnActive: Boolean? = null,
    val batteryLevel: Int? = null,
    val screenOn: Boolean? = null,
    val customProperties: Map<String, String> = emptyMap()
)

/**
 * Crash statistics
 */
data class CrashStatistics(
    val totalCrashes: Int,
    val crashesByType: Map<CrashType, Int>,
    val oldestCrash: Long?,
    val newestCrash: Long?
)