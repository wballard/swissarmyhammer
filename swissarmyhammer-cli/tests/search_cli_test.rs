//! Integration tests for semantic search CLI commands

use anyhow::Result;
use assert_cmd::Command;

/// Test that the old --glob flag version no longer works (breaking change)
#[test]
fn test_search_index_old_glob_flag_rejected() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["search", "index", "--glob", "**/*.rs"])
        .output()?;

    assert!(
        !output.status.success(),
        "search index with --glob should now fail (breaking change)"
    );

    // The error should indicate that --glob is not a valid argument
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unexpected argument") || stderr.contains("found argument"),
        "should show error about unexpected --glob argument: {}",
        stderr
    );

    Ok(())
}

/// Test that the new positional glob argument version works
#[test]
fn test_search_index_positional_glob() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["search", "index", "**/*.rs"])
        .output()?;

    assert!(
        output.status.success(),
        "search index with positional glob should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Would index files matching: **/*.rs"),
        "should show glob pattern in output"
    );

    Ok(())
}

/// Test search index with force flag
#[test]
fn test_search_index_with_force() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["search", "index", "**/*.py", "--force"])
        .output()?;

    assert!(
        output.status.success(),
        "search index with force should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Would index files matching: **/*.py"),
        "should show glob pattern in output"
    );
    assert!(
        stdout.contains("Force re-indexing: enabled"),
        "should show force flag is enabled"
    );

    Ok(())
}

/// Test search query functionality
#[test]
fn test_search_query() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["search", "query", "error handling"])
        .output()?;

    assert!(output.status.success(), "search query should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Would search for: error handling"),
        "should show search query in output"
    );

    Ok(())
}

/// Test search help output
#[test]
fn test_search_help() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["search", "--help"])
        .output()?;

    assert!(output.status.success(), "search help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("semantic search"),
        "help should mention semantic search"
    );
    assert!(
        stdout.contains("index") && stdout.contains("query"),
        "help should mention index and query subcommands"
    );

    Ok(())
}

/// Test search index help shows correct usage
#[test]
fn test_search_index_help() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["search", "index", "--help"])
        .output()?;

    assert!(output.status.success(), "search index help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // After our changes, this should show positional argument syntax
    assert!(
        stdout.contains("<GLOB>") || stdout.contains("glob"),
        "help should show glob pattern parameter"
    );

    Ok(())
}
