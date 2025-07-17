import Foundation
import UIKit

/// Privacy-respecting crash reporter for iOS
public class CrashReporter {
    
    // MARK: - Properties
    
    static let shared = CrashReporter()
    
    private let crashQueue = DispatchQueue(label: "com.adblock.crashreporter", qos: .background)
    private let crashDir: URL
    private let maxReports = 100
    private var recentCrashes: [CrashReport] = []
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()
    
    public var isEnabled: Bool = true {
        didSet {
            if !isEnabled {
                clearAllReports()
            }
        }
    }
    
    // MARK: - Initialization
    
    private init() {
        // Setup crash directory
        let documentsPath = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        self.crashDir = documentsPath.appendingPathComponent("crashes", isDirectory: true)
        
        // Create directory if needed
        try? FileManager.default.createDirectory(at: crashDir, withIntermediateDirectories: true)
        
        // Setup JSON encoder/decoder
        encoder.outputFormatting = .prettyPrinted
        encoder.dateEncodingStrategy = .iso8601
        decoder.dateDecodingStrategy = .iso8601
        
        // Load existing crashes
        loadRecentCrashes()
        
        // Install signal handlers
        installSignalHandlers()
        
        // Monitor for low memory warnings
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(handleMemoryWarning),
            name: UIApplication.didReceiveMemoryWarningNotification,
            object: nil
        )
    }
    
    // MARK: - Public Methods
    
    /// Report a crash
    public func reportCrash(
        type: CrashType,
        message: String,
        error: Error? = nil,
        context: CrashContext = CrashContext()
    ) {
        guard isEnabled else { return }
        
        let report = CrashReport(
            id: UUID().uuidString,
            timestamp: Date(),
            type: type,
            message: sanitizeMessage(message),
            stackTrace: error?.localizedDescription ?? Thread.callStackSymbols.joined(separator: "\n"),
            appVersion: Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "Unknown",
            osVersion: "iOS \(UIDevice.current.systemVersion)",
            deviceModel: getAnonymizedDeviceModel(),
            context: context
        )
        
        crashQueue.async { [weak self] in
            self?.addReport(report)
            self?.saveReport(report)
        }
        
        NSLog("Crash reported: \(type) - \(message)")
    }
    
    /// Report an exception
    public func reportException(_ error: Error, context: CrashContext? = nil) {
        reportCrash(
            type: .exception,
            message: error.localizedDescription,
            error: error,
            context: context ?? captureContext()
        )
    }
    
    /// Report ANR (Application Not Responding)
    public func reportANR(_ message: String) {
        reportCrash(
            type: .anr,
            message: message,
            context: captureContext()
        )
    }
    
    /// Report out of memory
    public func reportOOM(availableMemoryMB: Int) {
        var context = captureContext()
        context.memoryUsageMB = availableMemoryMB
        
        reportCrash(
            type: .outOfMemory,
            message: "Out of memory. Available: \(availableMemoryMB)MB",
            context: context
        )
    }
    
    /// Get recent crash reports
    public func getRecentReports(limit: Int = 10) -> [CrashReport] {
        return Array(recentCrashes.suffix(limit))
    }
    
    /// Get crash statistics
    public func getStatistics() -> CrashStatistics {
        let byType = Dictionary(grouping: recentCrashes, by: { $0.type })
            .mapValues { $0.count }
        
        return CrashStatistics(
            totalCrashes: recentCrashes.count,
            crashesByType: byType,
            oldestCrash: recentCrashes.first?.timestamp,
            newestCrash: recentCrashes.last?.timestamp
        )
    }
    
    /// Clear all crash reports
    public func clearAllReports() {
        crashQueue.async { [weak self] in
            self?.recentCrashes.removeAll()
            
            // Remove all crash files
            if let crashDir = self?.crashDir {
                try? FileManager.default.contentsOfDirectory(at: crashDir, includingPropertiesForKeys: nil)
                    .forEach { try? FileManager.default.removeItem(at: $0) }
            }
        }
    }
    
    // MARK: - Private Methods
    
    private func captureContext() -> CrashContext {
        let processInfo = ProcessInfo.processInfo
        let thermalState = processInfo.thermalState
        
        return CrashContext(
            filterRulesCount: nil, // Would be set by caller
            memoryUsageMB: getCurrentMemoryUsage(),
            availableMemoryMB: getAvailableMemory(),
            vpnActive: nil, // Would be set by caller
            batteryLevel: Int(UIDevice.current.batteryLevel * 100),
            thermalState: thermalStateString(thermalState),
            customProperties: [:]
        )
    }
    
    private func addReport(_ report: CrashReport) {
        recentCrashes.append(report)
        
        // Limit number of reports in memory
        if recentCrashes.count > maxReports {
            recentCrashes.removeFirst(recentCrashes.count - maxReports)
        }
    }
    
    private func saveReport(_ report: CrashReport) {
        let filename = "crash_\(report.id).json"
        let fileURL = crashDir.appendingPathComponent(filename)
        
        do {
            let data = try encoder.encode(report)
            try data.write(to: fileURL)
            
            // Clean up old files
            cleanupOldReports()
        } catch {
            NSLog("Failed to save crash report: \(error)")
        }
    }
    
    private func loadRecentCrashes() {
        crashQueue.async { [weak self] in
            guard let self = self else { return }
            
            do {
                let files = try FileManager.default.contentsOfDirectory(
                    at: self.crashDir,
                    includingPropertiesForKeys: [.contentModificationDateKey]
                )
                .filter { $0.pathExtension == "json" }
                .sorted { (url1, url2) in
                    let date1 = try? url1.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate
                    let date2 = try? url2.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate
                    return (date1 ?? Date.distantPast) > (date2 ?? Date.distantPast)
                }
                .prefix(self.maxReports)
                
                for file in files {
                    if let data = try? Data(contentsOf: file),
                       let report = try? self.decoder.decode(CrashReport.self, from: data) {
                        self.recentCrashes.append(report)
                    }
                }
            } catch {
                NSLog("Failed to load crash reports: \(error)")
            }
        }
    }
    
    private func cleanupOldReports() {
        do {
            let files = try FileManager.default.contentsOfDirectory(
                at: crashDir,
                includingPropertiesForKeys: [.contentModificationDateKey]
            )
            .filter { $0.pathExtension == "json" }
            .sorted { (url1, url2) in
                let date1 = try? url1.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate
                let date2 = try? url2.resourceValues(forKeys: [.contentModificationDateKey]).contentModificationDate
                return (date1 ?? Date.distantPast) < (date2 ?? Date.distantPast)
            }
            
            // Remove oldest files if we have too many
            if files.count > maxReports {
                for file in files.prefix(files.count - maxReports) {
                    try? FileManager.default.removeItem(at: file)
                }
            }
        } catch {
            NSLog("Failed to cleanup old reports: \(error)")
        }
    }
    
    private func sanitizeMessage(_ message: String) -> String {
        var sanitized = message
        
        // Remove email addresses
        let emailRegex = try? NSRegularExpression(pattern: "[A-Z0-9a-z._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,64}")
        sanitized = emailRegex?.stringByReplacingMatches(
            in: sanitized,
            range: NSRange(location: 0, length: sanitized.count),
            withTemplate: "[EMAIL]"
        ) ?? sanitized
        
        // Remove IP addresses
        let ipRegex = try? NSRegularExpression(pattern: "\\b(?:[0-9]{1,3}\\.){3}[0-9]{1,3}\\b")
        sanitized = ipRegex?.stringByReplacingMatches(
            in: sanitized,
            range: NSRange(location: 0, length: sanitized.count),
            withTemplate: "[IP]"
        ) ?? sanitized
        
        // Truncate if too long
        if sanitized.count > 1000 {
            sanitized = String(sanitized.prefix(1000)) + "..."
        }
        
        return sanitized
    }
    
    private func getAnonymizedDeviceModel() -> String {
        let model = UIDevice.current.model
        
        if model.contains("iPhone") {
            return "iPhone"
        } else if model.contains("iPad") {
            return "iPad"
        } else if model.contains("iPod") {
            return "iPod touch"
        } else {
            return "iOS Device"
        }
    }
    
    private func getCurrentMemoryUsage() -> Int {
        var info = mach_task_basic_info()
        var count = mach_msg_type_number_t(MemoryLayout<mach_task_basic_info>.size) / 4
        
        let result = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: 1) {
                task_info(mach_task_self_,
                         task_flavor_t(MACH_TASK_BASIC_INFO),
                         $0,
                         &count)
            }
        }
        
        return result == KERN_SUCCESS ? Int(info.resident_size / 1024 / 1024) : 0
    }
    
    private func getAvailableMemory() -> Int {
        let memoryInfo = ProcessInfo.processInfo.physicalMemory
        return Int(memoryInfo / 1024 / 1024)
    }
    
    private func thermalStateString(_ state: ProcessInfo.ThermalState) -> String {
        switch state {
        case .nominal: return "nominal"
        case .fair: return "fair"
        case .serious: return "serious"
        case .critical: return "critical"
        @unknown default: return "unknown"
        }
    }
    
    @objc private func handleMemoryWarning() {
        reportOOM(availableMemoryMB: getAvailableMemory())
    }
    
    // MARK: - Signal Handlers
    
    private func installSignalHandlers() {
        signal(SIGABRT) { signal in
            CrashReporter.shared.reportCrash(
                type: .nativeCrash,
                message: "Signal SIGABRT",
                context: CrashReporter.shared.captureContext()
            )
        }
        
        signal(SIGSEGV) { signal in
            CrashReporter.shared.reportCrash(
                type: .nativeCrash,
                message: "Signal SIGSEGV",
                context: CrashReporter.shared.captureContext()
            )
        }
        
        signal(SIGBUS) { signal in
            CrashReporter.shared.reportCrash(
                type: .nativeCrash,
                message: "Signal SIGBUS",
                context: CrashReporter.shared.captureContext()
            )
        }
    }
}

// MARK: - Data Models

/// Crash report
public struct CrashReport: Codable {
    let id: String
    let timestamp: Date
    let type: CrashType
    let message: String
    let stackTrace: String?
    let appVersion: String
    let osVersion: String
    let deviceModel: String
    let context: CrashContext
}

/// Crash types
public enum CrashType: String, Codable {
    case nativeCrash = "native_crash"
    case exception = "exception"
    case outOfMemory = "out_of_memory"
    case anr = "anr"
    case networkError = "network_error"
    case filterError = "filter_error"
    case other = "other"
}

/// Crash context
public struct CrashContext: Codable {
    var filterRulesCount: Int?
    var memoryUsageMB: Int?
    var availableMemoryMB: Int?
    var vpnActive: Bool?
    var batteryLevel: Int?
    var thermalState: String?
    var customProperties: [String: String]
}

/// Crash statistics
public struct CrashStatistics {
    let totalCrashes: Int
    let crashesByType: [CrashType: Int]
    let oldestCrash: Date?
    let newestCrash: Date?
}