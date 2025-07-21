//! Memory management and optimization for cost tracking
//!
//! This module provides memory pooling, resource management, and efficient
//! data structures to minimize memory allocation overhead.

use crate::cost::CostError;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Memory pool configuration
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    /// Initial pool size
    pub initial_size: usize,
    /// Maximum pool size
    pub max_size: usize,
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Maximum age for unused objects in seconds
    pub max_unused_age_secs: u64,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 1000,
            max_size: 10000,
            cleanup_interval_secs: 60,
            max_unused_age_secs: 300, // 5 minutes
        }
    }
}

/// Generic memory pool for reusable objects
pub struct MemoryPool<T> 
where 
    T: Default + Clone,
{
    /// Pool of available objects
    pool: Arc<Mutex<VecDeque<PooledObject<T>>>>,
    /// Pool configuration
    config: MemoryPoolConfig,
    /// Pool statistics
    stats: Arc<RwLock<PoolStats>>,
}

/// Object wrapper with metadata for pooling
#[derive(Debug)]
struct PooledObject<T> {
    /// The actual object
    object: T,
    /// When this object was last used
    last_used: Instant,
    /// Number of times this object has been reused
    reuse_count: usize,
}

/// Memory pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Total objects created
    pub total_created: usize,
    /// Objects currently in pool
    pub objects_in_pool: usize,
    /// Objects currently borrowed
    pub objects_borrowed: usize,
    /// Total reuses
    pub total_reuses: usize,
    /// Pool hits (successful borrows from pool)
    pub pool_hits: usize,
    /// Pool misses (new object creation required)
    pub pool_misses: usize,
    /// Objects cleaned up
    pub objects_cleaned: usize,
}

impl PoolStats {
    /// Calculate hit rate percentage
    pub fn hit_rate(&self) -> f64 {
        if self.pool_hits + self.pool_misses == 0 {
            0.0
        } else {
            self.pool_hits as f64 / (self.pool_hits + self.pool_misses) as f64 * 100.0
        }
    }
}

impl<T> MemoryPool<T> 
where
    T: Default + Clone,
{
    /// Create a new memory pool
    pub fn new(config: MemoryPoolConfig) -> Self {
        let mut pool = VecDeque::with_capacity(config.initial_size);
        
        // Pre-allocate initial objects
        for _ in 0..config.initial_size {
            pool.push_back(PooledObject {
                object: T::default(),
                last_used: Instant::now(),
                reuse_count: 0,
            });
        }
        
        let stats = PoolStats {
            total_created: config.initial_size,
            objects_in_pool: config.initial_size,
            objects_borrowed: 0,
            total_reuses: 0,
            pool_hits: 0,
            pool_misses: 0,
            objects_cleaned: 0,
        };

        Self {
            pool: Arc::new(Mutex::new(pool)),
            config,
            stats: Arc::new(RwLock::new(stats)),
        }
    }

    /// Borrow an object from the pool
    pub fn borrow(&self) -> PooledRef<T> {
        let mut pool = self.pool.lock().unwrap();
        let mut stats = self.stats.write().unwrap();
        
        if let Some(mut pooled_obj) = pool.pop_front() {
            // Found an object in the pool
            pooled_obj.last_used = Instant::now();
            pooled_obj.reuse_count += 1;
            
            stats.objects_in_pool -= 1;
            stats.objects_borrowed += 1;
            stats.pool_hits += 1;
            stats.total_reuses += 1;
            
            PooledRef {
                object: Some(pooled_obj.object.clone()),
                pool: Arc::clone(&self.pool),
                stats: Arc::clone(&self.stats),
            }
        } else {
            // Pool is empty, create new object
            let new_object = T::default();
            
            stats.total_created += 1;
            stats.objects_borrowed += 1;
            stats.pool_misses += 1;
            
            PooledRef {
                object: Some(new_object),
                pool: Arc::clone(&self.pool),
                stats: Arc::clone(&self.stats),
            }
        }
    }

    /// Clean up old objects from the pool
    pub fn cleanup(&self) {
        let mut pool = self.pool.lock().unwrap();
        let mut stats = self.stats.write().unwrap();
        
        let cutoff_time = Instant::now() - Duration::from_secs(self.config.max_unused_age_secs);
        let initial_size = pool.len();
        
        // Remove old objects
        pool.retain(|obj| obj.last_used > cutoff_time);
        
        let removed_count = initial_size - pool.len();
        stats.objects_in_pool = pool.len();
        stats.objects_cleaned += removed_count;
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.read().unwrap().clone()
    }
}

