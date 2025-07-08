//! Flow command implementation for executing workflows

use crate::cli::{FlowSubcommand, OutputFormat, PromptSource};
use std::collections::{HashMap, HashSet};
use std::future;
use std::time::Duration;
use swissarmyhammer::workflow::{
    WorkflowExecutor, WorkflowStorage, WorkflowRunId, WorkflowRunStatus, WorkflowName,
    Workflow, StateId, TransitionKey,
};
use swissarmyhammer::{Result, SwissArmyHammerError};
use tokio::signal;
use tokio::time::timeout;

/// Main entry point for flow command
pub async fn run_flow_command(subcommand: FlowSubcommand) -> Result<()> {
    match subcommand {
        FlowSubcommand::Run {
            workflow,
            vars,
            interactive,
            dry_run,
            test,
            timeout: timeout_str,
        } => {
            run_workflow_command(workflow, vars, interactive, dry_run, test, timeout_str).await
        }
        FlowSubcommand::Resume {
            run_id,
            interactive,
            timeout: timeout_str,
        } => resume_workflow_command(run_id, interactive, timeout_str).await,
        FlowSubcommand::List {
            format,
            verbose,
            source,
        } => list_workflows_command(format, verbose, source).await,
        FlowSubcommand::Status {
            run_id,
            format,
            watch,
        } => status_workflow_command(run_id, format, watch).await,
        FlowSubcommand::Logs {
            run_id,
            follow,
            tail,
            level,
        } => logs_workflow_command(run_id, follow, tail, level).await,
    }
}

