//! Example demonstrating the microsandbox-portal RPC command execution.
//!
//! This example showcases how to use the JSON-RPC API to execute system commands
//! and retrieve their output via the microsandbox-portal service. It demonstrates:
//!
//! - Connecting to the portal server
//! - Sending command execution requests
//! - Processing command output and exit codes
//! - Handling different command types
//!
//! # API Methods Demonstrated
//!
//! - `sandbox.command.run`: Execute a command in a sandboxed environment
//!
//! # Running the Example
//!
//! First, start the portal server:
//!
//! ```bash
//! # From the monocore directory:
//! cargo run --bin portal
//! ```
//!
//! Then, in another terminal, run this example:
//!
//! ```bash
//! cargo run --example rpc_command
//! ```
//!
//! # Requirements
//!
//! - A running microsandbox-portal server on localhost:4444
//!
//! # Example Output
//!
//! The example will display the RPC results and the output from each command execution:
//!
//! ```text
//! 📁 Running 'ls' command:
//! Command: ls
//! Args: ["-la"]
//! Exit code: 0
//! Success: true
//!
//! Output:
//! [stdout] total 92
//! [stdout] drwxr-xr-x  13 user  staff   416 Jul 15 10:30 .
//! [stdout] drwxr-xr-x   6 user  staff   192 Jul 15 10:20 ..
//! [stdout] -rw-r--r--   1 user  staff  2476 Jul 15 10:30 Cargo.toml
//! [stdout] drwxr-xr-x   4 user  staff   128 Jul 15 10:20 examples
//! [stdout] drwxr-xr-x   6 user  staff   192 Jul 15 10:30 lib
//! [stdout] drwxr-xr-x   4 user  staff   128 Jul 15 10:20 src
//! [stdout] drwxr-xr-x  12 user  staff   384 Jul 15 10:20 target
//! ```
//!
//! # Note
//!
//! This example demonstrates how to interact with the microsandbox-portal via
//! JSON-RPC for command execution. In a real application, you might want to implement
//! additional error handling and more sophisticated request/response processing.

use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

// Import the parameter types from the microsandbox-portal crate
use microsandbox_portal::payload::{JsonRpcRequest, SandboxCommandRunParams, JSONRPC_VERSION};

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Send a JSON-RPC request to the portal server
async fn send_rpc_request<T: serde::Serialize>(
    client: &Client,
    method: &str,
    params: T,
) -> Result<Value> {
    // Create a properly structured JSON-RPC request
    let request = JsonRpcRequest {
        jsonrpc: JSONRPC_VERSION.to_string(),
        method: method.to_string(),
        params: serde_json::to_value(params)?,
        id: json!(1),
    };

    let response = client
        .post("http://127.0.0.1:4444/api/v1/rpc")
        .json(&request)
        .send()
        .await?
        .json::<Value>()
        .await?;

    // Check for errors in the JSON-RPC error field
    if response.get("error").is_some() {
        let error = &response["error"];
        eprintln!(
            "RPC Error {}: {}",
            error["code"].as_i64().unwrap_or(0),
            error["message"].as_str().unwrap_or("Unknown error")
        );
        anyhow::bail!(
            "RPC request failed: {}",
            error["message"].as_str().unwrap_or("Unknown error")
        );
    }

    // Extract the result or return empty object if it doesn't exist
    let result = response.get("result").cloned().unwrap_or(json!({}));
    Ok(result)
}

/// Print command output lines from JSON
fn print_output_lines(output: &Value) {
    if let Some(output_array) = output.as_array() {
        if output_array.is_empty() {
            println!("No output lines found.");
        } else {
            for line in output_array {
                let stream = line
                    .get("stream")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let text = line.get("text").and_then(|v| v.as_str()).unwrap_or("");
                println!("[{}] {}", stream, text);
            }
        }
    } else {
        println!("No output found in response.");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create HTTP client
    let client = Client::new();

    // Execute a simple 'ls' command using the typed params
    println!("\n📁 Running 'ls' command:");
    let ls_params = SandboxCommandRunParams {
        command: "ls".to_string(),
        args: vec!["-la".to_string()],
        timeout: Some(30), // Add a 30 second timeout
    };

    let result = send_rpc_request(&client, "sandbox.command.run", ls_params).await?;

    // Extract command execution details
    let command = result
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let args = result
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| format!("{:?}", arr))
        .unwrap_or_else(|| "[]".to_string());

    let exit_code = result
        .get("exit_code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    let success = result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    println!("Command: {}", command);
    println!("Args: {}", args);
    println!("Exit code: {}", exit_code);
    println!("Success: {}", success);

    // Print the output lines
    println!("\nOutput from execute response:");
    if let Some(output) = result.get("output") {
        print_output_lines(output);
    } else {
        println!("No output found in response.");
    }

    // Execute another command with environment variables using the typed params
    println!("\n🔄 Running 'echo' command:");
    let echo_params = SandboxCommandRunParams {
        command: "echo".to_string(),
        args: vec!["Hello from the sandbox!".to_string()],
        timeout: None, // No timeout needed for simple echo command
    };

    let result = send_rpc_request(&client, "sandbox.command.run", echo_params).await?;

    // Extract command execution details
    let command = result
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let args = result
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| format!("{:?}", arr))
        .unwrap_or_else(|| "[]".to_string());

    let exit_code = result
        .get("exit_code")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    let success = result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    println!("Command: {}", command);
    println!("Args: {}", args);
    println!("Exit code: {}", exit_code);
    println!("Success: {}", success);

    // Print the output lines
    println!("\nOutput from execute response:");
    if let Some(output) = result.get("output") {
        print_output_lines(output);
    } else {
        println!("No output found in response.");
    }

    // Execute a command that will fail to demonstrate error handling
    println!("\n❌ Running a command that will fail:");
    let fail_params = SandboxCommandRunParams {
        command: "nonexistent_command".to_string(),
        args: vec![],
        timeout: Some(5), // Short timeout
    };

    // This will likely fail, so handle the error case
    match send_rpc_request(&client, "sandbox.command.run", fail_params).await {
        Ok(result) => {
            // Still might get a result with error details
            let exit_code = result
                .get("exit_code")
                .and_then(|v| v.as_i64())
                .unwrap_or(-1);

            println!("Command executed but failed with exit code: {}", exit_code);

            // Print any output/error messages
            println!("\nError output:");
            if let Some(output) = result.get("output") {
                print_output_lines(output);
            } else {
                println!("No error output found in response.");
            }
        }
        Err(e) => {
            println!("Failed to execute nonexistent command: {}", e);
        }
    }

    println!("\nCommand execution examples completed!");
    Ok(())
}
