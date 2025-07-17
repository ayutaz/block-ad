use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

/// Privacy-focused analytics system
/// Only collects anonymous usage data to improve the app
pub struct Analytics {
    /// Event storage
    events: Arc<Mutex<EventStore>>,
    /// Session information
    session: Arc<Mutex<SessionInfo>>,
    /// Whether analytics is enabled
    enabled: bool,
    /// Anonymous user ID
    anonymous_id: String,
}

#[derive(Debug, Clone)]
struct EventStore {
    /// Recent events for batching
    events: Vec<AnalyticsEvent>,
    /// Aggregated metrics
    metrics: HashMap<String, MetricValue>,
    /// Daily active users tracking
    daily_active: HashMap<String, DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    /// Event name
    pub name: String,
    /// Event category
    pub category: EventCategory,
    /// Event properties (no PII)
    pub properties: HashMap<String, serde_json::Value>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Session ID
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventCategory {
    /// App lifecycle events
    Lifecycle,
    /// User actions
    Action,
    /// Performance metrics
    Performance,
    /// Errors (non-crash)
    Error,
    /// Feature usage
    Feature,
}

#[derive(Debug, Clone)]
struct SessionInfo {
    /// Session ID
    id: String,
    /// Session start time
    start_time: DateTime<Utc>,
    /// Last activity time
    last_activity: DateTime<Utc>,
    /// Session properties
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
enum MetricValue {
    Count(u64),
    Sum(f64),
    Average { sum: f64, count: u64 },
    Distribution(Vec<f64>),
}

impl Analytics {
    /// Create a new analytics instance
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(EventStore {
                events: Vec::with_capacity(1000),
                metrics: HashMap::new(),
                daily_active: HashMap::new(),
            })),
            session: Arc::new(Mutex::new(SessionInfo {
                id: uuid::Uuid::new_v4().to_string(),
                start_time: Utc::now(),
                last_activity: Utc::now(),
                properties: HashMap::new(),
            })),
            enabled: true,
            anonymous_id: Self::generate_anonymous_id(),
        }
    }

    /// Enable or disable analytics
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            // Clear all data when disabled
            if let Ok(mut store) = self.events.lock() {
                store.events.clear();
                store.metrics.clear();
                store.daily_active.clear();
            }
        }
    }

    /// Track an event
    pub fn track_event(&self, name: &str, category: EventCategory, properties: HashMap<String, serde_json::Value>) {
        if !self.enabled {
            return;
        }

        let session_id = if let Ok(session) = self.session.lock() {
            session.id.clone()
        } else {
            return;
        };

        let event = AnalyticsEvent {
            name: name.to_string(),
            category,
            properties,
            timestamp: Utc::now(),
            session_id,
        };

        if let Ok(mut store) = self.events.lock() {
            // Add to events buffer
            if store.events.len() < 1000 {
                store.events.push(event);
            }

            // Update last activity
            if let Ok(mut session) = self.session.lock() {
                session.last_activity = Utc::now();
            }
        }
    }

    /// Track a simple action
    pub fn track_action(&self, action: &str) {
        self.track_event(action, EventCategory::Action, HashMap::new());
    }

    /// Track feature usage
    pub fn track_feature(&self, feature: &str, properties: HashMap<String, serde_json::Value>) {
        self.track_event(feature, EventCategory::Feature, properties);
    }

    /// Track performance metric
    pub fn track_performance(&self, metric: &str, value: f64) {
        let mut properties = HashMap::new();
        properties.insert("value".to_string(), serde_json::json!(value));
        self.track_event(metric, EventCategory::Performance, properties);
    }

    /// Track error (non-crash)
    pub fn track_error(&self, error: &str, error_type: &str) {
        let mut properties = HashMap::new();
        properties.insert("error_type".to_string(), serde_json::json!(error_type));
        self.track_event(error, EventCategory::Error, properties);
    }

    /// Record a metric value
    pub fn record_metric(&self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        if let Ok(mut store) = self.events.lock() {
            let metric = store.metrics.entry(name.to_string()).or_insert(MetricValue::Count(0));
            
            match metric {
                MetricValue::Count(count) => {
                    *count += 1;
                }
                MetricValue::Sum(sum) => {
                    *sum += value;
                }
                MetricValue::Average { sum, count } => {
                    *sum += value;
                    *count += 1;
                }
                MetricValue::Distribution(values) => {
                    values.push(value);
                    if values.len() > 1000 {
                        // Keep only recent values
                        values.drain(0..500);
                    }
                }
            }
        }
    }

    /// Increment a counter metric
    pub fn increment_counter(&self, name: &str) {
        self.record_metric(name, 1.0);
    }

    /// Start a new session
    pub fn start_session(&self) {
        if let Ok(mut session) = self.session.lock() {
            session.id = uuid::Uuid::new_v4().to_string();
            session.start_time = Utc::now();
            session.last_activity = Utc::now();
            session.properties.clear();
        }

        // Track session start
        self.track_event("session_start", EventCategory::Lifecycle, HashMap::new());
        
        // Update daily active user
        if let Ok(mut store) = self.events.lock() {
            store.daily_active.insert(self.anonymous_id.clone(), Utc::now());
            
            // Clean up old entries (older than 30 days)
            let cutoff = Utc::now() - Duration::days(30);
            store.daily_active.retain(|_, timestamp| *timestamp > cutoff);
        }
    }

    /// End the current session
    pub fn end_session(&self) {
        if let Ok(session) = self.session.lock() {
            let duration = (session.last_activity - session.start_time).num_seconds();
            let mut properties = HashMap::new();
            properties.insert("duration_seconds".to_string(), serde_json::json!(duration));
            
            drop(session); // Release lock before tracking event
            self.track_event("session_end", EventCategory::Lifecycle, properties);
        }
    }

    /// Get analytics summary
    pub fn get_summary(&self) -> AnalyticsSummary {
        let mut summary = AnalyticsSummary::default();

        if let Ok(store) = self.events.lock() {
            // Count events by category
            for event in &store.events {
                let category_name = match event.category {
                    EventCategory::Lifecycle => "lifecycle",
                    EventCategory::Action => "action",
                    EventCategory::Performance => "performance",
                    EventCategory::Error => "error",
                    EventCategory::Feature => "feature",
                };
                *summary.events_by_category.entry(category_name.to_string()).or_insert(0) += 1;
            }

            // Get metric summaries
            for (name, value) in &store.metrics {
                let metric_summary = match value {
                    MetricValue::Count(count) => {
                        format!("Count: {}", count)
                    }
                    MetricValue::Sum(sum) => {
                        format!("Sum: {:.2}", sum)
                    }
                    MetricValue::Average { sum, count } => {
                        let avg = if *count > 0 { sum / *count as f64 } else { 0.0 };
                        format!("Avg: {:.2} (n={})", avg, count)
                    }
                    MetricValue::Distribution(values) => {
                        if values.is_empty() {
                            "No data".to_string()
                        } else {
                            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
                            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                            let avg = values.iter().sum::<f64>() / values.len() as f64;
                            format!("Min: {:.2}, Max: {:.2}, Avg: {:.2}", min, max, avg)
                        }
                    }
                };
                summary.metrics.insert(name.clone(), metric_summary);
            }

            summary.total_events = store.events.len();
            summary.daily_active_users = store.daily_active.len();
        }

        if let Ok(session) = self.session.lock() {
            summary.current_session_duration = (Utc::now() - session.start_time).num_seconds();
        }

        summary
    }

    /// Export events for analysis
    pub fn export_events(&self, limit: usize) -> Vec<AnalyticsEvent> {
        if let Ok(store) = self.events.lock() {
            store.events.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all analytics data
    pub fn clear(&self) {
        if let Ok(mut store) = self.events.lock() {
            store.events.clear();
            store.metrics.clear();
            store.daily_active.clear();
        }
    }

    /// Generate anonymous ID based on device characteristics
    fn generate_anonymous_id() -> String {
        // In a real implementation, this would generate a stable anonymous ID
        // based on device characteristics, without collecting PII
        uuid::Uuid::new_v4().to_string()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_events: usize,
    pub events_by_category: HashMap<String, usize>,
    pub metrics: HashMap<String, String>,
    pub daily_active_users: usize,
    pub current_session_duration: i64,
}

/// Pre-defined analytics events
pub mod events {
    use super::*;

    /// Track app launch
    pub fn app_launch(analytics: &Analytics, launch_time_ms: u64) {
        let mut properties = HashMap::new();
        properties.insert("launch_time_ms".to_string(), serde_json::json!(launch_time_ms));
        analytics.track_event("app_launch", EventCategory::Lifecycle, properties);
    }

    /// Track VPN connection
    pub fn vpn_connected(analytics: &Analytics, connection_time_ms: u64) {
        let mut properties = HashMap::new();
        properties.insert("connection_time_ms".to_string(), serde_json::json!(connection_time_ms));
        analytics.track_event("vpn_connected", EventCategory::Action, properties);
    }

    /// Track VPN disconnection
    pub fn vpn_disconnected(analytics: &Analytics, reason: &str) {
        let mut properties = HashMap::new();
        properties.insert("reason".to_string(), serde_json::json!(reason));
        analytics.track_event("vpn_disconnected", EventCategory::Action, properties);
    }

    /// Track filter update
    pub fn filter_updated(analytics: &Analytics, rules_count: usize, duration_ms: u64) {
        let mut properties = HashMap::new();
        properties.insert("rules_count".to_string(), serde_json::json!(rules_count));
        properties.insert("duration_ms".to_string(), serde_json::json!(duration_ms));
        analytics.track_event("filter_updated", EventCategory::Action, properties);
    }

    /// Track custom rule added
    pub fn custom_rule_added(analytics: &Analytics, rule_type: &str) {
        let mut properties = HashMap::new();
        properties.insert("rule_type".to_string(), serde_json::json!(rule_type));
        analytics.track_event("custom_rule_added", EventCategory::Feature, properties);
    }

    /// Track ad blocked
    pub fn ad_blocked(analytics: &Analytics, domain: &str, size_bytes: u64) {
        // Don't track the actual domain for privacy, just the size
        let mut properties = HashMap::new();
        properties.insert("size_bytes".to_string(), serde_json::json!(size_bytes));
        properties.insert("domain_length".to_string(), serde_json::json!(domain.len()));
        analytics.track_event("ad_blocked", EventCategory::Action, properties);
    }

    /// Track performance warning
    pub fn performance_warning(analytics: &Analytics, metric: &str, value: f64) {
        let mut properties = HashMap::new();
        properties.insert("metric".to_string(), serde_json::json!(metric));
        properties.insert("value".to_string(), serde_json::json!(value));
        analytics.track_event("performance_warning", EventCategory::Performance, properties);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_basic() {
        let analytics = Analytics::new();
        
        // Track some events
        analytics.track_action("test_action");
        analytics.track_feature("test_feature", HashMap::new());
        analytics.track_performance("load_time", 150.0);
        
        // Check summary
        let summary = analytics.get_summary();
        assert_eq!(summary.total_events, 3);
        assert_eq!(*summary.events_by_category.get("action").unwrap(), 1);
        assert_eq!(*summary.events_by_category.get("feature").unwrap(), 1);
        assert_eq!(*summary.events_by_category.get("performance").unwrap(), 1);
    }

    #[test]
    fn test_metrics() {
        let analytics = Analytics::new();
        
        // Record some metrics
        analytics.increment_counter("clicks");
        analytics.increment_counter("clicks");
        analytics.record_metric("response_time", 100.0);
        analytics.record_metric("response_time", 200.0);
        
        let summary = analytics.get_summary();
        assert!(summary.metrics.contains_key("clicks"));
        assert!(summary.metrics.contains_key("response_time"));
    }

    #[test]
    fn test_disabled_analytics() {
        let mut analytics = Analytics::new();
        analytics.set_enabled(false);
        
        // Track events while disabled
        analytics.track_action("test_action");
        
        // Should not record anything
        let summary = analytics.get_summary();
        assert_eq!(summary.total_events, 0);
    }
}