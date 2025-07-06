use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio::time::timeout;

#[tokio::test]
async fn test_mcp_notification_e2e() -> Result<()> {
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
        .args(&["run", "--bin", "swissarmyhammer", "--", "serve"])
        .env("HOME", temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");

    // Give server time to start
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut stdin = server_process.stdin.take().expect("Failed to get stdin");
    let stdout = server_process.stdout.take().expect("Failed to get stdout");
    let stderr = server_process.stderr.take().expect("Failed to get stderr");

    // Spawn stderr reader for debugging
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                eprintln!("SERVER STDERR: {}", line);
            }
        }
    });

    // Create reader for server responses
    let mut reader = BufReader::new(stdout);

    // Helper to send JSON-RPC request
    let send_request = |stdin: &mut std::process::ChildStdin, request: Value| {
        let request_str = serde_json::to_string(&request).unwrap();
        eprintln!("CLIENT -> SERVER: {}", request_str);
        writeln!(stdin, "{}", request_str).unwrap();
        stdin.flush().unwrap();
    };

    // Helper to read JSON-RPC response
    let read_response = |reader: &mut BufReader<std::process::ChildStdout>| -> Result<Value> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        eprintln!("SERVER -> CLIENT: {}", line.trim());
        Ok(serde_json::from_str(&line.trim())?)
    };

    // Step 1: Initialize the connection
    send_request(
        &mut stdin,
        json!({
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
        }),
    );

    let init_response = timeout(Duration::from_secs(5), async { read_response(&mut reader) })
        .await??;
    
    assert_eq!(init_response["jsonrpc"], "2.0");
    assert_eq!(init_response["id"], 1);
    assert!(init_response["result"].is_object());

    // Step 2: Send initialized notification
    send_request(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    );

    // Give server time to process and start file watching
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Step 3: List initial prompts
    send_request(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "prompts/list"
        }),
    );

    let list_response = timeout(Duration::from_secs(5), async { read_response(&mut reader) })
        .await??;
    
    let initial_prompts = list_response["result"]["prompts"].as_array().unwrap();
    let has_test_prompt = initial_prompts
        .iter()
        .any(|p| p["name"].as_str() == Some("test-prompt"));
    assert!(has_test_prompt, "Initial test prompt should be loaded");

    // Step 4: Set up monitoring for notification in the background
    let notification_received = Arc::new(Mutex::new(false));
    let notification_received_clone = notification_received.clone();
    
    // Spawn a task to continuously read from the server
    let reader_handle = tokio::spawn(async move {
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        eprintln!("SERVER OUTPUT: {}", trimmed);
                        if let Ok(msg) = serde_json::from_str::<Value>(trimmed) {
                            if msg["method"] == "notifications/prompts/list_changed" {
                                eprintln!("🎉 RECEIVED prompts/listChanged NOTIFICATION!");
                                *notification_received_clone.lock().await = true;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    });

    // Step 5: Wait a bit to ensure file watching is active
    eprintln!("Waiting for file watching to be fully active...");
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Step 6: Modify the prompt file
    eprintln!("Modifying prompt file at: {:?}", test_prompt_path);
    std::fs::write(
        &test_prompt_path,
        "---\ntitle: Test Prompt Updated\n---\nModified content!",
    )?;
    eprintln!("File modified!");

    // Step 7: Wait for notification with timeout
    eprintln!("Waiting for notification...");
    let notification_timeout = timeout(Duration::from_secs(5), async {
        while !*notification_received.lock().await {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await;

    // Clean up
    reader_handle.abort();
    server_process.kill().expect("Failed to kill server");
    let _ = server_process.wait();

    // Verify we received the notification
    match notification_timeout {
        Ok(_) => {
            println!("✅ Successfully received prompts/listChanged notification!");
            Ok(())
        }
        Err(_) => {
            panic!("❌ Timeout waiting for prompts/listChanged notification - notification system is not working!");
        }
    }
}

