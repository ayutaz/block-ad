import SwiftUI
import BackgroundTasks

@main
struct AdBlockApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) var appDelegate
    
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

class AppDelegate: NSObject, UIApplicationDelegate {
    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey : Any]? = nil) -> Bool {
        // Configure background tasks
        do {
            let engine = try AdBlockEngine()
            BackgroundTaskManager.shared.configure(with: engine)
        } catch {
            print("Failed to initialize AdBlockEngine: \(error)")
        }
        
        // Request notification permissions
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { granted, error in
            if granted {
                print("Notification permission granted")
            }
        }
        
        return true
    }
}