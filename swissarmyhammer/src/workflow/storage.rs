//! Storage abstractions and implementations for workflows and workflow runs

use crate::security::MAX_DIRECTORY_DEPTH;
use crate::workflow::{MermaidParser, Workflow, WorkflowName, WorkflowRun, WorkflowRunId};
use crate::{Result, SwissArmyHammerError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::io::{Read, Write};

/// Source of a workflow (builtin, user, local, or dynamic)
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum WorkflowSource {
    /// Builtin workflows embedded in the binary or in resource directories
    Builtin,
    /// User workflows from ~/.swissarmyhammer/workflows
    User,
    /// Local workflows from .swissarmyhammer/workflows directories
    Local,
    /// Dynamically generated workflows
    Dynamic,
}

impl std::fmt::Display for WorkflowSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowSource::Builtin => write!(f, "builtin"),
            WorkflowSource::User => write!(f, "user"),
            WorkflowSource::Local => write!(f, "local"),
            WorkflowSource::Dynamic => write!(f, "dynamic"),
        }
    }
}

/// Handles loading workflows from various sources with proper precedence
pub struct WorkflowResolver {
    /// Track the source of each workflow by name
    pub workflow_sources: HashMap<WorkflowName, WorkflowSource>,
}

impl WorkflowResolver {
    /// Create a new WorkflowResolver
    pub fn new() -> Self {
        Self {
            workflow_sources: HashMap::new(),
        }
    }

    /// Get all directories that workflows are loaded from
    /// Returns paths in the same order as loading precedence
    pub fn get_workflow_directories(&self) -> Result<Vec<PathBuf>> {
        let mut directories = Vec::new();

        // User workflows directory
        if let Some(home) = dirs::home_dir() {
            let user_workflows_dir = home.join(".swissarmyhammer").join("workflows");
            if user_workflows_dir.exists() {
                directories.push(user_workflows_dir);
            }
        }

        // Local workflows directories (using same logic as prompts)
        let current_dir = std::env::current_dir()?;
        let mut workflow_dirs = Vec::new();
        let mut path = current_dir.as_path();
        let mut depth = 0;

        loop {
            // Limit traversal depth for security
            if depth >= MAX_DIRECTORY_DEPTH {
                break;
            }

            let swissarmyhammer_dir = path.join(".swissarmyhammer");
            if swissarmyhammer_dir.exists() && swissarmyhammer_dir.is_dir() {
                // Skip the user's home .swissarmyhammer directory to avoid duplicate
                if let Some(home) = dirs::home_dir() {
                    let user_swissarmyhammer_dir = home.join(".swissarmyhammer");
                    if swissarmyhammer_dir == user_swissarmyhammer_dir {
                        match path.parent() {
                            Some(parent) => {
                                path = parent;
                                depth += 1;
                            }
                            None => break,
                        }
                        continue;
                    }
                }

                let workflows_dir = swissarmyhammer_dir.join("workflows");
                if workflows_dir.exists() && workflows_dir.is_dir() {
                    workflow_dirs.push(workflows_dir);
                }
            }

            match path.parent() {
                Some(parent) => {
                    path = parent;
                    depth += 1;
                }
                None => break,
            }
        }

        // Add local directories in reverse order (root to current) to match loading order
        for workflows_dir in workflow_dirs.into_iter().rev() {
            directories.push(workflows_dir);
        }

        Ok(directories)
    }

    /// Load all workflows following the correct precedence:
    /// 1. Builtin workflows (least specific, embedded in binary or resource directories)
    /// 2. User workflows from ~/.swissarmyhammer/workflows
    /// 3. Local workflows from .swissarmyhammer directories (most specific)
    pub fn load_all_workflows(&mut self, storage: &mut dyn WorkflowStorageBackend) -> Result<()> {
        // Load builtin workflows first (least precedence)
        self.load_builtin_workflows(storage)?;

        // Load user workflows from home directory
        self.load_user_workflows(storage)?;

        // Load local workflows recursively (highest precedence)
        self.load_local_workflows(storage)?;

        Ok(())
    }

    /// Load builtin workflows from embedded binary data or resource directories
    pub fn load_builtin_workflows(
        &mut self,
        _storage: &mut dyn WorkflowStorageBackend,
    ) -> Result<()> {
        // For now, no builtin workflows are embedded
        // In the future, this could load from embedded workflow files
        // similar to how builtin prompts work
        Ok(())
    }

