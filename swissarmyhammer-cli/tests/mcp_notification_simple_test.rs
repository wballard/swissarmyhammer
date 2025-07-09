use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_mcp_notification_simple() -> Result<()> {
    // Create a temporary directory for test prompts
    let temp_dir = TempDir::new()?;
    let prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
    std::fs::create_dir_all(&prompts_dir)?;

    // Create an initial prompt
    let test_prompt_path = prompts_dir.join("test-prompt.md");
    std::fs::write(
        &test_prompt_path,
        "---\ntitle: Test Prompt\n---\nOriginal content",
    )?;

    // Start the MCP server with HOME set to temp dir
    let mut server_process = Command::new("cargo")
        .args(["run", "--bin", "swissarmyhammer", "--", "serve"])
        .env("HOME", temp_dir.path())
        .env("RUST_LOG", "debug")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");

    // Give server time to start
    std::thread::sleep(Duration::from_secs(2));

    let mut stdin = server_process.stdin.take().expect("Failed to get stdin");
    let stdout = server_process.stdout.take().expect("Failed to get stdout");
    let stderr = server_process.stderr.take().expect("Failed to get stderr");

    // Spawn thread to read stderr
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            eprintln!("STDERR: {}", line);
        }
    });

    // Initialize
    let init_req = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "prompts": {
                    "listChanged": true
                }
            },
            "clientInfo": {
                "name": "test-client",
                "version": "1.0"
            }
        }
    });

    writeln!(stdin, "{}", serde_json::to_string(&init_req)?)?;
    stdin.flush()?;

    let mut reader = BufReader::new(stdout);
    let mut response = String::new();
    reader.read_line(&mut response)?;
    println!("Init response: {}", response);

    // Send initialized
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    writeln!(stdin, "{}", serde_json::to_string(&initialized)?)?;
    stdin.flush()?;

    // Wait for file watching to start
    std::thread::sleep(Duration::from_secs(3));

    // Spawn thread to monitor for notifications
    let reader_thread = std::thread::spawn(move || {
        let mut notification_found = false;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        println!("SERVER: {}", trimmed);
                        if let Ok(msg) = serde_json::from_str::<Value>(trimmed) {
                            if msg["method"] == "notifications/prompts/list_changed" {
                                println!("🎉 NOTIFICATION RECEIVED!");
                                notification_found = true;
                                break;
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
        notification_found
    });

    // Modify the file
    println!("Modifying file...");
    std::fs::write(
        &test_prompt_path,
        "---\ntitle: Test Prompt Updated\n---\nNew content!",
    )?;

    // Wait for notification
    std::thread::sleep(Duration::from_secs(5));

    // Check if we got the notification
    server_process.kill()?;
    server_process.wait()?; // Wait for process to fully terminate
    let notification_found = reader_thread.join().unwrap();

    assert!(
        notification_found,
        "Should have received prompts/listChanged notification"
    );
    println!("✅ Test passed!");

    Ok(())
}
