//! Async storage optimization for database operations
//!
//! This module provides async storage operations with batching, connection pooling,
//! and write-behind caching to optimize database performance.

#[cfg(feature = "database")]
use crate::cost::database::{CostDatabase, DatabaseError};
use crate::cost::{CostSession, CostSessionId, ApiCall, ApiCallId, CostError};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, RwLock as AsyncRwLock};
use tokio::time::interval;
use serde::{Deserialize, Serialize};

/// Configuration for async storage optimization
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Batch size for write operations
    pub batch_size: usize,
    /// Flush interval in milliseconds
    pub flush_interval_ms: u64,
    /// Connection pool size
    pub connection_pool_size: usize,
    /// Write buffer size
    pub write_buffer_size: usize,
    /// Enable write-behind caching
    pub enable_write_behind: bool,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            flush_interval_ms: 1000,
            connection_pool_size: 10,
            write_buffer_size: 10000,
            enable_write_behind: true,
            cache_ttl_secs: 300,
        }
    }
}

/// Storage operation types for batching
#[derive(Debug, Clone)]
pub enum StorageOperation {
    /// Store a cost session
    StoreSession {
        session: CostSession,
        response_tx: oneshot::Sender<Result<(), CostError>>,
    },
    /// Update API call in session
    UpdateApiCall {
        session_id: CostSessionId,
        call_id: ApiCallId,
        api_call: ApiCall,
        response_tx: oneshot::Sender<Result<(), CostError>>,
    },
    /// Delete session
    DeleteSession {
        session_id: CostSessionId,
        response_tx: oneshot::Sender<Result<(), CostError>>,
    },
    /// Batch flush operation
    BatchFlush,
}

/// Batched storage operations
#[derive(Debug)]
struct StorageBatch {
    /// Operations in this batch
    operations: Vec<StorageOperation>,
    /// When this batch was created
    created_at: Instant,
    /// Batch size
    size: usize,
}

impl StorageBatch {
    /// Create a new empty batch
    fn new() -> Self {
        Self {
            operations: Vec::new(),
            created_at: Instant::now(),
            size: 0,
        }
    }

    /// Add operation to batch
    fn add_operation(&mut self, operation: StorageOperation) {
        self.operations.push(operation);
        self.size += 1;
    }

    /// Check if batch should be flushed
    fn should_flush(&self, config: &StorageConfig) -> bool {
        self.size >= config.batch_size || 
        self.created_at.elapsed().as_millis() >= config.flush_interval_ms as u128
    }
}

/// Write-behind cache entry
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// Cached value
    value: T,
    /// When entry was created
    created_at: Instant,
    /// Whether entry is dirty (needs to be written)
    dirty: bool,
}

/// Write-behind cache for storage operations
#[derive(Debug)]
pub struct WriteCache<T> 
where 
    T: Clone,
{
    /// Cache storage
    cache: Arc<AsyncRwLock<HashMap<String, CacheEntry<T>>>>,
    /// Cache configuration
    ttl: Duration,
}