/// Execute a workflow
async fn run_workflow_command(
    workflow_name: String,
    vars: Vec<String>,
    interactive: bool,
    dry_run: bool,
    test_mode: bool,
    timeout_str: Option<String>,
) -> Result<()> {
    let mut storage = WorkflowStorage::file_system()?;
    let workflow_name_typed = WorkflowName::new(&workflow_name);
    
    // Get the workflow
    let workflow = storage.get_workflow(&workflow_name_typed)?;
    
    // Parse variables
    let mut variables = HashMap::new();
    for var in vars {
        let parts: Vec<&str> = var.splitn(2, '=').collect();
        if parts.len() == 2 {
            variables.insert(parts[0].to_string(), serde_json::Value::String(parts[1].to_string()));
        } else {
            return Err(SwissArmyHammerError::Other(
                format!("Invalid variable format: '{}'. Use key=value format.", var)
            ));
        }
    }
    
    // Parse timeout
    let timeout_duration = if let Some(timeout_str) = timeout_str {
        Some(parse_duration(&timeout_str)?)
    } else {
        None
    };
    
    if dry_run {
        println!("🔍 Dry run mode - showing execution plan:");
        println!("📋 Workflow: {}", workflow.name);
        println!("🏁 Initial state: {}", workflow.initial_state);
        println!("🔧 Variables: {:?}", variables);
        if let Some(timeout) = timeout_duration {
            println!("⏱️  Timeout: {:?}", timeout);
        }
        println!("📊 States: {}", workflow.states.len());
        println!("🔄 Transitions: {}", workflow.transitions.len());
        
        // Show workflow structure
        println!("\n📈 Workflow structure:");
        for (state_id, state) in &workflow.states {
            println!("  {} - {} {}", 
                state_id,
                state.description,
                if state.is_terminal { "(terminal)" } else { "" }
            );
        }
        
        return Ok(());
    }
    
    if test_mode {
        println!("🧪 Test mode - executing workflow with mocked actions:");
        println!("📋 Workflow: {}", workflow.name);
        println!("🏁 Initial state: {}", workflow.initial_state);
        println!("🔧 Variables: {:?}", variables);
        if let Some(timeout) = timeout_duration {
            println!("⏱️  Timeout: {:?}", timeout);
        }
        
        // Execute in test mode with coverage tracking
        let coverage = execute_workflow_test_mode(workflow, variables, timeout_duration).await?;
        
        // Generate coverage report
        println!("\n📊 Coverage Report:");
        println!("  States visited: {}/{} ({:.1}%)", 
            coverage.visited_states.len(), 
            coverage.total_states,
            (coverage.visited_states.len() as f64 / coverage.total_states as f64) * 100.0
        );
        println!("  Transitions used: {}/{} ({:.1}%)",
            coverage.visited_transitions.len(),
            coverage.total_transitions,
            (coverage.visited_transitions.len() as f64 / coverage.total_transitions as f64) * 100.0
        );
        
        // Show unvisited states
        if !coverage.unvisited_states.is_empty() {
            println!("\n❌ Unvisited states:");
            for state in &coverage.unvisited_states {
                println!("  - {}", state);
            }
        }
        
        // Show unvisited transitions
        if !coverage.unvisited_transitions.is_empty() {
            println!("\n❌ Unvisited transitions:");
            for transition in &coverage.unvisited_transitions {
                println!("  - {}", transition);
            }
        }
        
        if coverage.visited_states.len() == coverage.total_states {
            println!("\n✅ Full state coverage achieved!");
        }
        if coverage.visited_transitions.len() == coverage.total_transitions {
            println!("✅ Full transition coverage achieved!");
        }
        
        return Ok(());
    }
    
    println!("🚀 Starting workflow: {}", workflow.name);
    
    // Create executor
    let mut executor = WorkflowExecutor::new();
    
    // Create workflow run
    let mut run = executor.start_workflow(workflow).await
        .map_err(|e| SwissArmyHammerError::Other(format!("Failed to start workflow: {}", e)))?;
    
    // Set initial variables
    run.context.extend(variables);
    
    // Setup signal handling for graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);
    let shutdown_tx_clone = shutdown_tx.clone();
    
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        let _ = shutdown_tx_clone.send(()).await;
    });
    
    // Execute workflow with timeout and signal handling
    let execution_result = if let Some(timeout_duration) = timeout_duration {
        tokio::select! {
            result = execute_workflow_with_progress(&mut executor, &mut run, interactive) => result,
            _ = timeout(timeout_duration, future::pending::<()>()) => {
                println!("⏰ Workflow execution timed out");
                run.status = WorkflowRunStatus::Cancelled;
                Ok(())
            },
            _ = shutdown_rx.recv() => {
                println!("\n🛑 Workflow execution interrupted");
                run.status = WorkflowRunStatus::Cancelled;
                Ok(())
            }
        }
    } else {
        tokio::select! {
            result = execute_workflow_with_progress(&mut executor, &mut run, interactive) => result,
            _ = shutdown_rx.recv() => {
                println!("\n🛑 Workflow execution interrupted");
                run.status = WorkflowRunStatus::Cancelled;
                Ok(())
            }
        }
    };
    
    // Store the run
    storage.store_run(&run)?;
    
    match execution_result {
        Ok(_) => {
            match run.status {
                WorkflowRunStatus::Completed => {
                    println!("✅ Workflow completed successfully");
                    println!("🆔 Run ID: {}", workflow_run_id_to_string(&run.id));
                }
                WorkflowRunStatus::Failed => {
                    println!("❌ Workflow failed");
                    println!("🆔 Run ID: {}", workflow_run_id_to_string(&run.id));
                }
                WorkflowRunStatus::Cancelled => {
                    println!("🚫 Workflow cancelled");
                    println!("🆔 Run ID: {}", workflow_run_id_to_string(&run.id));
                }
                _ => {
                    println!("⏸️  Workflow paused");
                    println!("🆔 Run ID: {}", workflow_run_id_to_string(&run.id));
                }
            }
        }
        Err(e) => {
            println!("❌ Workflow execution failed: {}", e);
            run.fail();
            storage.store_run(&run)?;
        }
    }
    
    Ok(())
}

