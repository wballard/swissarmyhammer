use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::timeout;

mod test_utils;
use test_utils::ProcessGuard;

/// End-to-end test for MCP server handling prompts with partials (issue #58)
#[tokio::test]
async fn test_mcp_server_partial_rendering() {
    // Start the MCP server process
    let child = Command::new("cargo")
        .args(["run", "--", "serve"])
        .current_dir("..") // Run from project root
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start MCP server");

    let mut child = ProcessGuard(child);

    // Give the server time to start
    std::thread::sleep(Duration::from_millis(1000));

    let mut stdin = child.0.stdin.take().expect("Failed to get stdin");
    let stdout = child.0.stdout.take().expect("Failed to get stdout");
    let stderr = child.0.stderr.take().expect("Failed to get stderr");
    let mut reader = BufReader::new(stdout);

    // Spawn stderr reader for debugging
    std::thread::spawn(move || {
        let stderr_reader = BufReader::new(stderr);
        for line in stderr_reader.lines().map_while(Result::ok) {
            eprintln!("SERVER: {}", line);
        }
    });

    // Helper to send JSON-RPC request
    let send_request = |stdin: &mut std::process::ChildStdin, request: Value| {
        let request_str = serde_json::to_string(&request).unwrap();
        writeln!(stdin, "{}", request_str).unwrap();
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

    // Step 1: Initialize
    send_request(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"prompts": {}},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }),
    );

    let response = timeout(Duration::from_secs(5), async { read_response(&mut reader) })
        .await
        .expect("Timeout")
        .expect("Failed to read initialize response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());

    // Step 2: Send initialized notification
    send_request(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    );

    // Give server time to process
    std::thread::sleep(Duration::from_millis(100));

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

    println!("Rendered content:\n{}", content);

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

        if response.get("error").is_none() {
            let result = &response["result"];
            let messages = result["messages"]
                .as_array()
                .expect("Expected messages array");
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

    // Cleanup
    // Clean up (handled by ProcessGuard drop)
}
