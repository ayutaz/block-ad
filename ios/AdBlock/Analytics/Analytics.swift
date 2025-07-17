import Foundation
import UIKit

/// Privacy-focused analytics for AdBlock
/// Only collects anonymous usage data to improve the app
public class Analytics {
    
    // MARK: - Properties
    
    static let shared = Analytics()
    
    private let analyticsQueue = DispatchQueue(label: "com.adblock.analytics", qos: .background)
    private let userDefaults = UserDefaults.standard
    private var events: [AnalyticsEvent] = []
    private var metrics: [String: MetricValue] = [:]
    private let maxEvents = 1000
    
    private var sessionId = UUID().uuidString
    private var sessionStartTime = Date()
    private var lastActivityTime = Date()
    
    public let anonymousId: String
    
    public var isEnabled: Bool {
        get { userDefaults.bool(forKey: "analytics_enabled") }
        set {
            userDefaults.set(newValue, forKey: "analytics_enabled")
            if !newValue {
                clearAllData()
            }
        }
    }
    
    // MARK: - Initialization
    
    private init() {
        // Get or create anonymous ID
        if let savedId = userDefaults.string(forKey: "anonymous_id") {
            self.anonymousId = savedId
        } else {
            self.anonymousId = UUID().uuidString
            userDefaults.set(anonymousId, forKey: "anonymous_id")
        }
        
        // Set default enabled state
        if userDefaults.object(forKey: "analytics_enabled") == nil {
            userDefaults.set(true, forKey: "analytics_enabled")
        }
        
        // Listen for app lifecycle events
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(appDidBecomeActive),
            name: UIApplication.didBecomeActiveNotification,
            object: nil
        )
        
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(appWillResignActive),
            name: UIApplication.willResignActiveNotification,
            object: nil
        )
    }
    
    // MARK: - Public Methods
    
    /// Track an event
    public func trackEvent(
        _ name: String,
        category: EventCategory,
        properties: [String: Any] = [:]
    ) {
        guard isEnabled else { return }
        
        analyticsQueue.async { [weak self] in
            guard let self = self else { return }
            
            let event = AnalyticsEvent(
                name: name,
                category: category,
                properties: properties,
                timestamp: Date(),
                sessionId: self.sessionId
            )
            
            self.events.append(event)
            
            // Limit number of events
            if self.events.count > self.maxEvents {
                self.events.removeFirst(self.events.count - self.maxEvents)
            }
            
            self.lastActivityTime = Date()
        }
    }
    
    /// Track a simple action
    public func trackAction(_ action: String) {
        trackEvent(action, category: .action)
    }
    
    /// Track feature usage
    public func trackFeature(_ feature: String, properties: [String: Any] = [:]) {
        trackEvent(feature, category: .feature, properties: properties)
    }
    
    /// Track performance metric
    public func trackPerformance(_ metric: String, value: Double) {
        trackEvent(metric, category: .performance, properties: ["value": value])
        recordMetric(metric, value: value)
    }
    
    /// Track error (non-crash)
    public func trackError(_ error: String, errorType: String) {
        trackEvent(error, category: .error, properties: ["error_type": errorType])
    }
    
    /// Record a metric value
    public func recordMetric(_ name: String, value: Double) {
        guard isEnabled else { return }
        
        analyticsQueue.async { [weak self] in
            guard let self = self else { return }
            
            if let existing = self.metrics[name] {
                switch existing {
                case .count(let count):
                    self.metrics[name] = .count(count + 1)
                case .sum(let sum):
                    self.metrics[name] = .sum(sum + value)
                case .average(let sum, let count):
                    self.metrics[name] = .average(sum: sum + value, count: count + 1)
                case .distribution(var values):
                    values.append(value)
                    if values.count > 1000 {
                        values.removeFirst(500)
                    }
                    self.metrics[name] = .distribution(values)
                }
            } else {
                self.metrics[name] = .sum(value)
            }
        }
    }
    
    /// Increment a counter
    public func incrementCounter(_ name: String) {
        guard isEnabled else { return }
        
        analyticsQueue.async { [weak self] in
            guard let self = self else { return }
            
            if case .count(let count) = self.metrics[name] {
                self.metrics[name] = .count(count + 1)
            } else {
                self.metrics[name] = .count(1)
            }
        }
    }
    
    /// Start a new session
    public func startSession() {
        sessionId = UUID().uuidString
        sessionStartTime = Date()
        lastActivityTime = Date()
        
        trackEvent("session_start", category: .lifecycle)
    }
    
    /// End the current session
    public func endSession() {
        let duration = lastActivityTime.timeIntervalSince(sessionStartTime)
        trackEvent("session_end", category: .lifecycle, properties: ["duration_seconds": Int(duration)])
    }
    
    /// Get analytics summary
    public func getSummary() -> AnalyticsSummary {
        var summary = AnalyticsSummary()
        
        analyticsQueue.sync {
            // Count events by category
            summary.eventsByCategory = Dictionary(grouping: events, by: { $0.category })
                .mapValues { $0.count }
            
            // Summarize metrics
            summary.metrics = metrics.compactMapValues { value in
                switch value {
                case .count(let count):
                    return "Count: \(count)"
                case .sum(let sum):
                    return "Sum: \(String(format: "%.2f", sum))"
                case .average(let sum, let count):
                    let avg = count > 0 ? sum / Double(count) : 0.0
                    return "Avg: \(String(format: "%.2f", avg)) (n=\(count))"
                case .distribution(let values):
                    if values.isEmpty {
                        return "No data"
                    } else {
                        let min = values.min() ?? 0
                        let max = values.max() ?? 0
                        let avg = values.reduce(0, +) / Double(values.count)
                        return "Min: \(String(format: "%.2f", min)), Max: \(String(format: "%.2f", max)), Avg: \(String(format: "%.2f", avg))"
                    }
                }
            }
            
            summary.totalEvents = events.count
            summary.sessionDuration = Int(Date().timeIntervalSince(sessionStartTime))
        }
        
        return summary
    }
    
    /// Export events for analysis
    public func exportEvents(limit: Int = 100) -> [AnalyticsEvent] {
        return analyticsQueue.sync {
            Array(events.suffix(limit))
        }
    }
    
    /// Clear all analytics data
    public func clearAllData() {
        analyticsQueue.async { [weak self] in
            self?.events.removeAll()
            self?.metrics.removeAll()
        }
    }
    
    // MARK: - Pre-defined Events
    
    public func trackAppLaunch(launchTimeMs: Int64) {
        trackEvent("app_launch", category: .lifecycle, properties: ["launch_time_ms": launchTimeMs])
    }
    
    public func trackVpnConnected(connectionTimeMs: Int64) {
        trackEvent("vpn_connected", category: .action, properties: ["connection_time_ms": connectionTimeMs])
        incrementCounter("vpn_connections")
    }
    
    public func trackVpnDisconnected(reason: String) {
        trackEvent("vpn_disconnected", category: .action, properties: ["reason": reason])
    }
    
    public func trackFilterUpdated(rulesCount: Int, durationMs: Int64) {
        trackEvent("filter_updated", category: .action, properties: [
            "rules_count": rulesCount,
            "duration_ms": durationMs
        ])
    }
    
    public func trackCustomRuleAdded(ruleType: String) {
        trackEvent("custom_rule_added", category: .feature, properties: ["rule_type": ruleType])
        incrementCounter("custom_rules")
    }
    
    public func trackAdBlocked(sizeBytes: Int64) {
        // Don't track domains for privacy
        incrementCounter("ads_blocked")
        recordMetric("bytes_saved", value: Double(sizeBytes))
    }
    
    public func trackPerformanceWarning(metric: String, value: Double) {
        trackEvent("performance_warning", category: .performance, properties: [
            "metric": metric,
            "value": value
        ])
    }
    
    // MARK: - Private Methods
    
    @objc private func appDidBecomeActive() {
        startSession()
    }
    
    @objc private func appWillResignActive() {
        endSession()
    }
}