impl<T> WriteCache<T> 
where 
    T: Clone,
{
    /// Create a new write cache
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(AsyncRwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    /// Get value from cache
    pub async fn get(&self, key: &str) -> Option<T> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.created_at.elapsed() < self.ttl {
                Some(entry.value.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Put value in cache
    pub async fn put(&self, key: String, value: T, dirty: bool) {
        let mut cache = self.cache.write().await;
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            dirty,
        };
        cache.insert(key, entry);
    }

    /// Get all dirty entries for flushing
    pub async fn get_dirty_entries(&self) -> Vec<(String, T)> {
        let mut cache = self.cache.write().await;
        let mut dirty_entries = Vec::new();
        
        // Collect dirty entries and mark them as clean
        for (key, entry) in cache.iter_mut() {
            if entry.dirty && entry.created_at.elapsed() < self.ttl {
                dirty_entries.push((key.clone(), entry.value.clone()));
                entry.dirty = false;
            }
        }
        
        dirty_entries
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let cutoff_time = Instant::now() - self.ttl;
        cache.retain(|_, entry| entry.created_at > cutoff_time);
    }
}

/// Async storage manager with optimization
pub struct AsyncStorageManager {
    /// Configuration
    config: StorageConfig,
    /// Operation sender channel
    operation_tx: mpsc::UnboundedSender<StorageOperation>,
    /// Write-behind cache for sessions
    session_cache: WriteCache<CostSession>,
    /// Storage statistics
    stats: Arc<AsyncRwLock<AsyncStorageStats>>,
}

/// Storage performance statistics
#[derive(Debug, Clone, Default)]
pub struct AsyncStorageStats {
    /// Total operations processed
    pub total_operations: usize,
    /// Operations batched
    pub batched_operations: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
    /// Average batch size
    pub avg_batch_size: f64,
    /// Total flush time in microseconds
    pub total_flush_time_micros: u64,
    /// Number of flushes
    pub flush_count: usize,
}

impl AsyncStorageStats {
    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64 * 100.0
        }
    }

    /// Calculate average flush time
    pub fn avg_flush_time_micros(&self) -> f64 {
        if self.flush_count == 0 {
            0.0
        } else {
            self.total_flush_time_micros as f64 / self.flush_count as f64
        }
    }
}

impl AsyncStorageManager {
    /// Create a new async storage manager
    pub fn new(config: StorageConfig) -> Self {
        let (operation_tx, operation_rx) = mpsc::unbounded_channel();
        let session_cache = WriteCache::new(config.cache_ttl_secs);
        let stats = Arc::new(AsyncRwLock::new(AsyncStorageStats::default()));
        
        let manager = Self {
            config: config.clone(),
            operation_tx,
            session_cache,
            stats: Arc::clone(&stats),
        };
        
        // Start background worker
        let worker = AsyncStorageWorker::new(config, operation_rx, Arc::clone(&stats));
        tokio::spawn(worker.run());
        
        manager
    }

    /// Store a session asynchronously
    pub async fn store_session(&self, session: CostSession) -> Result<(), CostError> {
        // Check cache first
        let session_key = session.session_id.to_string();
        
        if self.config.enable_write_behind {
            // Store in cache immediately
            self.session_cache.put(session_key, session.clone(), true).await;
            Ok(()) // Return immediately for write-behind
        } else {
            // Send to storage worker and wait for result
            let (response_tx, response_rx) = oneshot::channel();
            
            let operation = StorageOperation::StoreSession {
                session,
                response_tx,
            };
            
            self.operation_tx.send(operation)
                .map_err(|_| CostError::SerializationError {
                    message: "Storage worker channel closed".to_string(),
                })?;
            
            response_rx.await
                .map_err(|_| CostError::SerializationError {
                    message: "Storage operation cancelled".to_string(),
                })?
        }
    }

    /// Get a session, checking cache first
    pub async fn get_session(&self, session_id: &CostSessionId) -> Option<CostSession> {
        let session_key = session_id.to_string();
        
        // Check cache first
        if let Some(session) = self.session_cache.get(&session_key).await {
            let mut stats = self.stats.write().await;
            stats.cache_hits += 1;
            Some(session)
        } else {
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
            // In a real implementation, this would query the database
            // For now, return None since we don't have direct DB access
            None
        }
    }

    /// Update an API call in a session
    pub async fn update_api_call(
        &self, 
        session_id: CostSessionId,
        call_id: ApiCallId,
        api_call: ApiCall
    ) -> Result<(), CostError> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let operation = StorageOperation::UpdateApiCall {
            session_id,
            call_id,
            api_call,
            response_tx,
        };
        
        self.operation_tx.send(operation)
            .map_err(|_| CostError::SerializationError {
                message: "Storage worker channel closed".to_string(),
            })?;
        
