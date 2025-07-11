use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use swissarmyhammer::workflow::{
    ConditionType, MermaidParser, State, StateId, StateType, Transition, TransitionCondition,
    Workflow, WorkflowCacheManager, WorkflowExecutor, WorkflowName, WorkflowRun, WorkflowStorage,
};

// Workflow performance benchmarks
fn create_simple_workflow() -> Workflow {
    let mut workflow = Workflow::new(
        WorkflowName::new("benchmark_workflow"),
        "A simple workflow for benchmarking".to_string(),
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
        id: StateId::new("process"),
        description: "Process state".to_string(),
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

    workflow.add_transition(Transition {
        from_state: StateId::new("start"),
        to_state: StateId::new("process"),
        condition: TransitionCondition {
            condition_type: ConditionType::Always,
            expression: None,
        },
        action: None,
        metadata: HashMap::new(),
    });

    workflow.add_transition(Transition {
        from_state: StateId::new("process"),
        to_state: StateId::new("end"),
        condition: TransitionCondition {
            condition_type: ConditionType::Always,
            expression: None,
        },
        action: None,
        metadata: HashMap::new(),
    });

    workflow
}

fn create_complex_workflow() -> Workflow {
    let mut workflow = Workflow::new(
        WorkflowName::new("complex_benchmark_workflow"),
        "A complex workflow for benchmarking".to_string(),
        StateId::new("start"),
    );

    // Create a workflow with 100 states
    for i in 0..100 {
        workflow.add_state(State {
            id: StateId::new(format!("state_{}", i)),
            description: format!("State {}", i),
            state_type: StateType::Normal,
            is_terminal: i == 99,
            allows_parallel: false,
            metadata: HashMap::new(),
        });

        if i > 0 {
            workflow.add_transition(Transition {
                from_state: StateId::new(format!("state_{}", i - 1)),
                to_state: StateId::new(format!("state_{}", i)),
                condition: TransitionCondition {
                    condition_type: ConditionType::Always,
                    expression: None,
                },
                action: None,
                metadata: HashMap::new(),
            });
        }
    }

    // Rename the first state
    workflow.states.remove(&StateId::new("state_0"));
    workflow.add_state(State {
        id: StateId::new("start"),
        description: "Start state".to_string(),
        state_type: StateType::Normal,
        is_terminal: false,
        allows_parallel: false,
        metadata: HashMap::new(),
    });

    workflow.add_transition(Transition {
        from_state: StateId::new("start"),
        to_state: StateId::new("state_1"),
        condition: TransitionCondition {
            condition_type: ConditionType::Always,
            expression: None,
        },
        action: None,
        metadata: HashMap::new(),
    });

    workflow
}

fn benchmark_workflow_parsing(c: &mut Criterion) {
    let simple_mermaid = r#"
        stateDiagram-v2
            [*] --> Processing
            Processing --> Complete
            Complete --> [*]
    "#;

    let complex_mermaid = r#"
        stateDiagram-v2
            [*] --> Init
            Init --> Validate
            Validate --> Process: valid
            Validate --> Error: invalid
            Process --> Transform
            Transform --> Review
            Review --> Approve: approved
            Review --> Reject: rejected
            Approve --> Deploy
            Deploy --> Monitor
            Monitor --> Complete
            Complete --> [*]
            Error --> [*]
            Reject --> [*]
    "#;

    c.bench_function("parse simple workflow", |b| {
        b.iter(|| MermaidParser::parse(black_box(simple_mermaid), black_box("simple_workflow")));
    });

    c.bench_function("parse complex workflow", |b| {
        b.iter(|| MermaidParser::parse(black_box(complex_mermaid), black_box("complex_workflow")));
    });
}

fn benchmark_workflow_execution(c: &mut Criterion) {
    let simple_workflow = create_simple_workflow();
    let complex_workflow = create_complex_workflow();

    c.bench_function("execute simple workflow", |b| {
        b.iter(|| {
            let mut executor = WorkflowExecutor::new();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                executor
                    .start_workflow(black_box(simple_workflow.clone()))
            })
        });
    });

    c.bench_function("execute complex workflow", |b| {
        b.iter(|| {
            let mut executor = WorkflowExecutor::new();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                executor
                    .start_workflow(black_box(complex_workflow.clone()))
            })
        });
    });
}