/// RAII wrapper for borrowed objects
pub struct PooledRef<T> 
where
    T: Default + Clone,
{
    object: Option<T>,
    pool: Arc<Mutex<VecDeque<PooledObject<T>>>>,
    stats: Arc<RwLock<PoolStats>>,
}

impl<T> PooledRef<T> 
where
    T: Default + Clone,
{
    /// Get a reference to the object
    pub fn get(&self) -> &T {
        self.object.as_ref().unwrap()
    }
    
    /// Get a mutable reference to the object
    pub fn get_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

impl<T> Drop for PooledRef<T> 
where
    T: Default + Clone,
{
    fn drop(&mut self) {
        if let Some(object) = self.object.take() {
            let mut pool = self.pool.lock().unwrap();
            let mut stats = self.stats.write().unwrap();
            
            // Return object to pool if there's space
            if pool.len() < 10000 { // Max pool size check
                pool.push_back(PooledObject {
                    object,
                    last_used: Instant::now(),
                    reuse_count: 0,
                });
                stats.objects_in_pool += 1;
            }
            
            stats.objects_borrowed -= 1;
        }
    }
}

/// String interning for reducing memory usage of repeated strings
pub struct StringInterner {
    /// Interned strings
    strings: Arc<RwLock<HashMap<String, Arc<str>>>>,
    /// Statistics
    stats: Arc<RwLock<InternerStats>>,
}

/// String interner statistics
#[derive(Debug, Clone, Default)]
pub struct InternerStats {
    /// Total intern requests
    pub total_requests: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Unique strings stored
    pub unique_strings: usize,
    /// Estimated memory saved
    pub memory_saved_bytes: usize,
}

impl StringInterner {
    /// Create a new string interner
    pub fn new() -> Self {
        Self {
            strings: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(InternerStats::default())),
        }
    }

    /// Intern a string, returning a shared reference
    pub fn intern(&self, s: &str) -> Arc<str> {
        let mut stats = self.stats.write().unwrap();
        stats.total_requests += 1;
        
        // Check if already interned (read lock first for performance)
        {
            let strings = self.strings.read().unwrap();
            if let Some(interned) = strings.get(s) {
                stats.cache_hits += 1;
                stats.memory_saved_bytes += s.len();
                return Arc::clone(interned);
            }
        }
        
        // Not found, intern it (write lock)
        let mut strings = self.strings.write().unwrap();
        // Double-check in case another thread added it
        if let Some(interned) = strings.get(s) {
            stats.cache_hits += 1;
            stats.memory_saved_bytes += s.len();
            return Arc::clone(interned);
        }
        
        let interned: Arc<str> = s.into();
        strings.insert(s.to_string(), Arc::clone(&interned));
        stats.unique_strings += 1;
        
        interned
    }

    /// Get interner statistics
    pub fn stats(&self) -> InternerStats {
        self.stats.read().unwrap().clone()
    }
}

/// Resource manager for efficient resource allocation and cleanup
pub struct ResourceManager {
    /// Memory pool for string buffers
    buffer_pool: MemoryPool<String>,
    /// String interner for endpoints and models
    string_interner: StringInterner,
    /// Resource limits
    limits: ResourceLimits,
    /// Background cleanup handle
    cleanup_handle: Option<std::thread::JoinHandle<()>>,
}

