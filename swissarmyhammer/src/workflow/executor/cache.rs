//! Cache management for workflow execution

use super::WorkflowExecutor;
use crate::workflow::{StateId, TransitionKey, TransitionPath, WorkflowCacheManager};

impl WorkflowExecutor {
    /// Check if a CEL program is cached
    pub fn is_cel_program_cached(&self, expression: &str) -> bool {
        self.cache_manager.cel_cache.get(expression).is_some()
    }

    /// Get CEL program cache statistics
    pub fn get_cel_cache_stats(&self) -> (usize, usize) {
        let stats = self.cache_manager.cel_cache.stats();
        (stats.size, stats.capacity)
    }

    /// Get cache manager for advanced cache operations
    pub fn get_cache_manager(&self) -> &WorkflowCacheManager {
        &self.cache_manager
    }

    /// Get mutable cache manager for advanced cache operations
    pub fn get_cache_manager_mut(&mut self) -> &mut WorkflowCacheManager {
        &mut self.cache_manager
    }

    /// Cache a transition path for optimization
    pub fn cache_transition_path(
        &mut self,
        from_state: StateId,
        to_state: StateId,
        conditions: Vec<String>,
    ) {
        let key = TransitionKey::new(from_state.clone(), to_state.clone());
        let path = TransitionPath::new(from_state, to_state, conditions);
        self.cache_manager.transition_cache.put(key, path);
    }

    /// Get cached transition path if available
    pub fn get_cached_transition_path(
        &self,
        from_state: &StateId,
        to_state: &StateId,
    ) -> Option<TransitionPath> {
        let key = TransitionKey::new(from_state.clone(), to_state.clone());
        self.cache_manager.transition_cache.get(&key)
    }

    /// Clear all caches
    pub fn clear_all_caches(&mut self) {
        self.cache_manager.clear_all();
    }
}