    /// Find workflow directories in a given base path
    fn find_workflow_directories(&self, base_path: &Path) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        let swissarmyhammer_dir = base_path.join(".swissarmyhammer");
        if swissarmyhammer_dir.exists() && swissarmyhammer_dir.is_dir() {
            let workflows_dir = swissarmyhammer_dir.join("workflows");
            if workflows_dir.exists() && workflows_dir.is_dir() {
                dirs.push(workflows_dir);
            }
        }
        dirs
    }

    /// Load user workflows from ~/.swissarmyhammer/workflows
    pub fn load_user_workflows(&mut self, storage: &mut dyn WorkflowStorageBackend) -> Result<()> {
        if let Some(home) = dirs::home_dir() {
            for workflows_dir in self.find_workflow_directories(&home) {
                self.load_workflows_from_directory(&workflows_dir, WorkflowSource::User, storage)?;
            }
        }
        Ok(())
    }

    /// Load local workflows by recursively searching up for .swissarmyhammer directories
    fn load_local_workflows(&mut self, storage: &mut dyn WorkflowStorageBackend) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let mut workflow_dirs = Vec::new();
        let mut path = current_dir.as_path();

        // Skip the user's home directory to avoid duplicates
        let user_home_swissarmyhammer = dirs::home_dir().map(|h| h.join(".swissarmyhammer"));

        loop {
            // Find workflow directories at this level
            let found_dirs = self.find_workflow_directories(path);

            // Only add if not the user's home .swissarmyhammer directory
            for dir in found_dirs {
                let parent_swissarmyhammer = dir.parent();
                if let (Some(parent), Some(ref user_dir)) =
                    (parent_swissarmyhammer, &user_home_swissarmyhammer)
                {
                    if parent == user_dir {
                        continue; // Skip user's home .swissarmyhammer/workflows
                    }
                }
                workflow_dirs.push(dir);
            }

            match path.parent() {
                Some(parent) => path = parent,
                None => break,
            }
        }

        // Load in reverse order (root to current) so deeper paths override
        for workflows_dir in workflow_dirs.into_iter().rev() {
            self.load_workflows_from_directory(&workflows_dir, WorkflowSource::Local, storage)?;
        }

        Ok(())
    }

    /// Load workflows from a specific directory
    fn load_workflows_from_directory(
        &mut self,
        directory: &Path,
        source: WorkflowSource,
        storage: &mut dyn WorkflowStorageBackend,
    ) -> Result<()> {
        for entry in walkdir::WalkDir::new(directory)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("mermaid") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(workflow) = MermaidParser::parse(&content, stem) {
                            // Track the workflow source
                            self.workflow_sources
                                .insert(workflow.name.clone(), source.clone());

                            // Store the workflow (this will override any existing workflow with the same name)
                            storage.store_workflow(workflow)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for WorkflowResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to walk a directory and load JSON files
fn load_json_files_from_directory<T, F>(
    directory: &Path,
    filename_filter: Option<&str>,
    mut loader: F,
) -> Result<Vec<T>>
where
    T: for<'de> serde::Deserialize<'de>,
    F: FnMut(T, &Path) -> bool,
{
    let mut items = Vec::new();

    if !directory.exists() {
        return Ok(items);
    }

    for entry in walkdir::WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            // Check filename filter if provided
            if let Some(filter) = filename_filter {
                if path.file_name().and_then(|s| s.to_str()) != Some(filter) {
                    continue;
                }
            }

            // Try to load and parse the JSON file
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(item) = serde_json::from_str::<T>(&content) {
                    if loader(item, path) {
                        // Loader returned true, meaning we should keep this item
                        if let Ok(item) = serde_json::from_str::<T>(&content) {
                            items.push(item);
                        }
                    }
                }
            }
        }
    }

    Ok(items)
}

/// Trait for workflow storage backends
pub trait WorkflowStorageBackend: Send + Sync {
    /// Store a workflow
    fn store_workflow(&mut self, workflow: Workflow) -> Result<()>;

    /// Get a workflow by name
    fn get_workflow(&self, name: &WorkflowName) -> Result<Workflow>;

    /// List all workflows
    fn list_workflows(&self) -> Result<Vec<Workflow>>;

    /// Remove a workflow
    fn remove_workflow(&mut self, name: &WorkflowName) -> Result<()>;

    /// Check if a workflow exists
    fn workflow_exists(&self, name: &WorkflowName) -> Result<bool> {
        self.get_workflow(name).map(|_| true).or_else(|e| match e {
            SwissArmyHammerError::WorkflowNotFound(_) => Ok(false),
            _ => Err(e),
        })
    }

    /// Clone the storage backend in a box
    fn clone_box(&self) -> Box<dyn WorkflowStorageBackend>;
}

/// Trait for workflow run storage backends
pub trait WorkflowRunStorageBackend: Send + Sync {
    /// Store a workflow run
    fn store_run(&mut self, run: &WorkflowRun) -> Result<()>;

    /// Get a workflow run by ID
    fn get_run(&self, id: &WorkflowRunId) -> Result<WorkflowRun>;

    /// List all workflow runs
    fn list_runs(&self) -> Result<Vec<WorkflowRun>>;

    /// Remove a workflow run
    fn remove_run(&mut self, id: &WorkflowRunId) -> Result<()>;

    /// List runs for a specific workflow
    fn list_runs_for_workflow(&self, workflow_name: &WorkflowName) -> Result<Vec<WorkflowRun>>;

    /// Clean up old runs (older than specified days)
    fn cleanup_old_runs(&mut self, days: u32) -> Result<u32>;

    /// Check if a run exists
    fn run_exists(&self, id: &WorkflowRunId) -> Result<bool> {
        self.get_run(id).map(|_| true).or_else(|e| match e {
            SwissArmyHammerError::WorkflowRunNotFound(_) => Ok(false),
            _ => Err(e),
        })
    }

