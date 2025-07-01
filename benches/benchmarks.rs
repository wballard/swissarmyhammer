use criterion::{black_box, criterion_group, criterion_main, Criterion};
use swissarmyhammer::prompts::{PromptLoader, Prompt};
use swissarmyhammer::template::TemplateEngine;
use std::collections::HashMap;
use serde_json::Value;

fn benchmark_prompt_loading(c: &mut Criterion) {
    c.bench_function("load all prompts", |b| {
        b.iter(|| {
            let mut loader = PromptLoader::new();
            loader.load_all()
        });
    });
}

fn benchmark_template_processing(c: &mut Criterion) {
    let engine = TemplateEngine::new();
    let template = "Hello {{name}}, your score is {{score}} and you are {{status}}";
    let mut args = HashMap::new();
    args.insert("name".to_string(), Value::String("World".to_string()));
    args.insert("score".to_string(), Value::Number(42.into()));
    args.insert("status".to_string(), Value::String("active".to_string()));
    
    c.bench_function("template substitution", |b| {
        b.iter(|| {
            engine.process(black_box(template), black_box(&args))
        });
    });
}

fn benchmark_prompt_creation(c: &mut Criterion) {
    c.bench_function("create prompt", |b| {
        b.iter(|| {
            Prompt::new(
                black_box("test".to_string()),
                black_box("This is a test prompt".to_string()),
                black_box("test.md".to_string()),
            )
        });
    });
}

fn benchmark_prompt_storage(c: &mut Criterion) {
    let storage = swissarmyhammer::prompts::PromptStorage::new();
    let prompt = Prompt::new(
        "test".to_string(),
        "Test content".to_string(),
        "test.md".to_string(),
    );
    
    // Insert benchmark
    c.bench_function("prompt storage insert", |b| {
        b.iter(|| {
            storage.insert(black_box("test".to_string()), black_box(prompt.clone()));
        });
    });
    
    // Lookup benchmark
    storage.insert("lookup_test".to_string(), prompt.clone());
    c.bench_function("prompt storage get", |b| {
        b.iter(|| {
            storage.get(black_box("lookup_test"))
        });
    });
}

fn benchmark_template_validation(c: &mut Criterion) {
    let engine = TemplateEngine::new();
    let template = "Hello {{name}}, your score is {{score}}";
    let mut args = HashMap::new();
    args.insert("name".to_string(), Value::String("Test".to_string()));
    args.insert("score".to_string(), Value::Number(100.into()));
    
    let expected_args = vec![
        swissarmyhammer::template::TemplateArgument {
            name: "name".to_string(),
            description: Some("User name".to_string()),
            required: true,
            default_value: None,
        },
        swissarmyhammer::template::TemplateArgument {
            name: "score".to_string(),
            description: Some("User score".to_string()),
            required: true,
            default_value: None,
        },
    ];
    
    c.bench_function("template validation", |b| {
        b.iter(|| {
            engine.process_with_validation(
                black_box(template), 
                black_box(&args),
                black_box(&expected_args)
            )
        });
    });
}

criterion_group!(
    benches, 
    benchmark_prompt_loading,
    benchmark_template_processing,
    benchmark_prompt_creation,
    benchmark_prompt_storage,
    benchmark_template_validation
);
criterion_main!(benches);