fn benchmark_workflow_cache(c: &mut Criterion) {
    let cache_manager = WorkflowCacheManager::new();
    let workflow = std::sync::Arc::new(create_simple_workflow());

    c.bench_function("workflow cache put", |b| {
        b.iter(|| {
            cache_manager.workflow_cache.put(
                black_box(WorkflowName::new("test_workflow")),
                black_box(workflow.clone()),
            );
        });
    });

    // Pre-populate cache for get benchmark
    cache_manager
        .workflow_cache
        .put(WorkflowName::new("cached_workflow"), workflow.clone());

    c.bench_function("workflow cache get (hit)", |b| {
        b.iter(|| {
            cache_manager
                .workflow_cache
                .get(black_box(&WorkflowName::new("cached_workflow")))
        });
    });

    c.bench_function("workflow cache get (miss)", |b| {
        b.iter(|| {
            cache_manager
                .workflow_cache
                .get(black_box(&WorkflowName::new("missing_workflow")))
        });
    });
}

fn benchmark_workflow_storage(c: &mut Criterion) {
    let mut storage = WorkflowStorage::memory();
    let workflow = create_simple_workflow();

    c.bench_function("workflow storage store", |b| {
        b.iter(|| storage.store_workflow(black_box(workflow.clone())));
    });

    // Pre-populate storage for get benchmark
    storage.store_workflow(workflow.clone()).unwrap();

    c.bench_function("workflow storage get", |b| {
        b.iter(|| storage.get_workflow(black_box(&workflow.name)));
    });

    c.bench_function("workflow storage list", |b| {
        b.iter(|| storage.list_workflows());
    });
}

fn benchmark_workflow_state_transitions(c: &mut Criterion) {
    let workflow = create_simple_workflow();
    let mut executor = WorkflowExecutor::new();

    c.bench_function("single state transition", |b| {
        b.iter(|| {
            let mut run = WorkflowRun::new(black_box(workflow.clone()));
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async { executor.execute_single_state(&mut run).await })
        });
    });

    c.bench_function("cached transition lookup", |b| {
        b.iter(|| {
            executor.get_cached_transition_path(
                black_box(&StateId::new("start")),
                black_box(&StateId::new("process")),
            )
        });
    });
}

fn benchmark_cel_program_cache(c: &mut Criterion) {
    let cache_manager = WorkflowCacheManager::new();
    let expression = "input.value > 0 && input.status == 'active'";

    c.bench_function("CEL program compile and cache", |b| {
        b.iter(|| {
            cache_manager
                .cel_cache
                .get_or_compile(black_box(expression))
        });
    });

    // Pre-compile for cache hit benchmark
    cache_manager.cel_cache.get_or_compile(expression).unwrap();

    c.bench_function("CEL program cache hit", |b| {
        b.iter(|| cache_manager.cel_cache.get(black_box(expression)));
    });
}

fn benchmark_workflow_scalability(c: &mut Criterion) {
    let workflow_sizes = vec![10, 50, 100, 500];
    let mut group = c.benchmark_group("workflow_scalability");

    for size in workflow_sizes {
        group.bench_function(format!("workflow_size_{}", size), |b| {
            b.iter(|| {
                let mut workflow = Workflow::new(
                    WorkflowName::new(format!("scale_test_{}", size)),
                    format!("Scalability test with {} states", size),
                    StateId::new("start"),
                );

                // Create states and transitions
                for i in 0..size {
                    workflow.add_state(State {
                        id: StateId::new(format!("state_{}", i)),
                        description: format!("State {}", i),
                        state_type: StateType::Normal,
                        is_terminal: i == size - 1,
                        allows_parallel: false,
                        metadata: HashMap::new(),
                    });

                    if i > 0 {
                        workflow.add_transition(Transition {
                            from_state: StateId::new(format!("state_{}", i - 1)),
                            to_state: StateId::new(format!("state_{}", i)),
                            condition: TransitionCondition {
                                condition_type: ConditionType::Always,
                                expression: None,
                            },
                            action: None,
                            metadata: HashMap::new(),
                        });
                    }
                }

                // Rename first state to "start"
                workflow.states.remove(&StateId::new("state_0"));
                workflow.add_state(State {
                    id: StateId::new("start"),
                    description: "Start state".to_string(),
                    state_type: StateType::Normal,
                    is_terminal: false,
                    allows_parallel: false,
                    metadata: HashMap::new(),
                });

                if size > 1 {
                    workflow.add_transition(Transition {
                        from_state: StateId::new("start"),
                        to_state: StateId::new("state_1"),
                        condition: TransitionCondition {
                            condition_type: ConditionType::Always,
                            expression: None,
                        },
                        action: None,
                        metadata: HashMap::new(),
                    });
                }

                // Validate the workflow
                workflow.validate()
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_workflow_parsing,
    benchmark_workflow_execution,
    benchmark_workflow_cache,
    benchmark_workflow_storage,
    benchmark_workflow_state_transitions,
    benchmark_cel_program_cache,
    benchmark_workflow_scalability
);
criterion_main!(benches);
