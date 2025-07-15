import Foundation
import BackgroundTasks
import UserNotifications

/// Manages background tasks for filter updates
public class BackgroundTaskManager {
    
    static let shared = BackgroundTaskManager()
    
    private let taskIdentifier = "com.adblock.filter-update"
    private var filterListUpdater: FilterListUpdater?
    
    private init() {}
    
    /// Configure the background task manager
    public func configure(with engine: AdBlockEngine) {
        self.filterListUpdater = FilterListUpdater(engine: engine)
        registerBackgroundTask()
        scheduleBackgroundTask()
    }
    
    /// Register background task
    private func registerBackgroundTask() {
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: taskIdentifier,
            using: nil
        ) { task in
            guard let processingTask = task as? BGProcessingTask else {
                task.setTaskCompleted(success: false)
                return
            }
            self.handleFilterUpdateTask(task: processingTask)
        }
    }
    
    /// Schedule background task
    public func scheduleBackgroundTask() {
        let request = BGProcessingTaskRequest(identifier: taskIdentifier)
        request.requiresNetworkConnectivity = true
        request.requiresExternalPower = false
        
        // Schedule to run once per day
        request.earliestBeginDate = Date(timeIntervalSinceNow: 24 * 60 * 60)
        
        do {
            try BGTaskScheduler.shared.submit(request)
        } catch {
            print("Failed to schedule background task: \(error)")
        }
    }
    
    /// Handle background filter update task
    private func handleFilterUpdateTask(task: BGProcessingTask) {
        // Schedule the next update
        scheduleBackgroundTask()
        
        // Create a task to update filters
        let updateTask = Task {
            guard let updater = filterListUpdater else {
                task.setTaskCompleted(success: false)
                return
            }
            
            // Check if auto-update is enabled
            let autoUpdateEnabled = UserDefaults.standard.bool(forKey: "filter_auto_update")
            guard autoUpdateEnabled else {
                task.setTaskCompleted(success: true)
                return
            }
            
            // Check if update is needed
            guard updater.needsUpdate() else {
                task.setTaskCompleted(success: true)
                return
            }
            
            // Update filters
            updater.updateFilterLists { result in
                switch result {
                case .success(let message):
                    // Send notification if update was successful
                    self.sendUpdateNotification(message: message)
                    task.setTaskCompleted(success: true)
                    
                case .failure(let error):
                    print("Filter update failed: \(error)")
                    task.setTaskCompleted(success: false)
                }
            }
        }
        
        // Handle expiration
        task.expirationHandler = {
            updateTask.cancel()
            task.setTaskCompleted(success: false)
        }
    }
    
    /// Send notification about filter update
    private func sendUpdateNotification(message: String) {
        let content = UNMutableNotificationContent()
        content.title = "AdBlock"
        content.body = message
        content.sound = .default
        
        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil
        )
        
        UNUserNotificationCenter.current().add(request) { error in
            if let error = error {
                print("Failed to send notification: \(error)")
            }
        }
    }
    
    /// Enable auto-update
    public func setAutoUpdateEnabled(_ enabled: Bool) {
        UserDefaults.standard.set(enabled, forKey: "filter_auto_update")
        
        if enabled {
            scheduleBackgroundTask()
        } else {
            // Cancel scheduled tasks
            BGTaskScheduler.shared.cancel(taskRequestWithIdentifier: taskIdentifier)
        }
    }
    
    /// Check if auto-update is enabled
    public func isAutoUpdateEnabled() -> Bool {
        return UserDefaults.standard.bool(forKey: "filter_auto_update")
    }
}