/// Resource usage limits
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum number of sessions
    pub max_sessions: usize,
    /// Maximum API calls per session
    pub max_api_calls_per_session: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 100 * 1024 * 1024, // 100 MB
            max_sessions: 10000,
            max_api_calls_per_session: 1000,
        }
    }
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(limits: ResourceLimits) -> Result<Self, CostError> {
        // Use string buffer pool for memory optimization
        let buffer_pool = MemoryPool::new(MemoryPoolConfig::default());
        let string_interner = StringInterner::new();
        
        Ok(Self {
            buffer_pool,
            string_interner,
            limits,
            cleanup_handle: None,
        })
    }

    /// Start background cleanup thread
    pub fn start_background_cleanup(&mut self) {
        if self.cleanup_handle.is_some() {
            return; // Already running
        }
        
        let pool = Arc::clone(&self.buffer_pool.pool);
        let stats = Arc::clone(&self.buffer_pool.stats);
        
        let handle = std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(60));
                
                // Cleanup old objects
                let mut pool_guard = pool.lock().unwrap();
                let mut stats_guard = stats.write().unwrap();
                
                let cutoff_time = Instant::now() - Duration::from_secs(300); // 5 minutes
                let initial_size = pool_guard.len();
                
                pool_guard.retain(|obj| obj.last_used > cutoff_time);
                
                let removed_count = initial_size - pool_guard.len();
                stats_guard.objects_in_pool = pool_guard.len();
                stats_guard.objects_cleaned += removed_count;
                
                drop(pool_guard);
                drop(stats_guard);
            }
        });
        
        self.cleanup_handle = Some(handle);
    }

    /// Borrow a string buffer from the pool
    pub fn borrow_string_buffer(&self) -> PooledRef<String> {
        self.buffer_pool.borrow()
    }

    /// Intern a string to reduce memory usage
    pub fn intern_string(&self, s: &str) -> Arc<str> {
        self.string_interner.intern(s)
    }

    /// Get resource usage statistics
    pub fn get_resource_stats(&self) -> ResourceStats {
        ResourceStats {
            buffer_pool_stats: self.buffer_pool.stats(),
            string_interner_stats: self.string_interner.stats(),
            limits: self.limits.clone(),
        }
    }
}