    /// Clone the storage backend in a box
    fn clone_box(&self) -> Box<dyn WorkflowRunStorageBackend>;
}

/// In-memory workflow storage implementation
pub struct MemoryWorkflowStorage {
    workflows: HashMap<WorkflowName, Workflow>,
}

impl MemoryWorkflowStorage {
    /// Create a new memory workflow storage
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }
}

impl Default for MemoryWorkflowStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowStorageBackend for MemoryWorkflowStorage {
    fn store_workflow(&mut self, workflow: Workflow) -> Result<()> {
        self.workflows.insert(workflow.name.clone(), workflow);
        Ok(())
    }

    fn get_workflow(&self, name: &WorkflowName) -> Result<Workflow> {
        self.workflows
            .get(name)
            .cloned()
            .ok_or_else(|| SwissArmyHammerError::WorkflowNotFound(name.to_string()))
    }

    fn list_workflows(&self) -> Result<Vec<Workflow>> {
        Ok(self.workflows.values().cloned().collect())
    }

    fn remove_workflow(&mut self, name: &WorkflowName) -> Result<()> {
        self.workflows
            .remove(name)
            .ok_or_else(|| SwissArmyHammerError::WorkflowNotFound(name.to_string()))?;
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn WorkflowStorageBackend> {
        Box::new(MemoryWorkflowStorage {
            workflows: self.workflows.clone(),
        })
    }
}

/// In-memory workflow run storage implementation
pub struct MemoryWorkflowRunStorage {
    runs: HashMap<WorkflowRunId, WorkflowRun>,
}

impl MemoryWorkflowRunStorage {
    /// Create a new memory workflow run storage
    pub fn new() -> Self {
        Self {
            runs: HashMap::new(),
        }
    }
}

impl Default for MemoryWorkflowRunStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowRunStorageBackend for MemoryWorkflowRunStorage {
    fn store_run(&mut self, run: &WorkflowRun) -> Result<()> {
        self.runs.insert(run.id, run.clone());
        Ok(())
    }

    fn get_run(&self, id: &WorkflowRunId) -> Result<WorkflowRun> {
        self.runs
            .get(id)
            .cloned()
            .ok_or_else(|| SwissArmyHammerError::WorkflowRunNotFound(format!("{:?}", id)))
    }

    fn list_runs(&self) -> Result<Vec<WorkflowRun>> {
        Ok(self.runs.values().cloned().collect())
    }

    fn remove_run(&mut self, id: &WorkflowRunId) -> Result<()> {
        self.runs
            .remove(id)
            .ok_or_else(|| SwissArmyHammerError::WorkflowRunNotFound(format!("{:?}", id)))?;
        Ok(())
    }

    fn list_runs_for_workflow(&self, workflow_name: &WorkflowName) -> Result<Vec<WorkflowRun>> {
        Ok(self
            .runs
            .values()
            .filter(|run| &run.workflow.name == workflow_name)
            .cloned()
            .collect())
    }

    fn cleanup_old_runs(&mut self, days: u32) -> Result<u32> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let old_runs: Vec<WorkflowRunId> = self
            .runs
            .values()
            .filter(|run| run.started_at < cutoff)
            .map(|run| run.id)
            .collect();

        let count = old_runs.len() as u32;
        for id in old_runs {
            self.runs.remove(&id);
        }

        Ok(count)
    }

    fn clone_box(&self) -> Box<dyn WorkflowRunStorageBackend> {
        Box::new(MemoryWorkflowRunStorage {
            runs: self.runs.clone(),
        })
    }
}

/// File system workflow storage implementation that uses WorkflowResolver for hierarchical loading
pub struct FileSystemWorkflowStorage {
    cache: dashmap::DashMap<WorkflowName, Workflow>,
    resolver: WorkflowResolver,
}

impl FileSystemWorkflowStorage {
    /// Create a new file system workflow storage
    pub fn new() -> Result<Self> {
        let mut storage = Self {
            cache: dashmap::DashMap::new(),
            resolver: WorkflowResolver::new(),
        };

        // Load workflows from all hierarchical sources
        storage.reload_cache()?;

        Ok(storage)
    }

    /// Reload the cache from disk using hierarchical loading
    pub fn reload_cache(&mut self) -> Result<()> {
        self.cache.clear();
        self.resolver.workflow_sources.clear();

        // Create a temporary memory storage to collect workflows
        let mut temp_storage = MemoryWorkflowStorage::new();

        // Use the resolver to load workflows with proper precedence
        self.resolver.load_all_workflows(&mut temp_storage)?;

        // Transfer workflows from temp storage to our cache
        for workflow in temp_storage.list_workflows()? {
            self.cache.insert(workflow.name.clone(), workflow);
        }

        Ok(())
    }

    /// Get the source of a workflow
    pub fn get_workflow_source(&self, name: &WorkflowName) -> Option<&WorkflowSource> {
        self.resolver.workflow_sources.get(name)
    }

