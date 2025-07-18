use super::filesystem::{Issue, IssueStorage};
use super::cache::{IssueCache, CacheStats};
use crate::error::Result;
use async_trait::async_trait;
use tokio::time::Duration;
use std::sync::Arc;

pub struct CachedIssueStorage {
    storage: Box<dyn IssueStorage>,
    cache: Arc<IssueCache>,
}

impl CachedIssueStorage {
    pub fn new(storage: Box<dyn IssueStorage>) -> Self {
        let cache = Arc::new(IssueCache::new(
            Duration::from_secs(300), // 5 minutes TTL
            1000, // Max 1000 issues in cache
        ));
        
        Self { storage, cache }
    }
    
    pub fn with_cache_config(
        storage: Box<dyn IssueStorage>,
        ttl: Duration,
        max_size: usize,
    ) -> Self {
        let cache = Arc::new(IssueCache::new(ttl, max_size));
        Self { storage, cache }
    }
    
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
    
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
    
    pub fn reset_cache_stats(&self) {
        self.cache.reset_stats();
    }
}

#[async_trait]
impl IssueStorage for CachedIssueStorage {
    async fn create_issue(&self, name: String, content: String) -> Result<Issue> {
        let issue = self.storage.create_issue(name, content).await?;
        
        // Cache the new issue
        self.cache.put(issue.clone());
        
        Ok(issue)
    }
    
    async fn get_issue(&self, number: u32) -> Result<Issue> {
        // Try cache first
        if let Some(issue) = self.cache.get(number) {
            return Ok(issue);
        }
        
        // Cache miss, fetch from storage
        let issue = self.storage.get_issue(number).await?;
        
        // Cache the result
        self.cache.put(issue.clone());
        
        Ok(issue)
    }
    
    async fn update_issue(&self, number: u32, content: String) -> Result<Issue> {
        let issue = self.storage.update_issue(number, content).await?;
        
        // Update cache
        self.cache.put(issue.clone());
        
        Ok(issue)
    }
    
    async fn mark_complete(&self, number: u32) -> Result<Issue> {
        let issue = self.storage.mark_complete(number).await?;
        
        // Update cache
        self.cache.put(issue.clone());
        
        Ok(issue)
    }
    
    async fn list_issues(&self) -> Result<Vec<Issue>> {
        // For list operations, we typically don't cache the entire list
        // but we can cache individual issues from the list
        let issues = self.storage.list_issues().await?;
        
        // Cache individual issues
        for issue in &issues {
            self.cache.put(issue.clone());
        }
        
        Ok(issues)
    }

    async fn create_issues_batch(&self, issues: Vec<(String, String)>) -> Result<Vec<Issue>> {
        let created_issues = self.storage.create_issues_batch(issues).await?;
        
        // Cache all created issues
        for issue in &created_issues {
            self.cache.put(issue.clone());
        }
        
        Ok(created_issues)
    }

    async fn get_issues_batch(&self, numbers: Vec<u32>) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let mut cache_misses = Vec::new();
        
        // Try to get issues from cache first
        for number in numbers {
            if let Some(issue) = self.cache.get(number) {
                issues.push(issue);
            } else {
                cache_misses.push(number);
            }
        }
        
        // Fetch missing issues from storage
        if !cache_misses.is_empty() {
            let missing_issues = self.storage.get_issues_batch(cache_misses).await?;
            
            // Cache the missing issues
            for issue in &missing_issues {
                self.cache.put(issue.clone());
            }
            
            issues.extend(missing_issues);
        }
        
