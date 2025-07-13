use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics for the ad blocking engine
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    inner: Arc<MetricsInner>,
}

#[derive(Debug)]
struct MetricsInner {
    // Request processing metrics
    total_requests: AtomicU64,
    blocked_requests: AtomicU64,
    allowed_requests: AtomicU64,
    
    // Performance metrics
    total_processing_time_ns: AtomicU64,
    avg_processing_time_ns: AtomicU64,
    max_processing_time_ns: AtomicU64,
    min_processing_time_ns: AtomicU64,
    
    // Memory metrics
    filter_count: AtomicUsize,
    memory_usage_bytes: AtomicUsize,
    
    // Error metrics
    parse_errors: AtomicU64,
    match_errors: AtomicU64,
    
    // Cache metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_size: AtomicUsize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    /// Create new performance metrics instance
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                total_requests: AtomicU64::new(0),
                blocked_requests: AtomicU64::new(0),
                allowed_requests: AtomicU64::new(0),
                total_processing_time_ns: AtomicU64::new(0),
                avg_processing_time_ns: AtomicU64::new(0),
                max_processing_time_ns: AtomicU64::new(0),
                min_processing_time_ns: AtomicU64::new(u64::MAX),
                filter_count: AtomicUsize::new(0),
                memory_usage_bytes: AtomicUsize::new(0),
                parse_errors: AtomicU64::new(0),
                match_errors: AtomicU64::new(0),
                cache_hits: AtomicU64::new(0),
                cache_misses: AtomicU64::new(0),
                cache_size: AtomicUsize::new(0),
            }),
        }
    }
    
    /// Record a request processing
    pub fn record_request(&self, blocked: bool, processing_time: Duration) {
        let time_ns = processing_time.as_nanos() as u64;
        
        self.inner.total_requests.fetch_add(1, Ordering::Relaxed);
        
        if blocked {
            self.inner.blocked_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.inner.allowed_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        // Update processing time metrics
        self.inner.total_processing_time_ns.fetch_add(time_ns, Ordering::Relaxed);
        
        // Update max processing time
        self.inner.max_processing_time_ns.fetch_max(time_ns, Ordering::Relaxed);
        
        // Update min processing time
        loop {
            let current_min = self.inner.min_processing_time_ns.load(Ordering::Relaxed);
            if time_ns >= current_min {
                break;
            }
            if self.inner.min_processing_time_ns
                .compare_exchange_weak(current_min, time_ns, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        
        // Calculate average
        let total_requests = self.inner.total_requests.load(Ordering::Relaxed);
        let total_time = self.inner.total_processing_time_ns.load(Ordering::Relaxed);
        if total_requests > 0 {
            let avg = total_time / total_requests;
            self.inner.avg_processing_time_ns.store(avg, Ordering::Relaxed);
        }
    }
    
    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.inner.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.inner.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Update filter count
    pub fn set_filter_count(&self, count: usize) {
        self.inner.filter_count.store(count, Ordering::Relaxed);
    }
    
    /// Update memory usage
    pub fn set_memory_usage(&self, bytes: usize) {
        self.inner.memory_usage_bytes.store(bytes, Ordering::Relaxed);
    }
    
    /// Update cache size
    pub fn set_cache_size(&self, size: usize) {
        self.inner.cache_size.store(size, Ordering::Relaxed);
    }
    
    /// Record parse error
    pub fn record_parse_error(&self) {
        self.inner.parse_errors.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record match error
    pub fn record_match_error(&self) {
        self.inner.match_errors.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get current metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: self.inner.total_requests.load(Ordering::Relaxed),
            blocked_requests: self.inner.blocked_requests.load(Ordering::Relaxed),
            allowed_requests: self.inner.allowed_requests.load(Ordering::Relaxed),
            avg_processing_time_ns: self.inner.avg_processing_time_ns.load(Ordering::Relaxed),
            max_processing_time_ns: self.inner.max_processing_time_ns.load(Ordering::Relaxed),
            min_processing_time_ns: {
                let min = self.inner.min_processing_time_ns.load(Ordering::Relaxed);
                if min == u64::MAX { 0 } else { min }
            },
            filter_count: self.inner.filter_count.load(Ordering::Relaxed),
            memory_usage_bytes: self.inner.memory_usage_bytes.load(Ordering::Relaxed),
            parse_errors: self.inner.parse_errors.load(Ordering::Relaxed),
            match_errors: self.inner.match_errors.load(Ordering::Relaxed),
            cache_hits: self.inner.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.inner.cache_misses.load(Ordering::Relaxed),
            cache_size: self.inner.cache_size.load(Ordering::Relaxed),
            block_rate: self.calculate_block_rate(),
            cache_hit_rate: self.calculate_cache_hit_rate(),
        }
    }
    
    /// Reset all metrics
    pub fn reset(&self) {
        self.inner.total_requests.store(0, Ordering::Relaxed);
        self.inner.blocked_requests.store(0, Ordering::Relaxed);
        self.inner.allowed_requests.store(0, Ordering::Relaxed);
        self.inner.total_processing_time_ns.store(0, Ordering::Relaxed);
        self.inner.avg_processing_time_ns.store(0, Ordering::Relaxed);
        self.inner.max_processing_time_ns.store(0, Ordering::Relaxed);
        self.inner.min_processing_time_ns.store(u64::MAX, Ordering::Relaxed);
        self.inner.parse_errors.store(0, Ordering::Relaxed);
        self.inner.match_errors.store(0, Ordering::Relaxed);
        self.inner.cache_hits.store(0, Ordering::Relaxed);
        self.inner.cache_misses.store(0, Ordering::Relaxed);
    }
    
    fn calculate_block_rate(&self) -> f64 {
        let total = self.inner.total_requests.load(Ordering::Relaxed);
        let blocked = self.inner.blocked_requests.load(Ordering::Relaxed);
        
        if total > 0 {
            (blocked as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
    
    fn calculate_cache_hit_rate(&self) -> f64 {
        let hits = self.inner.cache_hits.load(Ordering::Relaxed);
        let misses = self.inner.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        
        if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Snapshot of performance metrics at a point in time
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub allowed_requests: u64,
    pub avg_processing_time_ns: u64,
    pub max_processing_time_ns: u64,
    pub min_processing_time_ns: u64,
    pub filter_count: usize,
    pub memory_usage_bytes: usize,
    pub parse_errors: u64,
    pub match_errors: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_size: usize,
    pub block_rate: f64,
    pub cache_hit_rate: f64,
}

impl MetricsSnapshot {
    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Create from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Performance timer for measuring request processing time
pub struct PerfTimer {
    start: Instant,
}

impl PerfTimer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }
    
    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_metrics_recording() {
        let metrics = PerformanceMetrics::new();
        
        // Record some requests
        metrics.record_request(true, Duration::from_nanos(1000));
        metrics.record_request(false, Duration::from_nanos(2000));
        metrics.record_request(true, Duration::from_nanos(1500));
        
        let snapshot = metrics.snapshot();
        
        assert_eq!(snapshot.total_requests, 3);
        assert_eq!(snapshot.blocked_requests, 2);
        assert_eq!(snapshot.allowed_requests, 1);
        assert_eq!(snapshot.avg_processing_time_ns, 1500);
        assert_eq!(snapshot.max_processing_time_ns, 2000);
        assert_eq!(snapshot.min_processing_time_ns, 1000);
        assert_eq!(snapshot.block_rate, 66.66666666666666);
    }
    
    #[test]
    fn test_cache_metrics() {
        let metrics = PerformanceMetrics::new();
        
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        let snapshot = metrics.snapshot();
        
        assert_eq!(snapshot.cache_hits, 2);
        assert_eq!(snapshot.cache_misses, 1);
        assert_eq!(snapshot.cache_hit_rate, 66.66666666666666);
    }
    
    #[test]
    fn test_concurrent_access() {
        let metrics = PerformanceMetrics::new();
        let metrics_clone = metrics.clone();
        
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                metrics_clone.record_request(true, Duration::from_nanos(1000));
            }
        });
        
        for _ in 0..1000 {
            metrics.record_request(false, Duration::from_nanos(2000));
        }
        
        handle.join().unwrap();
        
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_requests, 2000);
        assert_eq!(snapshot.blocked_requests, 1000);
        assert_eq!(snapshot.allowed_requests, 1000);
    }
}