    /// Get all workflow directories being monitored
    pub fn get_workflow_directories(&self) -> Result<Vec<PathBuf>> {
        self.resolver.get_workflow_directories()
    }

    /// Find the appropriate path to store a workflow (uses local directory if available, falls back to user)
    fn workflow_storage_path(&self, name: &WorkflowName) -> Result<PathBuf> {
        // Try to find a local .swissarmyhammer directory first
        let current_dir = std::env::current_dir()?;
        let local_dir = current_dir.join(".swissarmyhammer").join("workflows");
        if local_dir.exists() {
            return Ok(local_dir.join(format!("{}.mermaid", name.as_str())));
        }

        // Fall back to user directory
        if let Some(home) = dirs::home_dir() {
            let user_dir = home.join(".swissarmyhammer").join("workflows");
            std::fs::create_dir_all(&user_dir)?;
            return Ok(user_dir.join(format!("{}.mermaid", name.as_str())));
        }

        Err(SwissArmyHammerError::Storage(
            "No suitable directory found for storing workflow. Please create .swissarmyhammer/workflows in current directory or ensure HOME directory is accessible".to_string(),
        ))
    }
}

impl WorkflowStorageBackend for FileSystemWorkflowStorage {
    fn store_workflow(&mut self, workflow: Workflow) -> Result<()> {
        let path = self.workflow_storage_path(&workflow.name)?;

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // For now, store as JSON since we don't have mermaid serialization
        // In practice, this would serialize back to mermaid format
        let content = serde_json::to_string_pretty(&workflow)?;
        std::fs::write(&path, content)?;

        // Update cache and source tracking
        self.cache.insert(workflow.name.clone(), workflow.clone());

        // Determine source based on storage location
        let source = if path.starts_with(
            dirs::home_dir()
                .unwrap_or_default()
                .join(".swissarmyhammer"),
        ) {
            WorkflowSource::User
        } else {
            WorkflowSource::Local
        };
        self.resolver.workflow_sources.insert(workflow.name, source);

        Ok(())
    }

    fn get_workflow(&self, name: &WorkflowName) -> Result<Workflow> {
        if let Some(workflow) = self.cache.get(name) {
            return Ok(workflow.clone());
        }

        // If not in cache, workflow doesn't exist in our hierarchical loading
        Err(SwissArmyHammerError::WorkflowNotFound(name.to_string()))
    }

    fn list_workflows(&self) -> Result<Vec<Workflow>> {
        Ok(self
            .cache
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    fn remove_workflow(&mut self, name: &WorkflowName) -> Result<()> {
        // Find the workflow file in the appropriate directory
        let path = self.workflow_storage_path(name)?;
        if path.exists() {
            std::fs::remove_file(path)?;
        }

        // Remove from cache and source tracking
        self.cache.remove(name);
        self.resolver.workflow_sources.remove(name);
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn WorkflowStorageBackend> {
        // For cloning, create a new instance and reload
        let mut new_storage = FileSystemWorkflowStorage {
            cache: dashmap::DashMap::new(),
            resolver: WorkflowResolver::new(),
        };

        // Copy current cache state
        for entry in self.cache.iter() {
            new_storage
                .cache
                .insert(entry.key().clone(), entry.value().clone());
        }

        // Copy resolver state
        new_storage.resolver.workflow_sources = self.resolver.workflow_sources.clone();

        Box::new(new_storage)
    }
}

/// File system workflow run storage implementation
pub struct FileSystemWorkflowRunStorage {
    base_path: PathBuf,
    cache: dashmap::DashMap<WorkflowRunId, WorkflowRun>,
}

impl FileSystemWorkflowRunStorage {
    /// Create a new file system workflow run storage
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)?;
        }

        let storage = Self {
            base_path,
            cache: dashmap::DashMap::new(),
        };

        // Load existing runs into cache
        storage.reload_cache()?;

        Ok(storage)
    }

    /// Reload the cache from disk
    pub fn reload_cache(&self) -> Result<()> {
        self.cache.clear();

        let runs_dir = self.base_path.join("runs");
        if !runs_dir.exists() {
            std::fs::create_dir_all(&runs_dir)?;
        }

        // Use the helper function to load workflow runs
        let cache_ref = &self.cache;
        load_json_files_from_directory::<WorkflowRun, _>(
            &runs_dir,
            Some("run.json"),
            |run, _path| {
                cache_ref.insert(run.id, run);
                true
            },
        )?;

        Ok(())
    }

    fn run_path(&self, id: &WorkflowRunId) -> PathBuf {
        self.base_path
            .join("runs")
            .join(format!("{:?}", id))
            .join("run.json")
    }

    fn run_dir(&self, id: &WorkflowRunId) -> PathBuf {
        self.base_path.join("runs").join(format!("{:?}", id))
    }
}

impl WorkflowRunStorageBackend for FileSystemWorkflowRunStorage {
    fn store_run(&mut self, run: &WorkflowRun) -> Result<()> {
        let run_dir = self.run_dir(&run.id);
        if !run_dir.exists() {
            std::fs::create_dir_all(&run_dir)?;
        }

        let path = self.run_path(&run.id);
        let content = serde_json::to_string_pretty(run)?;
        std::fs::write(&path, content)?;

        self.cache.insert(run.id, run.clone());
        Ok(())
    }