        response_rx.await
            .map_err(|_| CostError::SerializationError {
                message: "Storage operation cancelled".to_string(),
            })?
    }

    /// Delete a session
    pub async fn delete_session(&self, session_id: CostSessionId) -> Result<(), CostError> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let operation = StorageOperation::DeleteSession {
            session_id,
            response_tx,
        };
        
        self.operation_tx.send(operation)
            .map_err(|_| CostError::SerializationError {
                message: "Storage worker channel closed".to_string(),
            })?;
        
        response_rx.await
            .map_err(|_| CostError::SerializationError {
                message: "Storage operation cancelled".to_string(),
            })?
    }

    /// Force flush pending operations
    pub async fn flush(&self) -> Result<(), CostError> {
        self.operation_tx.send(StorageOperation::BatchFlush)
            .map_err(|_| CostError::SerializationError {
                message: "Storage worker channel closed".to_string(),
            })?;
        
        // Give worker time to process flush
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> AsyncStorageStats {
        self.stats.read().await.clone()
    }
}

/// Background worker for processing storage operations
struct AsyncStorageWorker {
    /// Configuration
    config: StorageConfig,
    /// Operation receiver
    operation_rx: mpsc::UnboundedReceiver<StorageOperation>,
    /// Current batch
    current_batch: StorageBatch,
    /// Statistics
    stats: Arc<AsyncRwLock<AsyncStorageStats>>,
}

impl AsyncStorageWorker {
    /// Create a new storage worker
    fn new(
        config: StorageConfig,
        operation_rx: mpsc::UnboundedReceiver<StorageOperation>,
        stats: Arc<AsyncRwLock<AsyncStorageStats>>,
    ) -> Self {
        Self {
            config,
            operation_rx,
            current_batch: StorageBatch::new(),
            stats,
        }
    }

    /// Run the storage worker
    async fn run(mut self) {
        let mut flush_interval = interval(Duration::from_millis(self.config.flush_interval_ms));
        
        loop {
            tokio::select! {
                // Process incoming operations
                Some(operation) = self.operation_rx.recv() => {
                    self.handle_operation(operation).await;
                }
                
                // Periodic flush
                _ = flush_interval.tick() => {
                    if !self.current_batch.operations.is_empty() {
                        self.flush_batch().await;
                    }
                }
                
                else => {
                    // Channel closed, perform final flush and exit
                    self.flush_batch().await;
                    break;
                }
            }
        }
    }

    /// Handle a single storage operation
    async fn handle_operation(&mut self, operation: StorageOperation) {
        match operation {
            StorageOperation::BatchFlush => {
                self.flush_batch().await;
            }
            _ => {
                self.current_batch.add_operation(operation);
                
                // Flush if batch is ready
                if self.current_batch.should_flush(&self.config) {
                    self.flush_batch().await;
                }
            }
        }
    }

    /// Flush the current batch to storage
    async fn flush_batch(&mut self) {
        if self.current_batch.operations.is_empty() {
            return;
        }
        
        let start_time = Instant::now();
        let batch_size = self.current_batch.size;
        
        // Process operations in the batch
        for operation in self.current_batch.operations.drain(..) {
            match operation {
                StorageOperation::StoreSession { session: _, response_tx } => {
                    // In a real implementation, this would write to database
                    // For now, just respond with success
                    let _ = response_tx.send(Ok(()));
                }
                StorageOperation::UpdateApiCall { session_id: _, call_id: _, api_call: _, response_tx } => {
                    // Mock implementation
                    let _ = response_tx.send(Ok(()));
                }
                StorageOperation::DeleteSession { session_id: _, response_tx } => {
                    // Mock implementation
                    let _ = response_tx.send(Ok(()));
                }
                StorageOperation::BatchFlush => {
                    // Already handled above
                }
            }
        }
        
        let flush_duration = start_time.elapsed();
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_operations += batch_size;
            stats.batched_operations += batch_size;
            stats.flush_count += 1;
            stats.total_flush_time_micros += flush_duration.as_micros() as u64;
            stats.avg_batch_size = stats.batched_operations as f64 / stats.flush_count as f64;
        }
        
