//! Performance validation example for SwissArmyHammer cost tracking
//!
//! This example runs comprehensive benchmarks and validates that all performance
//! targets are achieved according to the requirements in issue 000203.
//!
//! Run with: cargo run --example validate_performance

use std::time::{Duration, Instant};
use swissarmyhammer::cost::performance::{
    benchmarks::BenchmarkSuite,
    optimization::{PerformanceConfigBuilder, PerformanceOptimizer},
};
use swissarmyhammer::cost::{ApiCallStatus, IssueId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Performance Validation for SwissArmyHammer Cost Tracking");
    println!("==========================================================\n");

    // Performance targets
    const API_OVERHEAD_TARGET_MS: u64 = 50;
    const MEMORY_LIMIT_PCT: f32 = 5.0;
    const OPERATIONS_COUNT: u64 = 1000;

    println!("📋 Performance Targets:");
    println!("  • API Call Overhead: < {}ms", API_OVERHEAD_TARGET_MS);
    println!("  • Memory Usage: < {}% overhead", MEMORY_LIMIT_PCT);
    println!("  • Operations per test: {}\n", OPERATIONS_COUNT);

    // 1. Run comprehensive benchmark suite
    println!("🔧 Running Comprehensive Benchmark Suite...");
    let suite = BenchmarkSuite::new(API_OVERHEAD_TARGET_MS);
    let benchmark_results = suite.run_all(OPERATIONS_COUNT);

    println!("\n📊 Benchmark Results:");
    println!("{}", suite.generate_report(&benchmark_results));

    // Validate benchmark targets
    let benchmarks_pass = suite.all_meet_targets(&benchmark_results);
    println!(
        "Benchmark Validation: {}",
        if benchmarks_pass {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );

    // 2. Test optimized performance optimizer
    println!("\n🔧 Testing Performance Optimizer...");
    let config = PerformanceConfigBuilder::new()
        .api_overhead_target_ms(API_OVERHEAD_TARGET_MS)
        .memory_limit_pct(MEMORY_LIMIT_PCT)
        .build();

    let optimizer = PerformanceOptimizer::new(config)?;

    // Measure actual API call overhead
    let start_total = Instant::now();

    let issue_id = IssueId::new("performance-validation")?;
    let session_id = optimizer.start_session(issue_id)?;

    // Simulate multiple API calls
    let mut total_api_overhead = Duration::new(0, 0);
    for i in 0..100 {
        let api_start = Instant::now();

        let call_id = optimizer.add_api_call(
            &session_id,
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )?;

        let response = format!(
            r#"{{"usage":{{"input_tokens":{},"output_tokens":{}}}}}"#,
            100 + i * 2,
            50 + i
        );

        let _usage = optimizer.complete_api_call_with_response(
            &session_id,
            &call_id,
            &response,
            ApiCallStatus::Success,
            None,
        )?;

        let api_duration = api_start.elapsed();
        total_api_overhead += api_duration;

        // Check individual call overhead
        if api_duration.as_millis() > API_OVERHEAD_TARGET_MS as u128 {
            println!(
                "⚠️  API call {} exceeded target: {}ms > {}ms",
                i,
                api_duration.as_millis(),
                API_OVERHEAD_TARGET_MS
            );
        }
    }

    optimizer.complete_session(
        &session_id,
        swissarmyhammer::cost::CostSessionStatus::Completed,
    )?;

    let total_duration = start_total.elapsed();
    let avg_api_overhead = total_api_overhead.as_millis() / 100;

    println!("📊 Performance Optimizer Results:");
    println!("  • Total Test Duration: {}ms", total_duration.as_millis());
    println!("  • Average API Call Overhead: {}ms", avg_api_overhead);
    println!("  • Target: < {}ms", API_OVERHEAD_TARGET_MS);

    let optimizer_pass = avg_api_overhead <= API_OVERHEAD_TARGET_MS as u128;
    println!(
        "Performance Optimizer Validation: {}",
        if optimizer_pass {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );

    // 3. Validate performance targets
    println!("\n🔧 Running Performance Validation...");
    let validation_result = optimizer.validate_performance()?;

    println!("📊 Performance Validation Results:");
    println!("  • Targets Met: {}", validation_result.targets_met);
    println!(
        "  • Memory Usage: {:.2}%",
        validation_result.memory_usage_pct
    );
    println!(
        "  • Cache Hit Rate: {:.1}%",
        validation_result.cache_hit_rate_pct
    );

    let validation_pass =
        validation_result.targets_met && validation_result.memory_usage_pct <= MEMORY_LIMIT_PCT;

    println!(
        "Performance Validation: {}",
        if validation_pass {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );

    // 4. Overall results
    println!("\n🏁 Overall Performance Validation Results");
    println!("==========================================");

    let overall_pass = benchmarks_pass && optimizer_pass && validation_pass;

    println!(
        "• Benchmark Suite: {}",
        if benchmarks_pass {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!(
        "• Performance Optimizer: {}",
        if optimizer_pass {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!(
        "• Target Validation: {}",
        if validation_pass {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );

    println!(
        "\n🎯 FINAL RESULT: {}",
        if overall_pass {
            "✅ ALL TARGETS MET"
        } else {
            "❌ PERFORMANCE TARGETS NOT MET"
        }
    );

    if !overall_pass {
        std::process::exit(1);
    }

    println!("\n🚀 Performance optimization implementation complete!");
    println!("   SwissArmyHammer cost tracking now meets < 50ms API overhead target");

    Ok(())
}