    fn get_run(&self, id: &WorkflowRunId) -> Result<WorkflowRun> {
        if let Some(run) = self.cache.get(id) {
            return Ok(run.clone());
        }

        let path = self.run_path(id);
        if !path.exists() {
            return Err(SwissArmyHammerError::WorkflowRunNotFound(format!(
                "{:?}",
                id
            )));
        }

        let content = std::fs::read_to_string(&path)?;
        let run: WorkflowRun = serde_json::from_str(&content)?;
        self.cache.insert(*id, run.clone());

        Ok(run)
    }

    fn list_runs(&self) -> Result<Vec<WorkflowRun>> {
        Ok(self
            .cache
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    fn remove_run(&mut self, id: &WorkflowRunId) -> Result<()> {
        let run_dir = self.run_dir(id);
        if !run_dir.exists() {
            return Err(SwissArmyHammerError::WorkflowRunNotFound(format!(
                "{:?}",
                id
            )));
        }

        std::fs::remove_dir_all(run_dir)?;
        self.cache.remove(id);
        Ok(())
    }

    fn list_runs_for_workflow(&self, workflow_name: &WorkflowName) -> Result<Vec<WorkflowRun>> {
        Ok(self
            .cache
            .iter()
            .filter(|entry| &entry.value().workflow.name == workflow_name)
            .map(|entry| entry.value().clone())
            .collect())
    }

    fn cleanup_old_runs(&mut self, days: u32) -> Result<u32> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let old_runs: Vec<WorkflowRunId> = self
            .cache
            .iter()
            .filter(|entry| entry.value().started_at < cutoff)
            .map(|entry| *entry.key())
            .collect();

        let count = old_runs.len() as u32;
        for id in old_runs {
            self.remove_run(&id)?;
        }

        Ok(count)
    }

    fn clone_box(&self) -> Box<dyn WorkflowRunStorageBackend> {
        Box::new(FileSystemWorkflowRunStorage {
            base_path: self.base_path.clone(),
            cache: self.cache.clone(),
        })
    }
}

/// Main workflow storage that can use different backends
pub struct WorkflowStorage {
    workflow_backend: Arc<dyn WorkflowStorageBackend>,
    run_backend: Arc<dyn WorkflowRunStorageBackend>,
}

impl WorkflowStorage {
    /// Create a new workflow storage with the given backends
    pub fn new(
        workflow_backend: Arc<dyn WorkflowStorageBackend>,
        run_backend: Arc<dyn WorkflowRunStorageBackend>,
    ) -> Self {
        Self {
            workflow_backend,
            run_backend,
        }
    }

    /// Create with memory backends
    pub fn memory() -> Self {
        Self::new(
            Arc::new(MemoryWorkflowStorage::new()),
            Arc::new(MemoryWorkflowRunStorage::new()),
        )
    }

    /// Create with file system backends using hierarchical loading
    pub fn file_system() -> Result<Self> {
        // Use a user directory as base path for workflow runs
        let base_path = dirs::home_dir()
            .ok_or_else(|| {
                SwissArmyHammerError::Storage(
                    "Cannot find home directory. Please ensure HOME environment variable is set"
                        .to_string(),
                )
            })?
            .join(".swissarmyhammer");

        Ok(Self::new(
            Arc::new(FileSystemWorkflowStorage::new()?),
            Arc::new(FileSystemWorkflowRunStorage::new(&base_path)?),
        ))
    }

    /// Store a workflow
    pub fn store_workflow(&mut self, workflow: Workflow) -> Result<()> {
        Arc::get_mut(&mut self.workflow_backend)
            .ok_or_else(|| {
                SwissArmyHammerError::Storage(
                    "Cannot get mutable reference to workflow storage backend".to_string(),
                )
            })?
            .store_workflow(workflow)
    }

    /// Get a workflow by name
    pub fn get_workflow(&self, name: &WorkflowName) -> Result<Workflow> {
        self.workflow_backend.get_workflow(name)
    }

    /// List all workflows
    pub fn list_workflows(&self) -> Result<Vec<Workflow>> {
        self.workflow_backend.list_workflows()
    }

    /// Remove a workflow
    pub fn remove_workflow(&mut self, name: &WorkflowName) -> Result<()> {
        Arc::get_mut(&mut self.workflow_backend)
            .ok_or_else(|| {
                SwissArmyHammerError::Storage(
                    "Cannot get mutable reference to workflow storage backend".to_string(),
                )
            })?
            .remove_workflow(name)
    }

    /// Store a workflow run
    pub fn store_run(&mut self, run: &WorkflowRun) -> Result<()> {
        Arc::get_mut(&mut self.run_backend)
            .ok_or_else(|| {
                SwissArmyHammerError::Storage(
                    "Cannot get mutable reference to run storage backend".to_string(),
                )
            })?
            .store_run(run)
    }

    /// Get a workflow run by ID
    pub fn get_run(&self, id: &WorkflowRunId) -> Result<WorkflowRun> {
        self.run_backend.get_run(id)
    }

