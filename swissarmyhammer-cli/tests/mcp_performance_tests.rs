use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use swissarmyhammer::PromptLibrary;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Performance metrics for MCP operations
#[derive(Debug, Default)]
struct PerformanceMetrics {
    total_operations: u64,
    total_duration: Duration,
    min_duration: Option<Duration>,
    max_duration: Option<Duration>,
    errors: u64,
}

impl PerformanceMetrics {
    fn record_operation(&mut self, duration: Duration, success: bool) {
        self.total_operations += 1;
        self.total_duration += duration;
        
        if !success {
            self.errors += 1;
        }
        
        match self.min_duration {
            None => self.min_duration = Some(duration),
            Some(min) if duration < min => self.min_duration = Some(duration),
            _ => {}
        }
        
        match self.max_duration {
            None => self.max_duration = Some(duration),
            Some(max) if duration > max => self.max_duration = Some(duration),
            _ => {}
        }
    }
    
    fn average_duration(&self) -> Duration {
        if self.total_operations > 0 {
            self.total_duration / self.total_operations as u32
        } else {
            Duration::ZERO
        }
    }
    
    fn success_rate(&self) -> f64 {
        if self.total_operations > 0 {
            ((self.total_operations - self.errors) as f64 / self.total_operations as f64) * 100.0
        } else {
            0.0
        }
    }
}

