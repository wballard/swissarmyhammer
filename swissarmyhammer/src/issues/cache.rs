use super::filesystem::Issue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, Instant};

/// An entry in the issue cache containing the issue data and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached issue data
    pub issue: Issue,
    /// When this entry was created in the cache
    pub created_at: Instant,
    /// When this entry was last accessed
    pub last_access: Instant,
    /// How many times this entry has been accessed
    pub access_count: u64,
}

/// In-memory cache for issue data with TTL and LRU eviction
pub struct IssueCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
    max_size: usize,
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl IssueCache {
    /// Create a new issue cache with specified TTL and maximum size
    pub fn new(ttl: Duration, max_size: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_size,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Get an issue from the cache by name, updating access statistics
    pub fn get(&self, issue_name: &str) -> Option<Issue> {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();

        if let Some(entry) = entries.get_mut(issue_name) {
            // Check if entry is still valid (TTL based on creation time)
            if now.duration_since(entry.created_at) < self.ttl {
                entry.access_count += 1;
                entry.last_access = now; // Update access time for LRU

                *self.hits.write().unwrap() += 1;
                return Some(entry.issue.clone());
            }
            // Entry expired, remove it
            entries.remove(issue_name);
        }

        *self.misses.write().unwrap() += 1;
        None
    }

    /// Put an issue into the cache, evicting old entries if necessary
    pub fn put(&self, issue: Issue) {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();

        // Check if we need to evict entries
        if entries.len() >= self.max_size {
            self.evict_lru(&mut entries);
        }

        let issue_name = issue.name.clone();
        entries.insert(
            issue_name,
            CacheEntry {
                issue,
                created_at: now,
                last_access: now,
                access_count: 1,
            },
        );
    }

    /// Remove a specific issue from the cache
    pub fn invalidate(&self, issue_name: &str) {
        let mut entries = self.entries.write().unwrap();
        entries.remove(issue_name);
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }

    /// Reset cache hit/miss statistics
    pub fn reset_stats(&self) {
        *self.hits.write().unwrap() = 0;
        *self.misses.write().unwrap() = 0;
    }

    /// Get current cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.read().unwrap();
        let misses = *self.misses.read().unwrap();
        let total = hits + misses;

        CacheStats {
            hits,
            misses,
            hit_rate: if total > 0 {
                hits as f64 / total as f64
            } else {
                0.0
            },
            size: self.entries.read().unwrap().len(),
            max_size: self.max_size,
        }
    }

    fn evict_lru(&self, entries: &mut HashMap<String, CacheEntry>) {
        if entries.is_empty() {
            return;
        }

        // Find the least recently used entry
        let lru_key = entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(key, _)| key.clone())
            .unwrap();

        entries.remove(&lru_key);
    }
}

/// Statistics about cache performance
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Cache hit rate (hits / total requests)
    pub hit_rate: f64,
    /// Current number of entries in cache
    pub size: usize,
    /// Maximum number of entries the cache can hold
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_issue(name: &str) -> Issue {
        Issue {
            number: 1, // Default test number
            name: name.to_string(),
            content: format!("Test content for issue {name}"),
            completed: false,
            file_path: PathBuf::from(format!("{name}.md")),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_cache_creation() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let stats = cache.stats();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 0.0);
        assert_eq!(stats.size, 0);
        assert_eq!(stats.max_size, 1000);
    }

    #[test]
    fn test_cache_put_and_get() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue = create_test_issue("test_issue");

        // Put issue in cache
        cache.put(issue.clone());

        // Get issue from cache
        let cached_issue = cache.get("test_issue").unwrap();
        assert_eq!(cached_issue.name, issue.name);
        assert_eq!(cached_issue.content, issue.content);

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 1.0);
        assert_eq!(stats.size, 1);
    }

    #[test]
    fn test_cache_miss() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);

        // Try to get non-existent issue
        let result = cache.get("nonexistent_issue");
        assert!(result.is_none());

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_cache_ttl_expiration() {
        let cache = IssueCache::new(Duration::from_millis(10), 1000);
        let issue = create_test_issue("test_issue");

        // Put issue in cache
        cache.put(issue.clone());

        // Immediately get - should hit
        let cached_issue = cache.get("test_issue");
        assert!(cached_issue.is_some());

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(20));

        // Get again - should miss due to expiration
        let expired_result = cache.get("test_issue");
        assert!(expired_result.is_none());

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 0); // Entry should be removed
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = IssueCache::new(Duration::from_secs(300), 2); // Max 2 entries
        let issue1 = create_test_issue("issue1");
        let issue2 = create_test_issue("issue2");
        let issue3 = create_test_issue("issue3");

        // Put first two issues
        cache.put(issue1.clone());
        cache.put(issue2.clone());

        // Access issue1 to make it more recently used
        cache.get("issue1");

        // Put third issue - should evict issue2 (LRU)
        cache.put(issue3.clone());

        // Issue1 should still be there
        assert!(cache.get("issue1").is_some());

        // Issue2 should be evicted
        assert!(cache.get("issue2").is_none());

        // Issue3 should be there
        assert!(cache.get("issue3").is_some());

        // Check size
        let stats = cache.stats();
        assert_eq!(stats.size, 2);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue = create_test_issue("test_issue");

        // Put issue in cache
        cache.put(issue.clone());
        assert!(cache.get("test_issue").is_some());

        // Invalidate the issue
        cache.invalidate("test_issue");
        assert!(cache.get("test_issue").is_none());

        // Check size
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue1 = create_test_issue("issue1");
        let issue2 = create_test_issue("issue2");

        // Put issues in cache
        cache.put(issue1.clone());
        cache.put(issue2.clone());

        // Clear cache
        cache.clear();

        // All issues should be gone
        assert!(cache.get("issue1").is_none());
        assert!(cache.get("issue2").is_none());

        // Check size
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_access_count_tracking() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue = create_test_issue("test_issue");

        // Put issue in cache
        cache.put(issue.clone());

        // Access multiple times
        cache.get("test_issue");
        cache.get("test_issue");
        cache.get("test_issue");

        // Check that access count is tracked
        let entries = cache.entries.read().unwrap();
        let entry = entries.get("test_issue").unwrap();
        assert_eq!(entry.access_count, 4); // 1 from put + 3 from gets
    }

    #[test]
    fn test_cache_stats_calculation() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue = create_test_issue("test_issue");

        // Put issue in cache
        cache.put(issue.clone());

        // Mix of hits and misses
        cache.get("test_issue"); // hit
        cache.get("test_issue"); // hit
        cache.get("issue2"); // miss
        cache.get("issue3"); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.hit_rate, 0.5);
        assert_eq!(stats.size, 1);
    }

    #[test]
    fn test_cache_concurrent_access() {
        let cache = Arc::new(IssueCache::new(Duration::from_secs(300), 1000));
        let issue = create_test_issue("test_issue");

        // Put issue in cache
        cache.put(issue.clone());

        // Test concurrent access
        let mut handles = vec![];
        for _ in 0..10 {
            let cache_clone = cache.clone();
            let handle = std::thread::spawn(move || cache_clone.get("test_issue"));
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_some());
        }

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 10);
        assert_eq!(stats.misses, 0);
    }
}