/// Resume a workflow run
async fn resume_workflow_command(
    run_id: String,
    interactive: bool,
    timeout_str: Option<String>,
) -> Result<()> {
    let mut storage = WorkflowStorage::file_system()?;
    
    // Parse run ID
    let run_id_typed = parse_workflow_run_id(&run_id)?;
    
    // Get the run
    let mut run = storage.get_run(&run_id_typed)?;
    
    // Check if run can be resumed
    if run.status == WorkflowRunStatus::Completed {
        println!("❌ Cannot resume completed workflow");
        return Ok(());
    }
    
    if run.status == WorkflowRunStatus::Failed {
        println!("❌ Cannot resume failed workflow");
        return Ok(());
    }
    
    // Parse timeout
    let timeout_duration = if let Some(timeout_str) = timeout_str {
        Some(parse_duration(&timeout_str)?)
    } else {
        None
    };
    
    println!("🔄 Resuming workflow: {}", run.workflow.name);
    println!("🔄 From state: {}", run.current_state);
    
    // Create executor
    let mut executor = WorkflowExecutor::new();
    
    // Setup signal handling for graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);
    let shutdown_tx_clone = shutdown_tx.clone();
    
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        let _ = shutdown_tx_clone.send(()).await;
    });
    
    // Resume workflow execution
    let execution_result = if let Some(timeout_duration) = timeout_duration {
        tokio::select! {
            result = execute_workflow_with_progress(&mut executor, &mut run, interactive) => result,
            _ = timeout(timeout_duration, future::pending::<()>()) => {
                println!("⏰ Workflow execution timed out");
                run.status = WorkflowRunStatus::Cancelled;
                Ok(())
            },
            _ = shutdown_rx.recv() => {
                println!("\n🛑 Workflow execution interrupted");
                run.status = WorkflowRunStatus::Cancelled;
                Ok(())
            }
        }
    } else {
        tokio::select! {
            result = execute_workflow_with_progress(&mut executor, &mut run, interactive) => result,
            _ = shutdown_rx.recv() => {
                println!("\n🛑 Workflow execution interrupted");
                run.status = WorkflowRunStatus::Cancelled;
                Ok(())
            }
        }
    };
    
    // Store the updated run
    storage.store_run(&run)?;
    
    match execution_result {
        Ok(_) => {
            match run.status {
                WorkflowRunStatus::Completed => {
                    println!("✅ Workflow resumed and completed successfully");
                }
                WorkflowRunStatus::Failed => {
                    println!("❌ Workflow resumed but failed");
                }
                WorkflowRunStatus::Cancelled => {
                    println!("🚫 Workflow resumed but was cancelled");
                }
                _ => {
                    println!("⏸️  Workflow resumed and paused");
                }
            }
        }
        Err(e) => {
            println!("❌ Workflow resume failed: {}", e);
            run.fail();
            storage.store_run(&run)?;
        }
    }
    
    Ok(())
}

/// List available workflows
async fn list_workflows_command(
    format: OutputFormat,
    verbose: bool,
    _source: Option<PromptSource>,
) -> Result<()> {
    let storage = WorkflowStorage::file_system()?;
    let workflows = storage.list_workflows()?;
    
    match format {
        OutputFormat::Table => {
            if workflows.is_empty() {
                println!("No workflows found.");
                return Ok(());
            }
            
            if verbose {
                println!("{:<20} {:<30} {:<10} {:<8} {:<12}", "NAME", "DESCRIPTION", "STATES", "TERMINAL", "TRANSITIONS");
                println!("{}", "-".repeat(90));
                for workflow in workflows {
                    let terminal_count = workflow.states.values().filter(|s| s.is_terminal).count();
                    println!("{:<20} {:<30} {:<10} {:<8} {:<12}", 
                        workflow.name.as_str(),
                        workflow.description.chars().take(30).collect::<String>(),
                        workflow.states.len(),
                        terminal_count,
                        workflow.transitions.len()
                    );
                }
            } else {
                println!("{:<20} {:<50}", "NAME", "DESCRIPTION");
                println!("{}", "-".repeat(70));
                for workflow in workflows {
                    println!("{:<20} {:<50}", 
                        workflow.name.as_str(),
                        workflow.description.chars().take(50).collect::<String>()
                    );
                }
            }
        }
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&workflows)?;
            println!("{}", json_output);
        }
        OutputFormat::Yaml => {
            let yaml_output = serde_yaml::to_string(&workflows)?;
            println!("{}", yaml_output);
        }
    }
    
    Ok(())
}

