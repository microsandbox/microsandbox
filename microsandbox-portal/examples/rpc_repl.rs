//! Example demonstrating the microsandbox-portal RPC code execution in REPL environment.
//!
//! This example showcases how to use the JSON-RPC API to execute code in a REPL environment
//! and retrieve output results via the microsandbox-portal service. It demonstrates:
//!
//! - Connecting to the portal server
//! - Sending code execution requests to REPL
//! - Processing execution output
//! - Error handling with JSON-RPC responses
//!
//! # API Methods Demonstrated
//!
//! - `sandbox.repl.run`: Execute code in a specific language's REPL
//!
//! # Running the Example
//!
//! First, start the portal server with the appropriate language features enabled:
//!
//! ```bash
//! # From the monocore directory:
//! cargo run --bin portal --features "python nodejs"
//! ```
//!
//! Then, in another terminal, run this example:
//!
//! ```bash
//! cargo run --example rpc_repl
//! ```
//!
//! # Requirements
//!
//! - A running microsandbox-portal server on localhost:4444
//! - Language features enabled on the server:
//!   - Python: Python interpreter installed and available in PATH
//!   - Node.js: Node.js installed and available in PATH
//!
//! # Example Output
//!
//! The example will display the RPC results and the output from each code execution:
//!
//! ```text
//! 🐍 Running Python example in REPL:
//! Status: success
//!
//! Output:
//! [stdout] Factorial examples:
//! [stdout] factorial(1) = 1
//! [stdout] factorial(2) = 2
//! [stdout] factorial(3) = 6
//! [stdout] factorial(4) = 24
//! [stdout] factorial(5) = 120
//! ```
//!
//! # Note
//!
//! This example demonstrates how to interact with the microsandbox-portal via
//! JSON-RPC. In a real application, you might want to implement additional
//! error handling and more sophisticated request/response processing.

use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

// Import the parameter types from the microsandbox-portal crate
use microsandbox_portal::payload::{JsonRpcRequest, SandboxReplRunParams, JSONRPC_VERSION};

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
        id: Some(Value::from(1)),
    };

    let response = client
        .post("http://127.0.0.1:4444/api/v1/rpc")
        .json(&request)
        .send()
        .await?
        .json::<Value>()
        .await?;

    // Print the full response for debugging
    println!("RPC Response: {}", response);

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

    // Also check for direct error codes (might be incorrectly formatted responses)
    if let Some(code) = response.get("code") {
        if let Some(message) = response.get("message") {
            eprintln!(
                "Direct Error {}: {}",
                code.as_i64().unwrap_or(0),
                message.as_str().unwrap_or("Unknown error")
            );
            anyhow::bail!(
                "RPC request failed: {}",
                message.as_str().unwrap_or("Unknown error")
            );
        }
    }

    // Extract the result or return empty object if it doesn't exist
    let result = response.get("result").cloned().unwrap_or(json!({}));
    Ok(result)
}

/// Print output lines from JSON
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

    // Execute Python code in REPL
    println!("\n🐍 Running Python example in REPL:");
    let python_code = r#"
# Define a function
def factorial(n):
    if n == 0 or n == 1:
        return 1
    else:
        return n * factorial(n-1)

# Use the function
print("Factorial examples:")
for i in range(1, 6):
    print(f"factorial({i}) = {factorial(i)}")
    "#;

    // Create typed parameters for Python code execution
    let python_params = SandboxReplRunParams {
        code: python_code.to_string(),
        language: "python".to_string(),
        timeout: Some(30), // Add a 30 second timeout
    };

    // Send sandbox.repl.run request with the typed parameters
    let run_result = match send_rpc_request(&client, "sandbox.repl.run", python_params).await {
        Ok(result) => result,
        Err(e) => {
            println!("Error running Python code in REPL: {}", e);
            println!("Execution will continue but might fail...");
            json!({})
        }
    };

    // Extract execution details
    let status = run_result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("Status: {}", status);

    // Print the output lines directly from the run response
    println!("\nOutput:");
    if let Some(output) = run_result.get("output") {
        print_output_lines(output);
    } else {
        println!("No output found in the response.");
    }

    // Execute JavaScript code in REPL
    println!("\n🟨 Running JavaScript example in REPL:");
    let js_code = r#"
// Define a class
class Counter {
    constructor(initial = 0) {
        this.count = initial;
    }

    increment() {
        this.count++;
        return this.count;
    }
}

// Use the class
const counter = new Counter(10);
console.log(`Initial count: ${counter.count}`);

for (let i = 0; i < 5; i++) {
    console.log(`After increment: ${counter.increment()}`);
}
    "#;

    // Create typed parameters for JavaScript code execution
    let js_params = SandboxReplRunParams {
        code: js_code.to_string(),
        language: "nodejs".to_string(),
        timeout: Some(30), // Add a 30 second timeout
    };

    // Send sandbox.repl.run request
    let js_result = match send_rpc_request(&client, "sandbox.repl.run", js_params).await {
        Ok(result) => result,
        Err(e) => {
            println!("Error running JavaScript code in REPL: {}", e);
            json!({})
        }
    };

    // Print status
    let status = js_result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("Status: {}", status);

    // Print the output lines directly from the run response
    println!("\nOutput:");
    if let Some(output) = js_result.get("output") {
        print_output_lines(output);
    } else {
        println!("No output found in the response.");
    }

    Ok(())
}
