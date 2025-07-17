package com.adblock

import android.app.Application
import com.adblock.analytics.Analytics
import com.adblock.analytics.CrashReporter
import com.adblock.analytics.CrashType
import com.adblock.analytics.EventCategory
import com.adblock.core.AdBlockEngine
import com.adblock.service.AdBlockVpnService
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch

class AdBlockApplication : Application() {
    
    companion object {
        lateinit var instance: AdBlockApplication
            private set
    }
    
    lateinit var crashReporter: CrashReporter
        private set
    
    lateinit var analytics: Analytics
        private set
    
    override fun onCreate() {
        super.onCreate()
        instance = this
        
        // Initialize crash reporter
        crashReporter = CrashReporter.getInstance(this)
        
        // Initialize analytics
        analytics = Analytics.getInstance(this)
        
        // Track app launch
        val startTime = System.currentTimeMillis()
        
        // Initialize AdBlock engine
        GlobalScope.launch {
            try {
                AdBlockEngine.initialize(this@AdBlockApplication)
                
                val launchTime = System.currentTimeMillis() - startTime
                analytics.trackAppLaunch(launchTime)
                
            } catch (e: Exception) {
                crashReporter.reportException(e)
            }
        }
        
        // Set up low memory callback
        registerComponentCallbacks(object : android.content.ComponentCallbacks2 {
            override fun onConfigurationChanged(newConfig: android.content.res.Configuration) {}
            
            override fun onLowMemory() {
                val memoryInfo = android.app.ActivityManager.MemoryInfo()
                val activityManager = getSystemService(ACTIVITY_SERVICE) as android.app.ActivityManager
                activityManager.getMemoryInfo(memoryInfo)
                
                val availableMemoryMB = (memoryInfo.availMem / 1024 / 1024).toInt()
                crashReporter.reportOOM(availableMemoryMB)
                
                // Try to free memory
                AdBlockEngine.instance?.optimizeMemory()
            }
            
            override fun onTrimMemory(level: Int) {
                when (level) {
                    TRIM_MEMORY_RUNNING_CRITICAL -> {
                        analytics.trackPerformanceWarning("memory_critical", level.toDouble())
                        AdBlockEngine.instance?.optimizeMemory()
                    }
                    TRIM_MEMORY_RUNNING_LOW -> {
                        analytics.trackPerformanceWarning("memory_low", level.toDouble())
                    }
                }
            }
        })
    }
    
    override fun onTerminate() {
        super.onTerminate()
        
        // End analytics session
        analytics.endSession()
    }
}