use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Memory optimization settings and utilities
pub struct MemoryOptimizer {
    /// Maximum cache size in bytes
    max_cache_size: AtomicUsize,
    /// Current cache size in bytes
    current_cache_size: AtomicUsize,
    /// LRU cache entries
    cache_entries: Arc<parking_lot::RwLock<LruCache>>,
    /// Memory pressure callback
    memory_pressure_callback: Option<Box<dyn Fn() + Send + Sync>>,
}

struct LruCache {
    entries: HashMap<String, CacheEntry>,
    access_order: Vec<String>,
    max_entries: usize,
}

struct CacheEntry {
    data: Vec<u8>,
    size: usize,
    last_accessed: Instant,
    access_count: u32,
}

impl MemoryOptimizer {
    /// Create a new memory optimizer with target of 30MB
    pub fn new() -> Self {
        Self {
            max_cache_size: AtomicUsize::new(30 * 1024 * 1024), // 30MB
            current_cache_size: AtomicUsize::new(0),
            cache_entries: Arc::new(parking_lot::RwLock::new(LruCache::new(1000))),
            memory_pressure_callback: None,
        }
    }

    /// Set maximum memory usage in bytes
    pub fn set_max_memory(&self, bytes: usize) {
        self.max_cache_size.store(bytes, Ordering::Relaxed);
        self.evict_if_needed();
    }

    /// Get current memory usage
    pub fn get_memory_usage(&self) -> usize {
        self.current_cache_size.load(Ordering::Relaxed)
    }

    /// Add data to cache with memory management
    pub fn cache_data(&self, key: String, data: Vec<u8>) {
        let size = data.len();
        
        // Check if this would exceed memory limit
        let current = self.current_cache_size.load(Ordering::Relaxed);
        let max = self.max_cache_size.load(Ordering::Relaxed);
        
        if current + size > max {
            // Evict old entries to make room
            self.evict_to_fit(size);
        }

        // Add to cache
        let mut cache = self.cache_entries.write();
        
        // Remove old entry if exists
        if let Some(old_entry) = cache.entries.remove(&key) {
            self.current_cache_size.fetch_sub(old_entry.size, Ordering::Relaxed);
        }

        // Add new entry
        cache.entries.insert(key.clone(), CacheEntry {
            data,
            size,
            last_accessed: Instant::now(),
            access_count: 1,
        });
        
        cache.access_order.push(key);
        self.current_cache_size.fetch_add(size, Ordering::Relaxed);

        // Trim access order if too large
        if cache.access_order.len() > cache.max_entries * 2 {
            cache.compact_access_order();
        }
    }

    /// Get data from cache
    pub fn get_cached(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.cache_entries.write();
        
        if let Some(entry) = cache.entries.get_mut(key) {
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            Some(entry.data.clone())
        } else {
            None
        }
    }

    /// Clear all cache to free memory
    pub fn clear_cache(&self) {
        let mut cache = self.cache_entries.write();
        cache.entries.clear();
        cache.access_order.clear();
        self.current_cache_size.store(0, Ordering::Relaxed);
    }

    /// Evict least recently used entries to fit new data
    fn evict_to_fit(&self, needed_size: usize) {
        let mut cache = self.cache_entries.write();
        let max = self.max_cache_size.load(Ordering::Relaxed);
        let mut current = self.current_cache_size.load(Ordering::Relaxed);
        
        // Sort by last accessed time
        let mut entries: Vec<_> = cache.entries.iter()
            .map(|(k, v)| (k.clone(), v.last_accessed, v.size))
            .collect();
        entries.sort_by_key(|(_, time, _)| *time);

        // Evict oldest entries until we have enough space
        for (key, _, size) in entries {
            if current + needed_size <= max {
                break;
            }

            if let Some(entry) = cache.entries.remove(&key) {
                current -= entry.size;
                self.current_cache_size.fetch_sub(entry.size, Ordering::Relaxed);
            }
        }

        // Clean up access order
        cache.compact_access_order();
    }

    /// Evict entries if over memory limit
    fn evict_if_needed(&self) {
        let current = self.current_cache_size.load(Ordering::Relaxed);
        let max = self.max_cache_size.load(Ordering::Relaxed);
        
        if current > max {
            let to_evict = current - max;
            self.evict_to_fit(to_evict);
        }
    }

    /// Set callback for memory pressure events
    pub fn set_memory_pressure_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.memory_pressure_callback = Some(Box::new(callback));
    }

    /// Trigger memory pressure handling
    pub fn handle_memory_pressure(&self) {
        // Clear 50% of cache on memory pressure
        let mut cache = self.cache_entries.write();
        let entries_to_remove = cache.entries.len() / 2;
        
        let mut removed = 0;
        let keys: Vec<_> = cache.access_order.iter()
            .take(entries_to_remove)
            .cloned()
            .collect();

        for key in keys {
            if let Some(entry) = cache.entries.remove(&key) {
                self.current_cache_size.fetch_sub(entry.size, Ordering::Relaxed);
                removed += 1;
            }
        }

        cache.compact_access_order();
        
        // Call callback if set
        if let Some(ref callback) = self.memory_pressure_callback {
            callback();
        }
        
        log::info!("Memory pressure handled: removed {} cache entries", removed);
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let cache = self.cache_entries.read();
        
        MemoryStats {
            total_memory_bytes: self.get_memory_usage(),
            cache_entries: cache.entries.len(),
            max_memory_bytes: self.max_cache_size.load(Ordering::Relaxed),
            cache_hit_rate: 0.0, // Would need to track this separately
        }
    }
}