/// Combined resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    /// Buffer pool statistics
    pub buffer_pool_stats: PoolStats,
    /// String interner statistics
    pub string_interner_stats: InternerStats,
    /// Resource limits
    pub limits: ResourceLimits,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_creation() {
        let config = MemoryPoolConfig {
            initial_size: 10,
            max_size: 100,
            cleanup_interval_secs: 30,
            max_unused_age_secs: 120,
        };
        
        let pool: MemoryPool<String> = MemoryPool::new(config);
        let stats = pool.stats();
        
        assert_eq!(stats.total_created, 10);
        assert_eq!(stats.objects_in_pool, 10);
        assert_eq!(stats.objects_borrowed, 0);
    }

    #[test]
    fn test_memory_pool_borrow_return() {
        let pool: MemoryPool<String> = MemoryPool::new(MemoryPoolConfig::default());
        
        {
            let mut borrowed = pool.borrow();
            *borrowed.get_mut() = "test".to_string();
            
            let stats = pool.stats();
            assert_eq!(stats.objects_borrowed, 1);
            assert_eq!(stats.pool_hits, 1);
        }
        
        // Object should be returned to pool when dropped
        let stats = pool.stats();
        assert_eq!(stats.objects_borrowed, 0);
        assert_eq!(stats.objects_in_pool, 1000); // Should be back in pool
    }

    #[test]
    fn test_memory_pool_stats() {
        let pool: MemoryPool<i32> = MemoryPool::new(MemoryPoolConfig::default());
        
        // Borrow more objects than initial size
        let mut borrowed = Vec::new();
        for _ in 0..1200 {
            borrowed.push(pool.borrow());
        }
        
        let stats = pool.stats();
        assert_eq!(stats.objects_borrowed, 1200);
        assert_eq!(stats.pool_hits, 1000); // Initial pool size
        assert_eq!(stats.pool_misses, 200); // New allocations
        assert!(stats.hit_rate() > 80.0); // Should have good hit rate
    }

    #[test]
    fn test_string_interner() {
        let interner = StringInterner::new();
        
        let str1 = interner.intern("test_string");
        let str2 = interner.intern("test_string");
        let str3 = interner.intern("different_string");
        
        // Same string should return same Arc
        assert!(Arc::ptr_eq(&str1, &str2));
        assert!(!Arc::ptr_eq(&str1, &str3));
        
        let stats = interner.stats();
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.unique_strings, 2);
        assert!(stats.memory_saved_bytes > 0);
    }

    #[test]
    fn test_string_interner_stats() {
        let interner = StringInterner::new();
        
        // Intern the same string multiple times
        for _ in 0..10 {
            interner.intern("repeated_string");
        }
        
        let stats = interner.stats();
        assert_eq!(stats.total_requests, 10);
        assert_eq!(stats.cache_hits, 9);
        assert_eq!(stats.unique_strings, 1);
        
        // Memory saved should be 9 * string_length
        assert_eq!(stats.memory_saved_bytes, 9 * "repeated_string".len());
    }

    #[test]
    fn test_resource_manager_creation() {
        let limits = ResourceLimits {
            max_memory_bytes: 50 * 1024 * 1024,
            max_sessions: 5000,
            max_api_calls_per_session: 500,
        };
        
        let manager = ResourceManager::new(limits.clone()).unwrap();
        let stats = manager.get_resource_stats();
        
        assert_eq!(stats.limits.max_memory_bytes, limits.max_memory_bytes);
        assert_eq!(stats.limits.max_sessions, limits.max_sessions);
    }

    #[test]
    fn test_resource_manager_buffer_pool() {
        let manager = ResourceManager::new(ResourceLimits::default()).unwrap();
        
        {
            let mut buffer = manager.borrow_string_buffer();
            buffer.get_mut().push_str("test content");
            assert!(buffer.get().contains("test content"));
        }
        
        let stats = manager.get_resource_stats();
        assert!(stats.buffer_pool_stats.pool_hits > 0 || stats.buffer_pool_stats.pool_misses > 0);
    }

    #[test]
    fn test_resource_manager_string_interning() {
        let manager = ResourceManager::new(ResourceLimits::default()).unwrap();
        
        let endpoint1 = manager.intern_string("https://api.anthropic.com/v1/messages");
        let endpoint2 = manager.intern_string("https://api.anthropic.com/v1/messages");
        let model = manager.intern_string("claude-3-sonnet");
        
        assert!(Arc::ptr_eq(&endpoint1, &endpoint2));
        assert!(!Arc::ptr_eq(&endpoint1, &model));
        
        let stats = manager.get_resource_stats();
        assert_eq!(stats.string_interner_stats.unique_strings, 2);
        assert_eq!(stats.string_interner_stats.cache_hits, 1);
    }

    #[test]
    fn test_memory_pool_cleanup() {
        let config = MemoryPoolConfig {
            initial_size: 5,
            max_size: 10,
            cleanup_interval_secs: 1,
            max_unused_age_secs: 1, // 1 second for testing
        };
        
        let pool: MemoryPool<String> = MemoryPool::new(config);
        
        // Wait for objects to age
        std::thread::sleep(Duration::from_secs(2));
        
        // Cleanup should remove aged objects
        pool.cleanup();
        
        let stats = pool.stats();
        assert!(stats.objects_cleaned > 0);
    }

    #[test]
    fn test_pool_stats_hit_rate() {
        let stats = PoolStats {
            total_created: 100,
            objects_in_pool: 50,
            objects_borrowed: 25,
            total_reuses: 200,
            pool_hits: 80,
            pool_misses: 20,
            objects_cleaned: 5,
        };
        
        assert_eq!(stats.hit_rate(), 80.0); // 80/(80+20) * 100
    }

    #[test]
    fn test_pool_stats_hit_rate_zero_division() {
        let stats = PoolStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
    }
}