/// Check workflow run status
async fn status_workflow_command(
    run_id: String,
    format: OutputFormat,
    watch: bool,
) -> Result<()> {
    let storage = WorkflowStorage::file_system()?;
    
    // Parse run ID
    let run_id_typed = parse_workflow_run_id(&run_id)?;
    
    if watch {
        println!("👁️  Watching workflow run status (Press Ctrl+C to stop)...");
        
        loop {
            match storage.get_run(&run_id_typed) {
                Ok(run) => {
                    print_run_status(&run, &format)?;
                    
                    // Exit if workflow is completed
                    if run.status == WorkflowRunStatus::Completed 
                        || run.status == WorkflowRunStatus::Failed 
                        || run.status == WorkflowRunStatus::Cancelled {
                        break;
                    }
                }
                Err(e) => {
                    println!("❌ Error getting run status: {}", e);
                    break;
                }
            }
            
            // Check for Ctrl+C
            if (tokio::time::timeout(Duration::from_secs(2), signal::ctrl_c()).await).is_ok() {
                println!("\n🛑 Stopped watching");
                break;
            }
        }
    } else {
        let run = storage.get_run(&run_id_typed)?;
        print_run_status(&run, &format)?;
    }
    
    Ok(())
}

/// View workflow run logs
async fn logs_workflow_command(
    run_id: String,
    follow: bool,
    tail: Option<usize>,
    level: Option<String>,
) -> Result<()> {
    let storage = WorkflowStorage::file_system()?;
    
    // Parse run ID
    let run_id_typed = parse_workflow_run_id(&run_id)?;
    
    let run = storage.get_run(&run_id_typed)?;
    
    if follow {
        println!("📄 Following logs for run {} (Press Ctrl+C to stop)...", run_id);
        
        loop {
            let updated_run = storage.get_run(&run_id_typed)?;
            print_run_logs(&updated_run, tail, &level)?;
            
            // Exit if workflow is completed
            if updated_run.status == WorkflowRunStatus::Completed 
                || updated_run.status == WorkflowRunStatus::Failed 
                || updated_run.status == WorkflowRunStatus::Cancelled {
                break;
            }
            
            // Check for Ctrl+C
            if (tokio::time::timeout(Duration::from_secs(1), signal::ctrl_c()).await).is_ok() {
                println!("\n🛑 Stopped following logs");
                break;
            }
        }
    } else {
        print_run_logs(&run, tail, &level)?;
    }
    
    Ok(())
}

/// Execute workflow with progress display
async fn execute_workflow_with_progress(
    executor: &mut WorkflowExecutor,
    run: &mut swissarmyhammer::workflow::WorkflowRun,
    interactive: bool,
) -> Result<()> {
    if interactive {
        println!("🎯 Interactive mode - press Enter to continue at each step");
        
        while run.status == WorkflowRunStatus::Running {
            println!("📍 Current state: {} - {}", 
                run.current_state,
                run.workflow.states.get(&run.current_state)
                    .map(|s| s.description.as_str())
                    .unwrap_or("Unknown state")
            );
            
            println!("Press Enter to execute this step...");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            // Execute single step
            executor.execute_state(run).await
                .map_err(|e| SwissArmyHammerError::Other(format!("Failed to execute state: {}", e)))?;
            
            println!("✅ Step completed");
            
            if run.status != WorkflowRunStatus::Running {
                break;
            }
        }
    } else {
        // Non-interactive execution
        executor.execute_state(run).await
            .map_err(|e| SwissArmyHammerError::Other(format!("Failed to execute workflow: {}", e)))?;
    }
    
    Ok(())
}

