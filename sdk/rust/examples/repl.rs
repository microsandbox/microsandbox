//! Advanced example demonstrating the Python sandbox features.
//!
//! This example shows:
//! 1. Different ways to create and manage sandboxes
//! 2. Resource configuration (memory, CPU)
//! 3. Error handling
//! 4. Multiple code execution patterns
//! 5. Output handling
//! 6. Timeouts and handling long-running starts
//!
//! Before running this example:
//!     1. Install the package as a dependency
//!     2. Start the Microsandbox server (microsandbox-server)
//!     3. Run this script: cargo run --example repl
//!
//! Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.

use microsandbox::{BaseSandbox, PythonSandbox, SandboxOptions, StartOptions};
use std::error::Error;

/// Example demonstrating basic sandbox creation and cleanup.
async fn example_simple_sandbox() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Simple Sandbox Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("sandbox-simple").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Run some computation
    let code = r#"
print("Hello, world!")
"#;
    let execution = sandbox.run(code).await?;
    let output = execution.output().await?;
    println!("Output: {}", output);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example using custom options and resource constraints.
async fn example_explicit_lifecycle() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Explicit Lifecycle Example ===");

    // Create sandbox with custom configuration
    let options = SandboxOptions::builder()
        .server_url("http://127.0.0.1:5555")
        .name("sandbox-explicit")
        .build();

    let mut sandbox = PythonSandbox::create_with_options(options).await?;

    // Start with resource constraints
    let start_options = StartOptions {
        image: None,
        memory: 1024, // 1GB RAM
        cpus: 2.0,    // 2 CPU cores
        timeout: 180.0,
    };

    // Start the sandbox
    sandbox.start(Some(start_options)).await?;

    // Run multiple code blocks with variable assignments
    sandbox.run("x = 42").await?;
    sandbox.run("y = [i**2 for i in range(10)]").await?;
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

    // Cleanup
    sandbox.stop().await?;

    Ok(())
}

/// Example demonstrating state persistence between executions.
async fn example_execution_chaining() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Execution Chaining Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("sandbox-chain").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Execute a sequence of related code blocks
    sandbox.run("name = 'Python'").await?;
    sandbox.run("import sys").await?;
    sandbox.run("version = sys.version").await?;
    let exec = sandbox
        .run("print(f'Hello from {name} {version}!')")
        .await?;

    // Only get output from the final execution
    println!("Output: {}", exec.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Run all examples
    example_simple_sandbox().await?;
    example_explicit_lifecycle().await?;
    example_execution_chaining().await?;

    Ok(())
}
