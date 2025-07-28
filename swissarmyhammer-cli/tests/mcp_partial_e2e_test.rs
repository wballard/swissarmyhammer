use serde_json::{json, Value};
use serial_test::serial;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::{sleep, timeout};

mod test_utils;
use test_utils::ProcessGuard;

/// Wait for the server to be ready by testing actual JSON-RPC communication
async fn wait_for_server_ready(
    stdin: &mut std::process::ChildStdin,
    reader: &mut BufReader<std::process::ChildStdout>,
) -> Result<(), Box<dyn std::error::Error>> {
    const MAX_ATTEMPTS: u32 = 10; // 1 second with 100ms intervals
    const RETRY_DELAY: Duration = Duration::from_millis(100);

    for attempt in 0..MAX_ATTEMPTS {
        // Send a simple initialize request to test if server is ready
        let test_request = json!({
            "jsonrpc": "2.0",
            "id": 0,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"prompts": {}},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        });

        let request_str = serde_json::to_string(&test_request)?;

        // Try to send the request
        match writeln!(stdin, "{request_str}") {
            Ok(_) => {
                if stdin.flush().is_ok() {
                    // Try to read a response with a short timeout
                    match timeout(Duration::from_millis(200), async {
                        let mut line = String::new();
                        reader.read_line(&mut line)
                    })
                    .await
                    {
                        Ok(Ok(_)) => {
                            // Server responded - it's ready!
                            return Ok(());
                        }
                        Ok(Err(_)) | Err(_) => {
                            // Server didn't respond or responded with error, try again
                        }
                    }
                }
            }
            Err(_) => {
                // Failed to write to stdin, server might not be ready
            }
        }

        // Wait before next attempt
        sleep(RETRY_DELAY).await;

        // Log progress every second
        if attempt % 10 == 0 && attempt > 0 {
            eprintln!("Waiting for server to be ready... (attempt {attempt})");
        }
    }

    Err("Server did not become ready within timeout".into())
}

/// End-to-end test for MCP server handling prompts with partials (issue #58)
#[tokio::test]
#[serial]
async fn test_mcp_server_partial_rendering() {
    // Start the MCP server process
    let child = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--bin",
            "swissarmyhammer",
            "--",
            "serve",
        ])
        .current_dir("..") // Run from project root
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");

    let mut child = ProcessGuard(child);

    let mut stdin = child.0.stdin.take().expect("Failed to get stdin");
    let stdout = child.0.stdout.take().expect("Failed to get stdout");
    let stderr = child.0.stderr.take().expect("Failed to get stderr");
    let mut reader = BufReader::new(stdout);

    // Wait for the server to be ready
    wait_for_server_ready(&mut stdin, &mut reader)
        .await
        .expect("Server failed to start properly");

    // Spawn stderr reader for debugging
    std::thread::spawn(move || {
        let stderr_reader = BufReader::new(stderr);
        for line in stderr_reader.lines().map_while(Result::ok) {
            eprintln!("SERVER: {line}");
        }
    });

    // Helper to send JSON-RPC request
    let send_request = |stdin: &mut std::process::ChildStdin, request: Value| {
        let request_str = serde_json::to_string(&request).unwrap();
        writeln!(stdin, "{request_str}").unwrap();
        stdin.flush().unwrap();
    };

    // Helper to read JSON-RPC response
    let read_response = |reader: &mut BufReader<std::process::ChildStdout>| -> Result<Value, Box<dyn std::error::Error>> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line.trim().is_empty() {
            return Err("Empty response".into());
        }
        Ok(serde_json::from_str(line.trim())?)
    };

    // Step 1: Send initialized notification (server is already initialized from readiness check)
    send_request(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    );

    // Step 3: Test getting the 'example' prompt which uses partials
    send_request(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "prompts/get",
            "params": {
                "name": "example",
                "arguments": {"topic": "testing partials"}
            }
        }),
    );

    let response = timeout(Duration::from_secs(5), async { read_response(&mut reader) })
        .await
        .expect("Timeout")
        .expect("Failed to read get prompt response");

    println!(
        "Get example prompt response: {}",
        serde_json::to_string_pretty(&response).unwrap()
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);

    // Check if we got an error
    if let Some(error) = response.get("error") {
        panic!(
            "MCP error: {}",
            serde_json::to_string_pretty(&error).unwrap()
        );
    }

    // Verify the result contains rendered content
    let result = &response["result"];
    assert!(result.is_object(), "Expected result to be an object");

    let messages = result["messages"]
        .as_array()
        .expect("Expected messages array");
    assert!(!messages.is_empty(), "Expected at least one message");

    let message = &messages[0];
    let content = message["content"]["text"]
        .as_str()
        .expect("Expected text content");

    println!("Rendered content:\n{content}");

    // Verify that the partial was successfully rendered
    assert!(
        content.contains("Example Prompt"),
        "Should contain main prompt content"
    );
    assert!(
        content.contains("testing partials"),
        "Should contain the argument value"
    );

    // Step 4: Test getting 'do_next_issue' prompt if available (uses principals partial)
    send_request(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "prompts/get",
            "params": {
                "name": "do_next_issue",
                "arguments": {}
            }
        }),
    );

    let response = timeout(Duration::from_secs(5), async { read_response(&mut reader) })
        .await
        .expect("Timeout");

    if let Ok(response) = response {
        println!(
            "Get do_next_issue prompt response: {}",
            serde_json::to_string_pretty(&response).unwrap()
        );

        // Check if this is a notification (which can happen during file changes)
        if response.get("method").is_some() {
            println!("Received notification instead of response - this is expected for non-existent prompts");
        } else if response.get("error").is_none() {
            let result = &response["result"];
            if let Some(messages) = result.get("messages").and_then(|m| m.as_array()) {
                if !messages.is_empty() {
                    let message = &messages[0];
                    let content = message["content"]["text"]
                        .as_str()
                        .expect("Expected text content");

                    println!(
                        "do_next_issue rendered content:\n{}",
                        &content[..content.len().min(500)]
                    );

                    // Verify principals partial was included
                    assert!(
                        content.contains("Principals") || content.contains("principals"),
                        "Should contain principals content"
                    );
                }
            }
        } else {
            println!("Received error response for non-existent prompt (expected behavior)");
        }
    }

    // Cleanup
    // Clean up (handled by ProcessGuard drop)
}