    /// List all workflow runs
    pub fn list_runs(&self) -> Result<Vec<WorkflowRun>> {
        self.run_backend.list_runs()
    }

    /// Remove a workflow run
    pub fn remove_run(&mut self, id: &WorkflowRunId) -> Result<()> {
        Arc::get_mut(&mut self.run_backend)
            .ok_or_else(|| {
                SwissArmyHammerError::Storage(
                    "Cannot get mutable reference to run storage backend".to_string(),
                )
            })?
            .remove_run(id)
    }

    /// List runs for a specific workflow
    pub fn list_runs_for_workflow(&self, workflow_name: &WorkflowName) -> Result<Vec<WorkflowRun>> {
        self.run_backend.list_runs_for_workflow(workflow_name)
    }

    /// Clean up old runs
    pub fn cleanup_old_runs(&mut self, days: u32) -> Result<u32> {
        Arc::get_mut(&mut self.run_backend)
            .ok_or_else(|| {
                SwissArmyHammerError::Storage(
                    "Cannot get mutable reference to run storage backend".to_string(),
                )
            })?
            .cleanup_old_runs(days)
    }
}

/// Compressed workflow storage that wraps another storage backend
pub struct CompressedWorkflowStorage {
    inner: Box<dyn WorkflowStorageBackend>,
    compression_level: i32,
}

impl CompressedWorkflowStorage {
    /// Create a new compressed storage wrapper
    pub fn new(inner: Box<dyn WorkflowStorageBackend>, compression_level: i32) -> Self {
        Self {
            inner,
            compression_level: compression_level.clamp(1, 22), // zstd compression levels 1-22
        }
    }

    /// Create with default compression level (3)
    pub fn with_default_compression(inner: Box<dyn WorkflowStorageBackend>) -> Self {
        Self::new(inner, 3)
    }

    /// Compress data using zstd
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::encode_all(data, self.compression_level)
            .map_err(|e| SwissArmyHammerError::Storage(format!("Compression failed: {}", e)))
    }

    /// Decompress data using zstd
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::decode_all(data)
            .map_err(|e| SwissArmyHammerError::Storage(format!("Decompression failed: {}", e)))
    }
}

impl WorkflowStorageBackend for CompressedWorkflowStorage {
    fn store_workflow(&mut self, workflow: Workflow) -> Result<()> {
        // Serialize workflow to JSON
        let json_data = serde_json::to_vec(&workflow)
            .map_err(|e| SwissArmyHammerError::Storage(format!("Serialization failed: {}", e)))?;

        // Compress the JSON data
        let compressed_data = self.compress_data(&json_data)?;

        // Create a temporary workflow with compressed data stored as description
        // This is a workaround since we can't modify the storage interface
        let mut compressed_workflow = workflow.clone();
        compressed_workflow.description = format!(
            "COMPRESSED_DATA:{}",
            base64::encode(&compressed_data)
        );

        self.inner.store_workflow(compressed_workflow)
    }

    fn get_workflow(&self, name: &WorkflowName) -> Result<Workflow> {
        let stored_workflow = self.inner.get_workflow(name)?;

        // Check if this is compressed data
        if stored_workflow.description.starts_with("COMPRESSED_DATA:") {
            let encoded_data = &stored_workflow.description[16..]; // Skip "COMPRESSED_DATA:"
            let compressed_data = base64::decode(encoded_data)
                .map_err(|e| SwissArmyHammerError::Storage(format!("Base64 decode failed: {}", e)))?;

            let json_data = self.decompress_data(&compressed_data)?;
            let workflow: Workflow = serde_json::from_slice(&json_data)
                .map_err(|e| SwissArmyHammerError::Storage(format!("Deserialization failed: {}", e)))?;

            Ok(workflow)
        } else {
            // Not compressed, return as-is
            Ok(stored_workflow)
        }
    }

    fn list_workflows(&self) -> Result<Vec<Workflow>> {
        let stored_workflows = self.inner.list_workflows()?;
        let mut workflows = Vec::new();

        for stored_workflow in stored_workflows {
            if stored_workflow.description.starts_with("COMPRESSED_DATA:") {
                let encoded_data = &stored_workflow.description[16..];
                let compressed_data = base64::decode(encoded_data)
                    .map_err(|e| SwissArmyHammerError::Storage(format!("Base64 decode failed: {}", e)))?;

                let json_data = self.decompress_data(&compressed_data)?;
                let workflow: Workflow = serde_json::from_slice(&json_data)
                    .map_err(|e| SwissArmyHammerError::Storage(format!("Deserialization failed: {}", e)))?;

                workflows.push(workflow);
            } else {
                workflows.push(stored_workflow);
            }
        }

        Ok(workflows)
    }

    fn remove_workflow(&mut self, name: &WorkflowName) -> Result<()> {
        self.inner.remove_workflow(name)
    }

    fn clone_box(&self) -> Box<dyn WorkflowStorageBackend> {
        Box::new(CompressedWorkflowStorage {
            inner: self.inner.clone_box(),
            compression_level: self.compression_level,
        })
    }
}

