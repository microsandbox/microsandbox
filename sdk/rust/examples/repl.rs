//! Advanced example demonstrating the Python sandbox features.
//!
//! This example shows:
//! 1. Different ways to create and manage sandboxes
//! 2. Resource configuration (memory, CPU)
//! 3. Error handling
//! 4. Multiple code execution patterns
//! 5. Output handling
//!
//! Before running this example:
//!     1. Install the package as a dependency
//!     2. Start the Microsandbox server (microsandbox-server)
//!     3. Run this script: cargo run --example repl
//!
//! Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.

use microsandbox::{PythonSandbox, SandboxOptions, StartOptions};
use std::error::Error;

/// Example using manual management of the sandbox lifecycle.
async fn example_manual_lifecycle() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Manual Lifecycle Example ===");

    // Create sandbox with custom configuration
    let mut sandbox = PythonSandbox::create(
        SandboxOptions::builder()
            .server_url("http://127.0.0.1:5555")
            .name("sandbox-explicit")
            .build(),
    )
    .await?;

    // Run multiple code blocks with variable assignments
    let _ = sandbox.run("x = 42").await?;
    let _ = sandbox.run("y = [i**2 for i in range(10)]").await?;
    let execution3 = sandbox.run("print(f'x = {x}')\nprint(f'y = {y}')").await?;

    println!("Output: {}", execution3.output().await?);

    // Demonstrate error handling
    match sandbox.run("1/0").await {
        // This will raise a ZeroDivisionError
        Ok(error_execution) => {
            println!("Error: {}", error_execution.error().await?);
        }
        Err(e) => {
            println!("Caught error: {}", e);
        }
    }

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example demonstrating resource configuration.
async fn example_resource_config() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Resource Configuration Example ===");

    // Create the sandbox without starting it
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("sandbox-resource").build()).await?;

    // Configure and start with resource constraints
    let options = StartOptions {
        memory: 1024, // 1GB RAM
        cpus: 2.0,    // 2 CPU cores
        timeout: 180.0,
        image: None,
    };

    // Start the sandbox with the configuration
    sandbox.start(Some(options)).await?;

    // Run some code to show it's working
    let exec = sandbox
        .run(
            r#"
import os
import psutil
process = psutil.Process(os.getpid())
print(f"Memory info: {process.memory_info()}")
print(f"CPU count: {os.cpu_count()}")
"#,
        )
        .await?;

    println!("Sandbox resource info:\n{}", exec.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example demonstrating execution chaining with variables.
async fn example_execution_chaining() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Execution Chaining Example ===");

    // Create and start a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("sandbox-chain").build()).await?;

    // Execute a sequence of related code blocks
    let _ = sandbox.run("name = 'Python'").await?;
    let _ = sandbox.run("import sys").await?;
    let _ = sandbox.run("version = sys.version").await?;
    let exec = sandbox
        .run("print(f'Hello from {name} {version}!')")
        .await?;

    // Only get output from the final execution
    println!("Output: {}", exec.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example demonstrating complex code execution and outputs.
async fn example_complex_code() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Complex Code Example ===");

    // Create and start a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("sandbox-complex").build()).await?;

    // Run some complex code
    let complex_code = r#"
import math
import json
from datetime import datetime

# Create a sample data structure
data = {
    'name': 'Example',
    'created_at': str(datetime.now()),
    'values': [math.sin(i/10) for i in range(5)],
    'metadata': {
        'version': '1.0',
        'type': 'test'
    }
}

# Process and display the data
print('Current time:', data['created_at'])
print('Values:', data['values'])
print('JSON representation:')
print(json.dumps(data, indent=2))
"#;

    let exec = sandbox.run(complex_code).await?;
    println!("Complex code output:\n{}", exec.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("REPL Examples");
    println!("=============");

    // Run all examples, continuing even if one fails
    let examples = vec![
        example_manual_lifecycle(),
        example_resource_config(),
        example_execution_chaining(),
        example_complex_code(),
    ];

    for (i, example) in futures::future::join_all(examples)
        .await
        .into_iter()
        .enumerate()
    {
        if let Err(e) = example {
            eprintln!("Error in example {}: {}", i + 1, e);
        }
    }

    println!("\nAll examples completed!");
    Ok(())
}