impl LruCache {
    fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: Vec::new(),
            max_entries,
        }
    }

    fn compact_access_order(&mut self) {
        // Remove duplicates and non-existent keys
        let mut seen = std::collections::HashSet::new();
        self.access_order.retain(|key| {
            self.entries.contains_key(key) && seen.insert(key.clone())
        });
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_memory_bytes: usize,
    pub cache_entries: usize,
    pub max_memory_bytes: usize,
    pub cache_hit_rate: f32,
}

/// Memory-efficient string interning for filter rules
pub struct StringInterner {
    strings: parking_lot::RwLock<HashMap<String, Arc<str>>>,
    total_size: AtomicUsize,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: parking_lot::RwLock::new(HashMap::new()),
            total_size: AtomicUsize::new(0),
        }
    }

    /// Intern a string to save memory on duplicates
    pub fn intern(&self, s: &str) -> Arc<str> {
        let mut strings = self.strings.write();
        
        if let Some(interned) = strings.get(s) {
            Arc::clone(interned)
        } else {
            let arc = Arc::from(s);
            strings.insert(s.to_string(), Arc::clone(&arc));
            self.total_size.fetch_add(s.len(), Ordering::Relaxed);
            arc
        }
    }

    /// Get total memory used by interned strings
    pub fn memory_usage(&self) -> usize {
        self.total_size.load(Ordering::Relaxed)
    }

    /// Clear all interned strings
    pub fn clear(&self) {
        let mut strings = self.strings.write();
        strings.clear();
        self.total_size.store(0, Ordering::Relaxed);
    }
}

/// Optimized filter rule storage
pub struct OptimizedFilterStorage {
    /// Interned domain patterns
    domains: Vec<Arc<str>>,
    /// Bit flags for rule properties  
    flags: Vec<u8>,
    /// Memory optimizer
    memory: Arc<MemoryOptimizer>,
}

impl OptimizedFilterStorage {
    pub fn new(memory: Arc<MemoryOptimizer>) -> Self {
        Self {
            domains: Vec::new(),
            flags: Vec::new(),
            memory,
        }
    }

    /// Add a filter rule with memory optimization
    pub fn add_rule(&mut self, domain: &str, flags: u8, interner: &StringInterner) {
        let interned = interner.intern(domain);
        self.domains.push(interned);
        self.flags.push(flags);

        // Check memory usage periodically
        if self.domains.len() % 1000 == 0 {
            let usage = self.estimate_memory_usage();
            if usage > 20 * 1024 * 1024 { // 20MB for filters
                log::warn!("Filter storage using {}MB", usage / 1024 / 1024);
            }
        }
    }

    /// Estimate memory usage of filter storage
    fn estimate_memory_usage(&self) -> usize {
        // Vec overhead + string data + flags
        self.domains.len() * std::mem::size_of::<Arc<str>>() +
        self.domains.iter().map(|s| s.len()).sum::<usize>() +
        self.flags.len()
    }

    /// Compact storage to reduce memory
    pub fn compact(&mut self) {
        // Remove duplicates
        let mut seen = std::collections::HashSet::new();
        let mut new_domains = Vec::new();
        let mut new_flags = Vec::new();

        for (domain, flag) in self.domains.iter().zip(self.flags.iter()) {
            if seen.insert(domain.clone()) {
                new_domains.push(domain.clone());
                new_flags.push(*flag);
            }
        }

        self.domains = new_domains;
        self.flags = new_flags;
        
        // Shrink to fit
        self.domains.shrink_to_fit();
        self.flags.shrink_to_fit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_optimizer() {
        let optimizer = MemoryOptimizer::new();
        
        // Set max to 1MB for testing
        optimizer.set_max_memory(1024 * 1024);
        
        // Add some data
        optimizer.cache_data("test1".to_string(), vec![0u8; 512 * 1024]);
        assert_eq!(optimizer.get_memory_usage(), 512 * 1024);
        
        // Add more data that triggers eviction
        optimizer.cache_data("test2".to_string(), vec![0u8; 768 * 1024]);
        
        // Should have evicted first entry
        assert!(optimizer.get_memory_usage() <= 1024 * 1024);
        assert!(optimizer.get_cached("test1").is_none());
        assert!(optimizer.get_cached("test2").is_some());
    }

    #[test]
    fn test_string_interner() {
        let interner = StringInterner::new();
        
        let s1 = interner.intern("example.com");
        let s2 = interner.intern("example.com");
        
        // Should be the same Arc
        assert!(Arc::ptr_eq(&s1, &s2));
        
        // Memory usage should be counted once
        assert_eq!(interner.memory_usage(), "example.com".len());
    }
}