impl WorkflowStorage {
    /// Create with compressed file system backends
    pub fn compressed_file_system() -> Result<Self> {
        let base_path = dirs::home_dir()
            .ok_or_else(|| {
                SwissArmyHammerError::Storage(
                    "Cannot find home directory. Please ensure HOME environment variable is set"
                        .to_string(),
                )
            })?
            .join(".swissarmyhammer");

        let workflow_backend = CompressedWorkflowStorage::with_default_compression(
            Box::new(FileSystemWorkflowStorage::new()?)
        );

        Ok(Self::new(
            Arc::new(workflow_backend),
            Arc::new(FileSystemWorkflowRunStorage::new(&base_path)?),
        ))
    }

    /// Create with compressed memory backends (for testing)
    pub fn compressed_memory() -> Self {
        let workflow_backend = CompressedWorkflowStorage::with_default_compression(
            Box::new(MemoryWorkflowStorage::new())
        );

        Self::new(
            Arc::new(workflow_backend),
            Arc::new(MemoryWorkflowRunStorage::new()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{State, StateId, StateType};

    fn create_test_workflow() -> Workflow {
        let mut workflow = Workflow::new(
            WorkflowName::new("test-workflow"),
            "A test workflow".to_string(),
            StateId::new("start"),
        );

        workflow.add_state(State {
            id: StateId::new("start"),
            description: "Start state".to_string(),
            state_type: StateType::Normal,
            is_terminal: false,
            allows_parallel: false,
            metadata: HashMap::new(),
        });

        workflow.add_state(State {
            id: StateId::new("end"),
            description: "End state".to_string(),
            state_type: StateType::Normal,
            is_terminal: true,
            allows_parallel: false,
            metadata: HashMap::new(),
        });

        workflow
    }

    #[test]
    fn test_memory_workflow_storage() {
        let mut storage = MemoryWorkflowStorage::new();
        let workflow = create_test_workflow();

        storage.store_workflow(workflow.clone()).unwrap();

        let retrieved = storage.get_workflow(&workflow.name).unwrap();
        assert_eq!(retrieved.name, workflow.name);

        let list = storage.list_workflows().unwrap();
        assert_eq!(list.len(), 1);

        storage.remove_workflow(&workflow.name).unwrap();
        assert!(storage.get_workflow(&workflow.name).is_err());
    }

    #[test]
    fn test_memory_workflow_run_storage() {
        let mut storage = MemoryWorkflowRunStorage::new();
        let workflow = create_test_workflow();
        let run = WorkflowRun::new(workflow.clone());

        storage.store_run(&run).unwrap();

        let retrieved = storage.get_run(&run.id).unwrap();
        assert_eq!(retrieved.id, run.id);

        let list = storage.list_runs().unwrap();
        assert_eq!(list.len(), 1);

        let workflow_runs = storage.list_runs_for_workflow(&workflow.name).unwrap();
        assert_eq!(workflow_runs.len(), 1);

        storage.remove_run(&run.id).unwrap();
        assert!(storage.get_run(&run.id).is_err());
    }

    #[test]
    fn test_cleanup_old_runs() {
        let mut storage = MemoryWorkflowRunStorage::new();
        let workflow = create_test_workflow();

        // Create an old run
        let mut old_run = WorkflowRun::new(workflow.clone());
        old_run.started_at = chrono::Utc::now() - chrono::Duration::days(10);

        // Create a recent run
        let recent_run = WorkflowRun::new(workflow);

        storage.store_run(&old_run).unwrap();
        storage.store_run(&recent_run).unwrap();

        let cleaned = storage.cleanup_old_runs(7).unwrap();
        assert_eq!(cleaned, 1);

        let remaining = storage.list_runs().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, recent_run.id);
    }

    #[test]
    fn test_combined_workflow_storage() {
        let mut storage = WorkflowStorage::memory();
        let workflow = create_test_workflow();
        let run = WorkflowRun::new(workflow.clone());

        // Test workflow operations
        storage.store_workflow(workflow.clone()).unwrap();
        let retrieved_workflow = storage.get_workflow(&workflow.name).unwrap();
        assert_eq!(retrieved_workflow.name, workflow.name);

        // Test run operations
        storage.store_run(&run).unwrap();
        let retrieved_run = storage.get_run(&run.id).unwrap();
        assert_eq!(retrieved_run.id, run.id);

        // Test listing runs for workflow
        let workflow_runs = storage.list_runs_for_workflow(&workflow.name).unwrap();
        assert_eq!(workflow_runs.len(), 1);
    }

    #[test]
    fn test_compressed_workflow_storage() {
        let mut storage = CompressedWorkflowStorage::with_default_compression(
            Box::new(MemoryWorkflowStorage::new())
        );
        let workflow = create_test_workflow();

        // Store compressed workflow
        storage.store_workflow(workflow.clone()).unwrap();

        // Retrieve and verify
        let retrieved = storage.get_workflow(&workflow.name).unwrap();
        assert_eq!(retrieved.name, workflow.name);
        assert_eq!(retrieved.description, workflow.description);
        assert_eq!(retrieved.states.len(), workflow.states.len());

        // Test listing
        let list = storage.list_workflows().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, workflow.name);

        // Test removal
        storage.remove_workflow(&workflow.name).unwrap();
        assert!(storage.get_workflow(&workflow.name).is_err());
    }

    #[test]
    fn test_compressed_storage_integration() {
        let mut storage = WorkflowStorage::compressed_memory();
        let workflow = create_test_workflow();
        let run = WorkflowRun::new(workflow.clone());

        // Test workflow operations with compression
        storage.store_workflow(workflow.clone()).unwrap();
        let retrieved_workflow = storage.get_workflow(&workflow.name).unwrap();
        assert_eq!(retrieved_workflow.name, workflow.name);

        // Test that compression doesn't affect run operations
        storage.store_run(&run).unwrap();
        let retrieved_run = storage.get_run(&run.id).unwrap();
        assert_eq!(retrieved_run.id, run.id);

        let workflow_runs = storage.list_runs_for_workflow(&workflow.name).unwrap();
        assert_eq!(workflow_runs.len(), 1);
    }

    #[test]
    fn test_workflow_resolver_user_workflows() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let user_workflows_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
        fs::create_dir_all(&user_workflows_dir).unwrap();

        // Create a test workflow file
        let workflow_file = user_workflows_dir.join("test_workflow.mermaid");
        let workflow_content = r#"
        stateDiagram-v2
            [*] --> State1
            State1 --> [*]
        "#;
        fs::write(&workflow_file, workflow_content).unwrap();

        let mut resolver = WorkflowResolver::new();
        let mut storage = MemoryWorkflowStorage::new();

        // Temporarily change home directory for test
        std::env::set_var("HOME", temp_dir.path());

        resolver.load_user_workflows(&mut storage).unwrap();

        let workflows = storage.list_workflows().unwrap();
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name.as_str(), "test_workflow");
        assert_eq!(
            resolver.workflow_sources.get(&workflows[0].name),
            Some(&WorkflowSource::User)
        );
    }

    #[test]
    fn test_workflow_resolver_local_workflows() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let local_workflows_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
        fs::create_dir_all(&local_workflows_dir).unwrap();

        // Create a test workflow file
        let workflow_file = local_workflows_dir.join("local_workflow.mermaid");
        let workflow_content = r#"
        stateDiagram-v2
            [*] --> Processing
            Processing --> [*]
        "#;
        fs::write(&workflow_file, workflow_content).unwrap();

        let mut resolver = WorkflowResolver::new();
        let mut storage = MemoryWorkflowStorage::new();

        // Change to the temp directory to simulate local workflows
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        resolver.load_local_workflows(&mut storage).unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        let workflows = storage.list_workflows().unwrap();
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name.as_str(), "local_workflow");
        assert_eq!(
            resolver.workflow_sources.get(&workflows[0].name),
            Some(&WorkflowSource::Local)
        );
    }

    #[test]
    fn test_workflow_resolver_precedence() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create user workflow directory
        let user_workflows_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
        fs::create_dir_all(&user_workflows_dir).unwrap();

        // Create local workflow directory
        let local_workflows_dir = temp_dir
            .path()
            .join("project")
            .join(".swissarmyhammer")
            .join("workflows");
        fs::create_dir_all(&local_workflows_dir).unwrap();

        // Create same-named workflow in both locations
        let workflow_content_user = r#"
        stateDiagram-v2
            [*] --> UserState
            UserState --> [*]
        "#;
        let workflow_content_local = r#"
        stateDiagram-v2
            [*] --> LocalState
            LocalState --> [*]
        "#;

        fs::write(
            user_workflows_dir.join("same_name.mermaid"),
            workflow_content_user,
        )
        .unwrap();
        fs::write(
            local_workflows_dir.join("same_name.mermaid"),
            workflow_content_local,
        )
        .unwrap();

        let mut resolver = WorkflowResolver::new();
        let mut storage = MemoryWorkflowStorage::new();

        // Temporarily change home directory and current directory for test
        std::env::set_var("HOME", temp_dir.path());
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path().join("project")).unwrap();

        // Load all workflows (user first, then local to test precedence)
        resolver.load_user_workflows(&mut storage).unwrap();
        resolver.load_local_workflows(&mut storage).unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        let workflows = storage.list_workflows().unwrap();
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name.as_str(), "same_name");

        // Local should have overridden user
        assert_eq!(
            resolver.workflow_sources.get(&workflows[0].name),
            Some(&WorkflowSource::Local)
        );

        // Verify the workflow content is from the local version
        assert!(workflows[0]
            .states
            .contains_key(&StateId::new("LocalState")));
        assert!(!workflows[0].states.contains_key(&StateId::new("UserState")));
    }

    #[test]
    fn test_workflow_directories() {
        let resolver = WorkflowResolver::new();
        let directories = resolver.get_workflow_directories().unwrap();

        // Should return a vector of PathBuf (may be empty if no directories exist)
        // All returned paths should be absolute and existing
        for dir in directories {
            assert!(dir.is_absolute());
            assert!(dir.exists());
            assert!(dir.is_dir());
        }
    }
}