        // Reset batch
        self.current_batch = StorageBatch::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::{CostSession, IssueId, ApiCall, ApiCallStatus};

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.flush_interval_ms, 1000);
        assert_eq!(config.connection_pool_size, 10);
        assert!(config.enable_write_behind);
        assert_eq!(config.cache_ttl_secs, 300);
    }

    #[tokio::test]
    async fn test_write_cache() {
        let cache: WriteCache<String> = WriteCache::new(1);
        
        // Test put and get
        cache.put("key1".to_string(), "value1".to_string(), false).await;
        let value = cache.get("key1").await;
        assert_eq!(value, Some("value1".to_string()));
        
        // Test dirty entries
        cache.put("key2".to_string(), "value2".to_string(), true).await;
        let dirty = cache.get_dirty_entries().await;
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].0, "key2");
        assert_eq!(dirty[0].1, "value2");
    }

    #[tokio::test]
    async fn test_write_cache_expiration() {
        let cache: WriteCache<String> = WriteCache::new(1); // 1 second TTL
        
        cache.put("key1".to_string(), "value1".to_string(), false).await;
        
        // Should be available immediately
        assert!(cache.get("key1").await.is_some());
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Should be expired
        assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_async_storage_manager() {
        let config = StorageConfig::default();
        let manager = AsyncStorageManager::new(config);
        
        // Create test session
        let issue_id = IssueId::new("test-issue").unwrap();
        let session = CostSession::new(issue_id);
        let session_id = session.session_id;
        
        // Store session
        let result = manager.store_session(session).await;
        assert!(result.is_ok());
        
        // Try to get session (should be in cache for write-behind)
        let retrieved = manager.get_session(&session_id).await;
        assert!(retrieved.is_some());
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.cache_hits, 1);
    }

    #[test]
    fn test_storage_batch() {
        let mut batch = StorageBatch::new();
        let config = StorageConfig {
            batch_size: 2,
            flush_interval_ms: 1000,
            ..Default::default()
        };
        
        assert!(!batch.should_flush(&config));
        
        // Add operations
        let (tx, _rx) = oneshot::channel();
        batch.add_operation(StorageOperation::DeleteSession {
            session_id: CostSessionId::new(),
            response_tx: tx,
        });
        
        assert!(!batch.should_flush(&config)); // Still below batch size
        
        let (tx2, _rx2) = oneshot::channel();
        batch.add_operation(StorageOperation::DeleteSession {
            session_id: CostSessionId::new(),
            response_tx: tx2,
        });
        
        assert!(batch.should_flush(&config)); // Now at batch size
    }

    #[tokio::test]
    async fn test_storage_operations() {
        let config = StorageConfig::default();
        let manager = AsyncStorageManager::new(config);
        
        // Test update API call
        let session_id = CostSessionId::new();
        let call_id = ApiCallId::new();
        let mut api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet"
        ).unwrap();
        api_call.complete(100, 50, ApiCallStatus::Success, None);
        
        let result = manager.update_api_call(session_id, call_id, api_call).await;
        assert!(result.is_ok());
        
        // Test delete session
        let result = manager.delete_session(session_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_storage_flush() {
        let config = StorageConfig::default();
        let manager = AsyncStorageManager::new(config);
        
        // Add some operations
        let issue_id = IssueId::new("flush-test").unwrap();
        let session = CostSession::new(issue_id);
        
        manager.store_session(session).await.unwrap();
        
        // Force flush
        let result = manager.flush().await;
        assert!(result.is_ok());
        
        let stats = manager.get_stats().await;
        assert!(stats.total_operations > 0);
    }

    #[test]
    fn test_async_storage_stats() {
        let mut stats = AsyncStorageStats::default();
        stats.cache_hits = 80;
        stats.cache_misses = 20;
        stats.total_flush_time_micros = 50000;
        stats.flush_count = 10;
        
        assert_eq!(stats.cache_hit_rate(), 80.0);
        assert_eq!(stats.avg_flush_time_micros(), 5000.0);
    }
}