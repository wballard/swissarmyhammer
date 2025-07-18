use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{Duration, Instant};
use super::filesystem::Issue;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub issue: Issue,
    pub timestamp: Instant,
    pub access_count: u64,
}

/// In-memory cache for issue data with TTL and LRU eviction
pub struct IssueCache {
    entries: Arc<RwLock<HashMap<u32, CacheEntry>>>,
    ttl: Duration,
    max_size: usize,
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl IssueCache {
    pub fn new(ttl: Duration, max_size: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_size,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }
    
    pub fn get(&self, issue_number: u32) -> Option<Issue> {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();
        
        if let Some(entry) = entries.get_mut(&issue_number) {
            // Check if entry is still valid
            if now.duration_since(entry.timestamp) < self.ttl {
                entry.access_count += 1;
                entry.timestamp = now; // Update access time for LRU
                
                *self.hits.write().unwrap() += 1;
                return Some(entry.issue.clone());
            } else {
                // Entry expired, remove it
                entries.remove(&issue_number);
            }
        }
        
        *self.misses.write().unwrap() += 1;
        None
    }
    
    pub fn put(&self, issue: Issue) {
        let now = Instant::now();
        let mut entries = self.entries.write().unwrap();
        
        // Check if we need to evict entries
        if entries.len() >= self.max_size {
            self.evict_lru(&mut entries);
        }
        
        entries.insert(issue.number.value(), CacheEntry {
            issue,
            timestamp: now,
            access_count: 1,
        });
    }
    
    pub fn invalidate(&self, issue_number: u32) {
        let mut entries = self.entries.write().unwrap();
        entries.remove(&issue_number);
    }
    
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();
    }
    
    pub fn reset_stats(&self) {
        *self.hits.write().unwrap() = 0;
        *self.misses.write().unwrap() = 0;
    }
    
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.read().unwrap();
        let misses = *self.misses.read().unwrap();
        let total = hits + misses;
        
        CacheStats {
            hits,
            misses,
            hit_rate: if total > 0 { hits as f64 / total as f64 } else { 0.0 },
            size: self.entries.read().unwrap().len(),
            max_size: self.max_size,
        }
    }
    
    fn evict_lru(&self, entries: &mut HashMap<u32, CacheEntry>) {
        if entries.is_empty() {
            return;
        }
        
        // Find the least recently used entry
        let lru_key = entries.iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(key, _)| *key)
            .unwrap();
        
        entries.remove(&lru_key);
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issues::IssueNumber;
    use std::path::PathBuf;
    use chrono::Utc;

    fn create_test_issue(number: u32, name: &str) -> Issue {
        Issue {
            number: IssueNumber::from(number),
            name: name.to_string(),
            content: format!("Test content for issue {}", number),
            completed: false,
            file_path: PathBuf::from(format!("test_{}.md", number)),
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
        let issue = create_test_issue(1, "test_issue");
        
        // Put issue in cache
        cache.put(issue.clone());
        
        // Get issue from cache
        let cached_issue = cache.get(1).unwrap();
        assert_eq!(cached_issue.number, issue.number);
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
        let result = cache.get(999);
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
        let issue = create_test_issue(1, "test_issue");
        
        // Put issue in cache
        cache.put(issue.clone());
        
        // Immediately get - should hit
        let cached_issue = cache.get(1);
        assert!(cached_issue.is_some());
        
        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(20));
        
        // Get again - should miss due to expiration
        let expired_result = cache.get(1);
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
        let issue1 = create_test_issue(1, "issue1");
        let issue2 = create_test_issue(2, "issue2");
        let issue3 = create_test_issue(3, "issue3");
        
        // Put first two issues
        cache.put(issue1.clone());
        cache.put(issue2.clone());
        
        // Access issue1 to make it more recently used
        cache.get(1);
        
        // Put third issue - should evict issue2 (LRU)
        cache.put(issue3.clone());
        
        // Issue1 should still be there
        assert!(cache.get(1).is_some());
        
        // Issue2 should be evicted
        assert!(cache.get(2).is_none());
        
        // Issue3 should be there
        assert!(cache.get(3).is_some());
        
        // Check size
        let stats = cache.stats();
        assert_eq!(stats.size, 2);
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue = create_test_issue(1, "test_issue");
        
        // Put issue in cache
        cache.put(issue.clone());
        assert!(cache.get(1).is_some());
        
        // Invalidate the issue
        cache.invalidate(1);
        assert!(cache.get(1).is_none());
        
        // Check size
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue1 = create_test_issue(1, "issue1");
        let issue2 = create_test_issue(2, "issue2");
        
        // Put issues in cache
        cache.put(issue1.clone());
        cache.put(issue2.clone());
        
        // Clear cache
        cache.clear();
        
        // All issues should be gone
        assert!(cache.get(1).is_none());
        assert!(cache.get(2).is_none());
        
        // Check size
        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_access_count_tracking() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue = create_test_issue(1, "test_issue");
        
        // Put issue in cache
        cache.put(issue.clone());
        
        // Access multiple times
        cache.get(1);
        cache.get(1);
        cache.get(1);
        
        // Check that access count is tracked
        let entries = cache.entries.read().unwrap();
        let entry = entries.get(&1).unwrap();
        assert_eq!(entry.access_count, 4); // 1 from put + 3 from gets
    }

    #[test]
    fn test_cache_stats_calculation() {
        let cache = IssueCache::new(Duration::from_secs(300), 1000);
        let issue = create_test_issue(1, "test_issue");
        
        // Put issue in cache
        cache.put(issue.clone());
        
        // Mix of hits and misses
        cache.get(1); // hit
        cache.get(1); // hit
        cache.get(2); // miss
        cache.get(3); // miss
        
        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.hit_rate, 0.5);
        assert_eq!(stats.size, 1);
    }

    #[test]
    fn test_cache_concurrent_access() {
        let cache = Arc::new(IssueCache::new(Duration::from_secs(300), 1000));
        let issue = create_test_issue(1, "test_issue");
        
        // Put issue in cache
        cache.put(issue.clone());
        
        // Test concurrent access
        let mut handles = vec![];
        for _ in 0..10 {
            let cache_clone = cache.clone();
            let handle = std::thread::spawn(move || {
                cache_clone.get(1)
            });
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