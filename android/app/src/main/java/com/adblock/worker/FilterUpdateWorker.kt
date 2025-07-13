package com.adblock.worker

import android.content.Context
import android.app.NotificationChannel
import android.app.NotificationManager
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.work.*
import com.adblock.AdBlockEngine
import com.adblock.filter.FilterListManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.concurrent.TimeUnit

/**
 * Worker class for automatic filter list updates
 */
class FilterUpdateWorker(
    context: Context,
    params: WorkerParameters
) : CoroutineWorker(context, params) {
    
    companion object {
        const val WORK_NAME = "filter_update_work"
        const val NOTIFICATION_CHANNEL_ID = "filter_updates"
        const val NOTIFICATION_ID = 1001
        
        /**
         * Schedule periodic filter updates
         */
        fun schedulePeriodicUpdate(context: Context) {
            val constraints = Constraints.Builder()
                .setRequiredNetworkType(NetworkType.CONNECTED)
                .setRequiresBatteryNotLow(true)
                .build()
            
            val updateRequest = PeriodicWorkRequestBuilder<FilterUpdateWorker>(
                7, TimeUnit.DAYS,  // Run every 7 days
                1, TimeUnit.HOURS  // Flex interval of 1 hour
            )
                .setConstraints(constraints)
                .setBackoffCriteria(
                    BackoffPolicy.LINEAR,
                    30, TimeUnit.MINUTES
                )
                .addTag("filter_update")
                .build()
            
            WorkManager.getInstance(context)
                .enqueueUniquePeriodicWork(
                    WORK_NAME,
                    ExistingPeriodicWorkPolicy.KEEP,
                    updateRequest
                )
        }
        
        /**
         * Schedule immediate one-time update
         */
        fun scheduleImmediateUpdate(context: Context) {
            val constraints = Constraints.Builder()
                .setRequiredNetworkType(NetworkType.CONNECTED)
                .build()
            
            val updateRequest = OneTimeWorkRequestBuilder<FilterUpdateWorker>()
                .setConstraints(constraints)
                .addTag("filter_update_immediate")
                .build()
            
            WorkManager.getInstance(context)
                .enqueue(updateRequest)
        }
        
        /**
         * Cancel all scheduled updates
         */
        fun cancelAllUpdates(context: Context) {
            WorkManager.getInstance(context)
                .cancelUniqueWork(WORK_NAME)
        }
    }
    
    override suspend fun doWork(): Result = withContext(Dispatchers.IO) {
        val filterListManager = FilterListManager(applicationContext)
        
        // Check if auto-update is enabled
        if (!filterListManager.isAutoUpdateEnabled()) {
            return@withContext Result.success()
        }
        
        // Check if update is needed
        if (!filterListManager.needsUpdate()) {
            return@withContext Result.success()
        }
        
        try {
            // Show notification that update is in progress
            showUpdateNotification("Updating ad block filters...", ongoing = true)
            
            // Perform the update
            val result = filterListManager.updateFilterLists()
            
            if (result.isSuccess) {
                // Load updated filters into engine
                val filterContent = result.getOrNull()
                if (!filterContent.isNullOrEmpty()) {
                    val engine = AdBlockEngine.getInstance()
                    engine.loadFilters(filterContent)
                    
                    // Show success notification
                    showUpdateNotification("Filter lists updated successfully", ongoing = false)
                }
                
                Result.success()
            } else {
                // Show error notification
                showUpdateNotification(
                    "Failed to update filter lists", 
                    ongoing = false,
                    isError = true
                )
                
                // Retry later
                Result.retry()
            }
        } catch (e: Exception) {
            // Show error notification
            showUpdateNotification(
                "Error updating filter lists: ${e.message}", 
                ongoing = false,
                isError = true
            )
            
            Result.retry()
        }
    }
    
    private fun showUpdateNotification(
        message: String, 
        ongoing: Boolean = false,
        isError: Boolean = false
    ) {
        val notificationManager = applicationContext.getSystemService(
            Context.NOTIFICATION_SERVICE
        ) as NotificationManager
        
        // Create notification channel for Android O and above
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "Filter Updates",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Notifications for filter list updates"
            }
            notificationManager.createNotificationChannel(channel)
        }
        
        val notification = NotificationCompat.Builder(applicationContext, NOTIFICATION_CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle("AdBlock")
            .setContentText(message)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(ongoing)
            .setAutoCancel(!ongoing)
            .apply {
                if (isError) {
                    setCategory(NotificationCompat.CATEGORY_ERROR)
                    setSmallIcon(android.R.drawable.ic_dialog_alert)
                }
            }
            .build()
        
        notificationManager.notify(NOTIFICATION_ID, notification)
        
        // Cancel notification after 5 seconds if not ongoing
        if (!ongoing) {
            Thread.sleep(5000)
            notificationManager.cancel(NOTIFICATION_ID)
        }
    }
}