async fn create_large_prompt_library(temp_dir: &TempDir, count: usize) -> Result<PromptLibrary> {
    let prompts_dir = temp_dir.path().join("prompts");
    std::fs::create_dir_all(&prompts_dir)?;
    
    // Create many prompt files
    for i in 0..count {
        let prompt_file = prompts_dir.join(format!("prompt_{:04}.md", i));
        let content = format!(r#"---
name: prompt_{:04}
description: Performance test prompt {}
category: performance
tags: ["test", "performance", "load"]
arguments:
  - name: arg1
    description: First argument
    required: true
  - name: arg2
    description: Second argument
    required: false
    default: "default_value"
  - name: arg3
    description: Third argument
    required: false
---
This is performance test prompt {} with template content.

{{{{arg1}}}} is required.
{{%- if arg2 %}}
{{{{arg2}}}}
{{%- else %}}
default_value
{{%- endif %}} has a default.
{{%- if arg3 %}}
Optional arg3: {{{{arg3}}}}
{{%- endif %}}

Multiple lines to simulate
a more realistic prompt
with various content types
and formatting."#, i, i, i);
        
        std::fs::write(&prompt_file, content)?;
    }
    
    let mut library = PromptLibrary::new();
    library.add_directory(&prompts_dir)?;
    
    Ok(library)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_load_performance_small() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        let start = Instant::now();
        let library = create_large_prompt_library(&temp_dir, 100).await?;
        let duration = start.elapsed();
        
        println!("Loading 100 prompts took: {:?}", duration);
        assert!(duration < Duration::from_secs(2), "Loading 100 prompts took too long");
        
        let prompts = library.list()?;
        assert_eq!(prompts.len(), 100);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_load_performance_medium() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        let start = Instant::now();
        let library = create_large_prompt_library(&temp_dir, 500).await?;
        let duration = start.elapsed();
        
        println!("Loading 500 prompts took: {:?}", duration);
        assert!(duration < Duration::from_secs(5), "Loading 500 prompts took too long");
        
        let prompts = library.list()?;
        assert_eq!(prompts.len(), 500);
        
        Ok(())
    }

    #[tokio::test]
    #[ignore] // Run with --ignored flag for full performance tests
    async fn test_load_performance_large() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        let start = Instant::now();
        let library = create_large_prompt_library(&temp_dir, 5000).await?;
        let duration = start.elapsed();
        
        println!("Loading 5000 prompts took: {:?}", duration);
        assert!(duration < Duration::from_secs(30), "Loading 5000 prompts took too long");
        
        let prompts = library.list()?;
        assert_eq!(prompts.len(), 5000);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_list_prompts_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let library = Arc::new(RwLock::new(create_large_prompt_library(&temp_dir, 1000).await?));
        
        let mut metrics = PerformanceMetrics::default();
        
        // Perform many list operations
        for _ in 0..100 {
            let start = Instant::now();
            let result = library.read().await.list();
            let duration = start.elapsed();
            
            metrics.record_operation(duration, result.is_ok());
        }
        
        println!("List prompts performance:");
        println!("  Total operations: {}", metrics.total_operations);
        println!("  Average duration: {:?}", metrics.average_duration());
        println!("  Min duration: {:?}", metrics.min_duration.unwrap_or_default());
        println!("  Max duration: {:?}", metrics.max_duration.unwrap_or_default());
        println!("  Success rate: {:.2}%", metrics.success_rate());
        
        assert!(metrics.average_duration() < Duration::from_millis(10));
        assert_eq!(metrics.success_rate(), 100.0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_prompt_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let library = Arc::new(RwLock::new(create_large_prompt_library(&temp_dir, 1000).await?));
        
        let mut metrics = PerformanceMetrics::default();
        
        // Perform many get operations
        for i in 0..100 {
            let prompt_name = format!("prompt_{:04}", i % 1000);
            let start = Instant::now();
            let result = library.read().await.get(&prompt_name);
            let duration = start.elapsed();
            
            metrics.record_operation(duration, result.is_ok());
        }
        
        println!("Get prompt performance:");
        println!("  Total operations: {}", metrics.total_operations);
        println!("  Average duration: {:?}", metrics.average_duration());
        println!("  Min duration: {:?}", metrics.min_duration.unwrap_or_default());
        println!("  Max duration: {:?}", metrics.max_duration.unwrap_or_default());
        println!("  Success rate: {:.2}%", metrics.success_rate());
        
        assert!(metrics.average_duration() < Duration::from_millis(5));
        assert_eq!(metrics.success_rate(), 100.0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_render_prompt_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let library = Arc::new(RwLock::new(create_large_prompt_library(&temp_dir, 100).await?));
        
        let mut metrics = PerformanceMetrics::default();
        
        // Prepare arguments
        let mut args = HashMap::new();
        args.insert("arg1".to_string(), "test_value".to_string());
        args.insert("arg2".to_string(), "custom_value".to_string());
        args.insert("arg3".to_string(), "optional_value".to_string());
        
        // Perform many render operations
        for i in 0..1000 {
            let prompt_name = format!("prompt_{:04}", i % 100);
            let start = Instant::now();
            
            let result = {
                let lib = library.read().await;
                match lib.get(&prompt_name) {
                    Ok(prompt) => prompt.render(&args).map_err(|e| anyhow::anyhow!(e)),
                    Err(_) => Err(anyhow::anyhow!("Prompt not found"))
                }
            };
            
            let duration = start.elapsed();
            metrics.record_operation(duration, result.is_ok());
        }
        
        println!("Render prompt performance:");
        println!("  Total operations: {}", metrics.total_operations);
        println!("  Average duration: {:?}", metrics.average_duration());
        println!("  Min duration: {:?}", metrics.min_duration.unwrap_or_default());
        println!("  Max duration: {:?}", metrics.max_duration.unwrap_or_default());
        println!("  Success rate: {:.2}%", metrics.success_rate());
        
        assert!(metrics.average_duration() < Duration::from_millis(2));
        assert_eq!(metrics.success_rate(), 100.0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_access_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let library = Arc::new(RwLock::new(create_large_prompt_library(&temp_dir, 500).await?));
        
        let start = Instant::now();
        let mut handles = vec![];
        
        // Spawn many concurrent tasks
        for i in 0..50 {
            let library_clone = library.clone();
            let handle = tokio::spawn(async move {
                let mut local_metrics = PerformanceMetrics::default();
                
                // Each task performs multiple operations
                for j in 0..20 {
                    let prompt_name = format!("prompt_{:04}", (i * 20 + j) % 500);
                    
                    // List operation
                    let list_start = Instant::now();
                    let list_result = library_clone.read().await.list();
                    local_metrics.record_operation(list_start.elapsed(), list_result.is_ok());
                    
                    // Get operation
                    let get_start = Instant::now();
                    let get_result = library_clone.read().await.get(&prompt_name);
                    local_metrics.record_operation(get_start.elapsed(), get_result.is_ok());
                    
                    // Render operation
                    if let Ok(prompt) = get_result {
                        let mut args = HashMap::new();
                        args.insert("arg1".to_string(), format!("value_{}", i));
                        
                        let render_start = Instant::now();
                        let render_result = prompt.render(&args);
                        local_metrics.record_operation(render_start.elapsed(), render_result.is_ok());
                    }
                }
                
                local_metrics
            });
            handles.push(handle);
        }
        
        // Collect all metrics
        let mut total_metrics = PerformanceMetrics::default();
        for handle in handles {
            let metrics = handle.await?;
            total_metrics.total_operations += metrics.total_operations;
            total_metrics.total_duration += metrics.total_duration;
            total_metrics.errors += metrics.errors;
        }
        
        let total_duration = start.elapsed();
        
        println!("Concurrent access performance:");
        println!("  Total time: {:?}", total_duration);
        println!("  Total operations: {}", total_metrics.total_operations);
        println!("  Operations per second: {:.2}", 
                 total_metrics.total_operations as f64 / total_duration.as_secs_f64());
        println!("  Success rate: {:.2}%", total_metrics.success_rate());
        
        assert!(total_duration < Duration::from_secs(10));
        assert!(total_metrics.success_rate() > 99.0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_usage_with_large_library() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create library with many prompts
        let library = create_large_prompt_library(&temp_dir, 1000).await?;
        
        // Perform operations to ensure everything is loaded
        let prompts = library.list()?;
        assert_eq!(prompts.len(), 1000);
        
        // Get and render several prompts
        let mut args = HashMap::new();
        args.insert("arg1".to_string(), "test".to_string());
        
        for i in 0..100 {
            let prompt_name = format!("prompt_{:04}", i);
            let prompt = library.get(&prompt_name)?;
            let _ = prompt.render(&args)?;
        }
        
        // Note: In a real test, we'd measure actual memory usage
        // For now, we just ensure operations complete successfully
        
        Ok(())
    }

    #[tokio::test]
    async fn test_search_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let library = Arc::new(RwLock::new(create_large_prompt_library(&temp_dir, 1000).await?));
        
        let search_terms = vec!["performance", "test", "prompt", "load", "arg1"];
        let mut metrics = PerformanceMetrics::default();
        
        for term in search_terms {
            let start = Instant::now();
            
            // Simulate search by filtering prompts
            let result = {
                let lib = library.read().await;
                lib.list().map(|prompts| {
                    prompts.into_iter()
                        .filter(|p| {
                            p.name.contains(term) ||
                            p.description.as_ref().is_some_and(|d| d.contains(term)) ||
                            p.tags.iter().any(|t| t.contains(term))
                        })
                        .collect::<Vec<_>>()
                })
            };
            
            let duration = start.elapsed();
            metrics.record_operation(duration, result.is_ok());
            
            if let Ok(results) = result {
                println!("Search for '{}' found {} results in {:?}", term, results.len(), duration);
            }
        }
        
        println!("Search performance:");
        println!("  Average duration: {:?}", metrics.average_duration());
        println!("  Success rate: {:.2}%", metrics.success_rate());
        
        assert!(metrics.average_duration() < Duration::from_millis(50));
        assert_eq!(metrics.success_rate(), 100.0);
        
        Ok(())
    }
}