        Ok(issues)
    }

    async fn update_issues_batch(&self, updates: Vec<(u32, String)>) -> Result<Vec<Issue>> {
        let updated_issues = self.storage.update_issues_batch(updates).await?;
        
        // Update cache with new versions
        for issue in &updated_issues {
            self.cache.put(issue.clone());
        }
        
        Ok(updated_issues)
    }

    async fn mark_complete_batch(&self, numbers: Vec<u32>) -> Result<Vec<Issue>> {
        let completed_issues = self.storage.mark_complete_batch(numbers).await?;
        
        // Update cache with completed versions
        for issue in &completed_issues {
            self.cache.put(issue.clone());
        }
        
        Ok(completed_issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issues::filesystem::FileSystemIssueStorage;
    use tempfile::TempDir;
    use tokio::time::Duration;

    fn create_test_storage() -> (CachedIssueStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().join("issues");
        
        let fs_storage = FileSystemIssueStorage::new(issues_dir).unwrap();
        let cached_storage = CachedIssueStorage::new(Box::new(fs_storage));
        
        (cached_storage, temp_dir)
    }

    #[tokio::test]
    async fn test_cached_storage_creation() {
        let (storage, _temp) = create_test_storage();
        
        // Check initial cache stats
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 0);
        assert_eq!(stats.max_size, 1000);
    }

    #[tokio::test]
    async fn test_cached_storage_with_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().join("issues");
        
        let fs_storage = FileSystemIssueStorage::new(issues_dir).unwrap();
        let cached_storage = CachedIssueStorage::with_cache_config(
            Box::new(fs_storage),
            Duration::from_secs(60), // 1 minute TTL
            100, // Max 100 issues
        );
        
        let stats = cached_storage.cache_stats();
        assert_eq!(stats.max_size, 100);
    }

    #[tokio::test]
    async fn test_create_issue_caches_result() {
        let (storage, _temp) = create_test_storage();
        
        // Create an issue
        let issue = storage.create_issue("test_issue".to_string(), "Test content".to_string()).await.unwrap();
        
        // Check that it was cached
        let stats = storage.cache_stats();
        assert_eq!(stats.size, 1);
        
        // Verify we can get it from cache
        let cached_issue = storage.get_issue(issue.number.value()).await.unwrap();
        assert_eq!(cached_issue.number, issue.number);
        assert_eq!(cached_issue.content, issue.content);
        
        // Should be a cache hit
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_get_issue_cache_miss_then_hit() {
        let (_storage, _temp) = create_test_storage();
        
        // Create an issue directly in the underlying storage
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().join("issues");
        let fs_storage = FileSystemIssueStorage::new(issues_dir).unwrap();
        let issue = fs_storage.create_issue("test_issue".to_string(), "Test content".to_string()).await.unwrap();
        
        // Create a new cached storage using the same underlying storage
        let cached_storage = CachedIssueStorage::new(Box::new(fs_storage));
        
        // First get - should be cache miss
        let retrieved_issue = cached_storage.get_issue(issue.number.value()).await.unwrap();
        assert_eq!(retrieved_issue.number, issue.number);
        
        let stats = cached_storage.cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 1);
        
        // Second get - should be cache hit
        let cached_issue = cached_storage.get_issue(issue.number.value()).await.unwrap();
        assert_eq!(cached_issue.number, issue.number);
        
        let stats = cached_storage.cache_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_update_issue_updates_cache() {
        let (storage, _temp) = create_test_storage();
        
        // Create an issue
        let issue = storage.create_issue("test_issue".to_string(), "Original content".to_string()).await.unwrap();
        
        // Update the issue
        let _updated_issue = storage.update_issue(issue.number.value(), "Updated content".to_string()).await.unwrap();
        
        // Get from cache - should have updated content
        let cached_issue = storage.get_issue(issue.number.value()).await.unwrap();
        assert_eq!(cached_issue.content, "Updated content");
        
        // Should be cache hits
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_mark_complete_updates_cache() {
        let (storage, _temp) = create_test_storage();
        
        // Create an issue
        let issue = storage.create_issue("test_issue".to_string(), "Test content".to_string()).await.unwrap();
        assert!(!issue.completed);
        
        // Mark as complete
        let completed_issue = storage.mark_complete(issue.number.value()).await.unwrap();
        assert!(completed_issue.completed);
        
        // Get from cache - should show as completed
        let cached_issue = storage.get_issue(issue.number.value()).await.unwrap();
        assert!(cached_issue.completed);
        
        // Should be cache hits
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_list_issues_caches_individual_issues() {
        let (storage, _temp) = create_test_storage();
        
        // Create multiple issues
        let issue1 = storage.create_issue("issue1".to_string(), "Content 1".to_string()).await.unwrap();
        let issue2 = storage.create_issue("issue2".to_string(), "Content 2".to_string()).await.unwrap();
        
        // Clear cache to start fresh
        storage.clear_cache();
        
        // List issues - should cache individual issues
        let issues = storage.list_issues().await.unwrap();
        assert_eq!(issues.len(), 2);
        
        // Check that individual issues are now cached
        let stats = storage.cache_stats();
        assert_eq!(stats.size, 2);
        
        // Getting individual issues should be cache hits
        let cached_issue1 = storage.get_issue(issue1.number.value()).await.unwrap();
        let cached_issue2 = storage.get_issue(issue2.number.value()).await.unwrap();
        
        assert_eq!(cached_issue1.number, issue1.number);
        assert_eq!(cached_issue2.number, issue2.number);
        
        // Should be cache hits
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let (storage, _temp) = create_test_storage();
        
        // Create and cache an issue
        let issue = storage.create_issue("test_issue".to_string(), "Test content".to_string()).await.unwrap();
        
        // Verify it's cached
        let stats = storage.cache_stats();
        assert_eq!(stats.size, 1);
        
        // Clear cache
        storage.clear_cache();
        
        // Verify cache is empty
        let stats = storage.cache_stats();
        assert_eq!(stats.size, 0);
        
        // Getting the issue should be a cache miss
        let retrieved_issue = storage.get_issue(issue.number.value()).await.unwrap();
        assert_eq!(retrieved_issue.number, issue.number);
        
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_cache_performance_improvement() {
        let (storage, _temp) = create_test_storage();
        
        // Create an issue
        let issue = storage.create_issue("test_issue".to_string(), "Test content".to_string()).await.unwrap();
        
        // Clear cache and reset stats to start fresh
        storage.clear_cache();
        storage.reset_cache_stats();
        
        // First get should be a miss (loads from storage and caches)
        let _first_get = storage.get_issue(issue.number.value()).await.unwrap();
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
        
        // Subsequent gets should be hits
        for _ in 0..9 {
            storage.get_issue(issue.number.value()).await.unwrap();
        }
        
        // Verify cache stats - 9 hits, 1 miss total
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 9);
        assert_eq!(stats.misses, 1);
        
        // Clear cache and reset stats
        storage.clear_cache();
        storage.reset_cache_stats();
        
        // First get after clear should be miss again
        let _first_get = storage.get_issue(issue.number.value()).await.unwrap();
        let stats = storage.cache_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }
}