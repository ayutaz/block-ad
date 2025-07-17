use std::fmt::Write as FmtWrite;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::path::Path;
use std::fs::{self, File};
use std::io::Write;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;

/// Privacy-respecting crash reporter
/// Only collects technical data necessary for debugging
pub struct CrashReporter {
    /// Recent crash reports
    reports: Arc<Mutex<VecDeque<CrashReport>>>,
    /// Maximum number of reports to keep in memory
    max_reports: usize,
    /// Path to persist crash reports
    reports_path: Option<String>,
    /// Whether crash reporting is enabled
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashReport {
    /// Unique identifier for this crash
    pub id: String,
    /// Timestamp when the crash occurred
    pub timestamp: DateTime<Utc>,
    /// Type of crash/error
    pub error_type: CrashType,
    /// Error message
    pub message: String,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// App version
    pub app_version: String,
    /// OS version
    pub os_version: String,
    /// Device model (anonymized)
    pub device_model: String,
    /// Additional context
    pub context: CrashContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrashType {
    /// Native crash (segfault, etc)
    Native,
    /// Unhandled exception
    Exception,
    /// Out of memory
    OutOfMemory,
    /// ANR (Application Not Responding)
    ANR,
    /// Network error
    NetworkError,
    /// Filter parsing error
    FilterError,
    /// Other errors
    Other(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrashContext {
    /// Number of active filter rules
    pub filter_rules_count: Option<u32>,
    /// Memory usage in MB
    pub memory_usage_mb: Option<u32>,
    /// Whether VPN was active
    pub vpn_active: Option<bool>,
    /// Last user action (anonymized)
    pub last_action: Option<String>,
    /// Custom properties (no PII)
    pub custom_properties: std::collections::HashMap<String, String>,
}

impl CrashReporter {
    /// Create a new crash reporter
    pub fn new(reports_path: Option<String>) -> Self {
        let mut reporter = Self {
            reports: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            max_reports: 100,
            reports_path,
            enabled: true,
        };

        // Load existing reports if path is provided
        if let Some(ref path) = reporter.reports_path {
            reporter.load_reports(path);
        }

        reporter
    }

    /// Enable or disable crash reporting
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            // Clear all reports when disabled
            if let Ok(mut reports) = self.reports.lock() {
                reports.clear();
            }
        }
    }

    /// Report a crash
    pub fn report_crash(&self, error_type: CrashType, message: String, context: CrashContext) {
        if !self.enabled {
            return;
        }

        let report = CrashReport {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            error_type,
            message: Self::sanitize_message(&message),
            stack_trace: Self::capture_stack_trace(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            os_version: Self::get_os_version(),
            device_model: Self::get_device_model(),
            context,
        };

        // Add to in-memory queue
        if let Ok(mut reports) = self.reports.lock() {
            if reports.len() >= self.max_reports {
                reports.pop_front();
            }
            reports.push_back(report.clone());
        }

        // Persist to disk if configured
        if let Some(ref path) = self.reports_path {
            self.save_report(&report, path);
        }

        log::error!("Crash reported: {:?} - {}", error_type, message);
    }

    /// Report an exception with automatic context capture
    pub fn report_exception(&self, exception: &str, context: Option<CrashContext>) {
        let ctx = context.unwrap_or_else(|| self.capture_context());
        self.report_crash(
            CrashType::Exception,
            exception.to_string(),
            ctx,
        );
    }

    /// Report out of memory condition
    pub fn report_oom(&self, memory_usage_mb: u32) {
        let mut context = self.capture_context();
        context.memory_usage_mb = Some(memory_usage_mb);
        
        self.report_crash(
            CrashType::OutOfMemory,
            format!("Out of memory at {}MB", memory_usage_mb),
            context,
        );
    }

    /// Capture current context for crash reporting
    fn capture_context(&self) -> CrashContext {
        CrashContext {
            filter_rules_count: None, // Would be set by caller
            memory_usage_mb: Self::get_memory_usage(),
            vpn_active: None, // Would be set by caller
            last_action: None,
            custom_properties: std::collections::HashMap::new(),
        }
    }

    /// Get recent crash reports
    pub fn get_reports(&self, limit: usize) -> Vec<CrashReport> {
        if let Ok(reports) = self.reports.lock() {
            reports.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get crash statistics
    pub fn get_statistics(&self) -> CrashStatistics {
        if let Ok(reports) = self.reports.lock() {
            let total = reports.len();
            let mut by_type = std::collections::HashMap::new();
            
            for report in reports.iter() {
                let type_name = match &report.error_type {
                    CrashType::Native => "Native",
                    CrashType::Exception => "Exception",
                    CrashType::OutOfMemory => "OOM",
                    CrashType::ANR => "ANR",
                    CrashType::NetworkError => "Network",
                    CrashType::FilterError => "Filter",
                    CrashType::Other(_) => "Other",
                };
                *by_type.entry(type_name.to_string()).or_insert(0) += 1;
            }

            CrashStatistics {
                total_crashes: total,
                crashes_by_type: by_type,
                oldest_crash: reports.front().map(|r| r.timestamp),
                newest_crash: reports.back().map(|r| r.timestamp),
            }
        } else {
            CrashStatistics::default()
        }
    }

    /// Clear all crash reports
    pub fn clear_reports(&self) {
        if let Ok(mut reports) = self.reports.lock() {
            reports.clear();
        }
        
        // Also clear persisted reports
        if let Some(ref path) = self.reports_path {
            let _ = fs::remove_dir_all(path);
        }
    }

    /// Load reports from disk
    fn load_reports(&self, path: &str) {
        let reports_dir = Path::new(path);
        if !reports_dir.exists() {
            return;
        }

        let mut loaded_reports = Vec::new();
        
        if let Ok(entries) = fs::read_dir(reports_dir) {
            for entry in entries.flatten() {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Ok(report) = serde_json::from_str::<CrashReport>(&content) {
                        loaded_reports.push(report);
                    }
                }
            }
        }

        // Sort by timestamp and keep only recent ones
        loaded_reports.sort_by_key(|r| r.timestamp);
        let to_keep = loaded_reports.into_iter()
            .rev()
            .take(self.max_reports)
            .collect::<Vec<_>>();

        if let Ok(mut reports) = self.reports.lock() {
            for report in to_keep.into_iter().rev() {
                reports.push_back(report);
            }
        }
    }

    /// Save a single report to disk
    fn save_report(&self, report: &CrashReport, base_path: &str) {
        let reports_dir = Path::new(base_path);
        if let Err(e) = fs::create_dir_all(reports_dir) {
            log::error!("Failed to create crash reports directory: {}", e);
            return;
        }

        let filename = format!("crash_{}.json", report.id);
        let file_path = reports_dir.join(filename);
        
        if let Ok(mut file) = File::create(file_path) {
            if let Ok(json) = serde_json::to_string_pretty(report) {
                let _ = file.write_all(json.as_bytes());
            }
        }
    }

    /// Sanitize message to remove any potential PII
    fn sanitize_message(message: &str) -> String {
        // Static regex patterns for better performance and error handling
        static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
                .expect("Invalid email regex pattern")
        });
        
        static IP_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b")
                .expect("Invalid IP regex pattern")
        });
        
        static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b")
                .expect("Invalid phone regex pattern")
        });
        
        let mut sanitized = message.to_string();
        
        // Remove email addresses
        sanitized = EMAIL_REGEX.replace_all(&sanitized, "[EMAIL]").to_string();
        
        // Remove IP addresses
        sanitized = IP_REGEX.replace_all(&sanitized, "[IP]").to_string();
        
        // Remove phone numbers
        sanitized = PHONE_REGEX.replace_all(&sanitized, "[PHONE]").to_string();
        
        // Truncate if too long
        if sanitized.len() > 1000 {
            sanitized.truncate(1000);
            sanitized.push_str("...");
        }
        
        sanitized
    }

    /// Capture stack trace (platform-specific)
    fn capture_stack_trace() -> Option<String> {
        #[cfg(feature = "backtrace")]
        {
            use backtrace::Backtrace;
            let backtrace = Backtrace::new();
            Some(format!("{:?}", backtrace))
        }
        
        #[cfg(not(feature = "backtrace"))]
        None
    }

    /// Get OS version
    fn get_os_version() -> String {
        #[cfg(target_os = "android")]
        {
            "Android".to_string() // Would get actual version from JNI
        }
        
        #[cfg(target_os = "ios")]
        {
            "iOS".to_string() // Would get actual version from system
        }
        
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
        }
    }

    /// Get anonymized device model
    fn get_device_model() -> String {
        // Return generic device category instead of specific model
        #[cfg(target_os = "android")]
        {
            "Android Device".to_string()
        }
        
        #[cfg(target_os = "ios")]
        {
            "iOS Device".to_string()
        }
        
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            "Unknown Device".to_string()
        }
    }

    /// Get current memory usage in MB
    fn get_memory_usage() -> Option<u32> {
        // This would be implemented platform-specifically
        // For now, return None
        None
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrashStatistics {
    pub total_crashes: usize,
    pub crashes_by_type: std::collections::HashMap<String, usize>,
    pub oldest_crash: Option<DateTime<Utc>>,
    pub newest_crash: Option<DateTime<Utc>>,
}

/// Panic handler that reports crashes
pub fn install_panic_handler(reporter: Arc<CrashReporter>) {
    std::panic::set_hook(Box::new(move |panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = if let Some(location) = panic_info.location() {
            format!(" at {}:{}:{}", location.file(), location.line(), location.column())
        } else {
            String::new()
        };

        reporter.report_crash(
            CrashType::Native,
            format!("Panic: {}{}", message, location),
            CrashContext::default(),
        );
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_reporter() {
        let reporter = CrashReporter::new(None);
        
        // Report a crash
        reporter.report_crash(
            CrashType::Exception,
            "Test exception".to_string(),
            CrashContext::default(),
        );
        
        // Check that it was recorded
        let reports = reporter.get_reports(10);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].message, "Test exception");
    }

    #[test]
    fn test_sanitize_message() {
        let message = "Error for user@example.com at 192.168.1.1";
        let sanitized = CrashReporter::sanitize_message(message);
        assert_eq!(sanitized, "Error for [EMAIL] at [IP]");
    }

    #[test]
    fn test_crash_statistics() {
        let reporter = CrashReporter::new(None);
        
        // Report different types of crashes
        reporter.report_crash(CrashType::Exception, "Test 1".to_string(), CrashContext::default());
        reporter.report_crash(CrashType::OutOfMemory, "Test 2".to_string(), CrashContext::default());
        reporter.report_crash(CrashType::Exception, "Test 3".to_string(), CrashContext::default());
        
        let stats = reporter.get_statistics();
        assert_eq!(stats.total_crashes, 3);
        assert_eq!(*stats.crashes_by_type.get("Exception").unwrap(), 2);
        assert_eq!(*stats.crashes_by_type.get("OOM").unwrap(), 1);
    }
}