/// Print run status
fn print_run_status(
    run: &swissarmyhammer::workflow::WorkflowRun,
    format: &OutputFormat,
) -> Result<()> {
    match format {
        OutputFormat::Table => {
            println!("🆔 Run ID: {}", workflow_run_id_to_string(&run.id));
            println!("📋 Workflow: {}", run.workflow.name);
            println!("📊 Status: {:?}", run.status);
            println!("📍 Current State: {}", run.current_state);
            println!("🕐 Started: {}", run.started_at.format("%Y-%m-%d %H:%M:%S UTC"));
            if let Some(completed_at) = run.completed_at {
                println!("🏁 Completed: {}", completed_at.format("%Y-%m-%d %H:%M:%S UTC"));
            }
            println!("📈 History: {} transitions", run.history.len());
            println!("🔧 Variables: {} items", run.context.len());
        }
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&run)?;
            println!("{}", json_output);
        }
        OutputFormat::Yaml => {
            let yaml_output = serde_yaml::to_string(&run)?;
            println!("{}", yaml_output);
        }
    }
    
    Ok(())
}

/// Print run logs
fn print_run_logs(
    run: &swissarmyhammer::workflow::WorkflowRun,
    tail: Option<usize>,
    _level: &Option<String>,
) -> Result<()> {
    println!("📄 Logs for run {}", workflow_run_id_to_string(&run.id));
    println!("📋 Workflow: {}", run.workflow.name);
    println!();
    
    // Show execution history as logs
    let history = if let Some(tail_count) = tail {
        if run.history.len() > tail_count {
            &run.history[run.history.len() - tail_count..]
        } else {
            &run.history
        }
    } else {
        &run.history
    };
    
    for (state_id, timestamp) in history {
        let state_desc = run.workflow.states.get(state_id)
            .map(|s| s.description.as_str())
            .unwrap_or("Unknown state");
        
        println!("{} 📍 Transitioned to: {} - {}", 
            timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            state_id,
            state_desc
        );
    }
    
    // Show current context/variables
    if !run.context.is_empty() {
        println!("\n🔧 Current Variables:");
        for (key, value) in &run.context {
            println!("  {} = {}", key, value);
        }
    }
    
    Ok(())
}

/// Parse duration string (e.g., "30s", "5m", "1h")
fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        return Err(SwissArmyHammerError::Other(
            "Empty duration string".to_string()
        ));
    }
    
    let (value_str, unit) = if let Some(stripped) = s.strip_suffix('s') {
        (stripped, "s")
    } else if let Some(stripped) = s.strip_suffix('m') {
        (stripped, "m")
    } else if let Some(stripped) = s.strip_suffix('h') {
        (stripped, "h")
    } else {
        (s, "s") // Default to seconds
    };
    
    let value: u64 = value_str.parse()
        .map_err(|_| SwissArmyHammerError::Other(
            format!("Invalid duration value: {}", value_str)
        ))?;
    
    let duration = match unit {
        "s" => Duration::from_secs(value),
        "m" => Duration::from_secs(value * 60),
        "h" => Duration::from_secs(value * 3600),
        _ => return Err(SwissArmyHammerError::Other(
            format!("Invalid duration unit: {}", unit)
        )),
    };
    
    Ok(duration)
}

/// Helper to parse WorkflowRunId from string
fn parse_workflow_run_id(s: &str) -> Result<WorkflowRunId> {
    WorkflowRunId::parse(s)
        .map_err(SwissArmyHammerError::Other)
}

/// Helper to convert WorkflowRunId to string
fn workflow_run_id_to_string(id: &WorkflowRunId) -> String {
    id.to_string()
}

/// Coverage tracking for workflow test execution
struct WorkflowCoverage {
    visited_states: HashSet<StateId>,
    visited_transitions: HashSet<TransitionKey>,
    total_states: usize,
    total_transitions: usize,
    unvisited_states: Vec<StateId>,
    unvisited_transitions: Vec<TransitionKey>,
}

