package com.adblock.filter

import android.content.Context
import android.content.SharedPreferences
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.net.URL
import java.util.Date

/**
 * Manages filter list downloads and updates
 */
class FilterListManager(private val context: Context) {
    
    companion object {
        private const val PREFS_NAME = "filter_prefs"
        private const val KEY_LAST_UPDATE = "last_update"
        private const val KEY_AUTO_UPDATE = "auto_update"
        private const val FILTER_FILE_NAME = "easylist.txt"
        
        // Default filter list URLs
        private val DEFAULT_FILTER_URLS = listOf(
            "https://easylist.to/easylist/easylist.txt",
            "https://raw.githubusercontent.com/easylist/easylist/master/easylist/easylist.txt"
        )
        
        // Update interval: 7 days
        private const val UPDATE_INTERVAL_MS = 7L * 24 * 60 * 60 * 1000
    }
    
    private val prefs: SharedPreferences = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    private val filterFile = File(context.filesDir, FILTER_FILE_NAME)
    
    /**
     * Check if filter list needs update
     */
    fun needsUpdate(): Boolean {
        val lastUpdate = prefs.getLong(KEY_LAST_UPDATE, 0)
        val now = System.currentTimeMillis()
        return now - lastUpdate > UPDATE_INTERVAL_MS
    }
    
    /**
     * Update filter lists from remote sources
     */
    suspend fun updateFilterLists(): Result<String> = withContext(Dispatchers.IO) {
        try {
            // Try each URL until one succeeds
            for (url in DEFAULT_FILTER_URLS) {
                try {
                    val filterContent = downloadFilterList(url)
                    if (filterContent.isNotEmpty()) {
                        // Save to file
                        filterFile.writeText(filterContent)
                        
                        // Update last update time
                        prefs.edit().putLong(KEY_LAST_UPDATE, System.currentTimeMillis()).apply()
                        
                        return@withContext Result.success(filterContent)
                    }
                } catch (e: Exception) {
                    // Try next URL
                    continue
                }
            }
            
            return@withContext Result.failure(Exception("Failed to download filter list from all sources"))
        } catch (e: Exception) {
            return@withContext Result.failure(e)
        }
    }
    
    /**
     * Download filter list from URL
     */
    private fun downloadFilterList(urlString: String): String {
        val url = URL(urlString)
        val connection = url.openConnection()
        connection.connectTimeout = 30000 // 30 seconds
        connection.readTimeout = 30000
        
        return connection.getInputStream().bufferedReader().use { it.readText() }
    }
    
    /**
     * Load filter list from local storage
     */
    fun loadLocalFilterList(): String? {
        return if (filterFile.exists()) {
            try {
                filterFile.readText()
            } catch (e: Exception) {
                null
            }
        } else {
            null
        }
    }
    
    /**
     * Get last update time
     */
    fun getLastUpdateTime(): Date? {
        val lastUpdate = prefs.getLong(KEY_LAST_UPDATE, 0)
        return if (lastUpdate > 0) Date(lastUpdate) else null
    }
    
    /**
     * Set auto-update enabled
     */
    fun setAutoUpdateEnabled(enabled: Boolean) {
        prefs.edit().putBoolean(KEY_AUTO_UPDATE, enabled).apply()
    }
    
    /**
     * Check if auto-update is enabled
     */
    fun isAutoUpdateEnabled(): Boolean {
        return prefs.getBoolean(KEY_AUTO_UPDATE, true)
    }
    
    /**
     * Clear all filter data
     */
    fun clearFilterData() {
        filterFile.delete()
        prefs.edit().clear().apply()
    }
}