// MARK: - Data Models

/// Analytics event
public struct AnalyticsEvent: Codable {
    let name: String
    let category: EventCategory
    let properties: [String: Any]
    let timestamp: Date
    let sessionId: String
    
    enum CodingKeys: String, CodingKey {
        case name, category, timestamp, sessionId
    }
    
    public init(name: String, category: EventCategory, properties: [String: Any], timestamp: Date, sessionId: String) {
        self.name = name
        self.category = category
        self.properties = properties
        self.timestamp = timestamp
        self.sessionId = sessionId
    }
    
    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        name = try container.decode(String.self, forKey: .name)
        category = try container.decode(EventCategory.self, forKey: .category)
        timestamp = try container.decode(Date.self, forKey: .timestamp)
        sessionId = try container.decode(String.self, forKey: .sessionId)
        properties = [:] // Properties not decoded for simplicity
    }
    
    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(name, forKey: .name)
        try container.encode(category, forKey: .category)
        try container.encode(timestamp, forKey: .timestamp)
        try container.encode(sessionId, forKey: .sessionId)
        // Properties not encoded for simplicity
    }
}

/// Event categories
public enum EventCategory: String, Codable {
    case lifecycle
    case action
    case performance
    case error
    case feature
}

/// Metric value types
enum MetricValue {
    case count(Int64)
    case sum(Double)
    case average(sum: Double, count: Int64)
    case distribution([Double])
}

/// Analytics summary
public struct AnalyticsSummary {
    var totalEvents: Int = 0
    var eventsByCategory: [EventCategory: Int] = [:]
    var metrics: [String: String] = [:]
    var sessionDuration: Int = 0
}