/// Execute workflow in test mode with mocked actions
async fn execute_workflow_test_mode(
    workflow: Workflow,
    initial_variables: HashMap<String, serde_json::Value>,
    timeout_duration: Option<Duration>,
) -> Result<WorkflowCoverage> {
    use swissarmyhammer::workflow::{WorkflowRun, ConditionType};
    
    let mut coverage = WorkflowCoverage {
        visited_states: HashSet::new(),
        visited_transitions: HashSet::new(),
        total_states: workflow.states.len(),
        total_transitions: workflow.transitions.len(),
        unvisited_states: Vec::new(),
        unvisited_transitions: Vec::new(),
    };
    
    // Create a mock workflow run
    let mut run = WorkflowRun::new(workflow.clone());
    run.context.extend(initial_variables);
    
    // Track visited states and transitions
    let mut current_state = workflow.initial_state.clone();
    coverage.visited_states.insert(current_state.clone());
    
    println!("\n▶️  Starting test execution...");
    
    // Simple execution loop - try to visit all states
    let start_time = std::time::Instant::now();
    let timeout = timeout_duration.unwrap_or(Duration::from_secs(60));
    
    while !workflow.states.get(&current_state).map(|s| s.is_terminal).unwrap_or(false) {
        if start_time.elapsed() > timeout {
            println!("⏰ Test execution timed out");
            break;
        }
        
        // Find transitions from current state
        let available_transitions: Vec<_> = workflow.transitions.iter()
            .filter(|t| t.from_state == current_state)
            .collect();
        
        if available_transitions.is_empty() {
            println!("⚠️  No transitions from state: {}", current_state);
            break;
        }
        
        // Try each transition, preferring unvisited ones
        let mut transition_taken = false;
        for transition in &available_transitions {
            let transition_key = TransitionKey::from_refs(&transition.from_state, &transition.to_state);
            
            // Check if we should take this transition based on condition
            let should_take = match &transition.condition.condition_type {
                ConditionType::Always => true,
                ConditionType::Never => false,
                ConditionType::OnSuccess => true, // Mock success
                ConditionType::OnFailure => false,
                ConditionType::Custom => true, // Always true in test mode
            };
            
            if should_take && (!coverage.visited_transitions.contains(&transition_key) || available_transitions.len() == 1) {
                // Mock action execution
                if let Some(action) = &transition.action {
                    println!("🎭 Mock executing: {}", action);
                    // Set mock result in context
                    run.context.insert("result".to_string(), serde_json::json!({
                        "success": true,
                        "output": "Mock output"
                    }));
                }
                
                // Take the transition
                println!("➡️  {}", transition_key);
                coverage.visited_transitions.insert(transition_key);
                coverage.visited_states.insert(transition.to_state.clone());
                current_state = transition.to_state.clone();
                transition_taken = true;
                break;
            }
        }
        
        if !transition_taken {
            // All transitions have been visited or conditions not met
            println!("🔚 All transitions from {} have been explored", current_state);
            break;
        }
    }
    
    // Calculate unvisited states and transitions
    for state_id in workflow.states.keys() {
        if !coverage.visited_states.contains(state_id) {
            coverage.unvisited_states.push(state_id.clone());
        }
    }
    
    for transition in &workflow.transitions {
        let transition_key = TransitionKey::from_refs(&transition.from_state, &transition.to_state);
        if !coverage.visited_transitions.contains(&transition_key) {
            coverage.unvisited_transitions.push(transition_key);
        }
    }
    
    println!("\n✅ Test execution completed");
    
    Ok(coverage)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("60").unwrap(), Duration::from_secs(60));
        
        assert!(parse_duration("").is_err());
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("10x").is_err());
    }
    
    #[test]
    fn test_workflow_run_id_helpers() {
        let id = WorkflowRunId::new();
        let id_str = workflow_run_id_to_string(&id);
        let parsed_id = parse_workflow_run_id(&id_str).unwrap();
        
        // Test round-trip conversion works correctly
        assert_eq!(id, parsed_id);
        assert_eq!(id_str, workflow_run_id_to_string(&parsed_id));
    }

    #[test]
    fn test_workflow_run_id_parse_error() {
        let invalid_id = "invalid-ulid-string";
        let result = parse_workflow_run_id(invalid_id);
        assert!(result.is_err());
    }
}