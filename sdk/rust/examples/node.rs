//! Example demonstrating how to use NodeSandbox to execute JavaScript code.
//!
//! This example shows:
//! 1. Basic Node.js code execution
//! 2. Different sandbox management patterns
//! 3. JavaScript module usage
//! 4. Error output handling
//!
//! Before running this example:
//!     1. Install the package as a dependency
//!     2. Start the Microsandbox server (microsandbox-server)
//!     3. Run this script: cargo run --example node

use microsandbox::{BaseSandbox, NodeSandbox};
use std::error::Error;

/// Example showing basic JavaScript code execution.
async fn basic_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Basic Node.js Example ===");

    // Create a sandbox
    let mut sandbox = NodeSandbox::create("node-basic").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Run a simple JavaScript code snippet
    let execution = sandbox.run("console.log('Hello from Node.js!');").await?;
    let output = execution.output().await?;
    println!("Output: {}", output);

    // Run JavaScript code that uses Node.js functionality
    let version_code = r#"
const version = process.version;
const platform = process.platform;
console.log(`Node.js ${version} running on ${platform}`);
"#;
    let version_execution = sandbox.run(version_code).await?;
    println!("Node.js info: {}", version_execution.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing how to handle JavaScript errors.
async fn error_handling_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Error Handling Example ===");

    // Create a sandbox
    let mut sandbox = NodeSandbox::create("node-error").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Run code with a caught error
    let caught_error_code = r#"
try {
    // This will cause a ReferenceError
    console.log(undefinedVariable);
} catch (error) {
    console.error('Caught error:', error.message);
}
"#;
    let caught_execution = sandbox.run(caught_error_code).await?;
    println!("Standard output: {}", caught_execution.output().await?);
    println!("Error output: {}", caught_execution.error().await?);
    println!("Has error: {}", caught_execution.has_error());

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing Node.js module usage.
async fn module_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Module Usage Example ===");

    // Create a sandbox
    let mut sandbox = NodeSandbox::create("node-module").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Using built-in Node.js modules
    let fs_code = r#"
const fs = require('fs');
const os = require('os');

// Write a file
fs.writeFileSync('/tmp/hello.txt', 'Hello from Node.js!');
console.log('File written successfully');

// Read the file back
const content = fs.readFileSync('/tmp/hello.txt', 'utf8');
console.log('File content:', content);

// Get system info
console.log('Hostname:', os.hostname());
console.log('Platform:', os.platform());
console.log('Architecture:', os.arch());
"#;
    let fs_execution = sandbox.run(fs_code).await?;
    println!("{}", fs_execution.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example demonstrating execution chaining with variables.
async fn execution_chaining_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Execution Chaining Example ===");

    // Create a sandbox
    let mut sandbox = NodeSandbox::create("node-chain").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Execute a sequence of related code blocks that maintain state
    sandbox.run("const name = 'Node.js';").await?;
    sandbox.run("const version = process.version;").await?;
    sandbox.run("const numbers = [1, 2, 3, 4, 5];").await?;

    // Use variables from previous executions
    let final_execution = sandbox
        .run(
            r#"
console.log(`Hello from ${name} ${version}!`);
const sum = numbers.reduce((a, b) => a + b, 0);
console.log(`Sum of numbers: ${sum}`);
"#,
        )
        .await?;

    println!("{}", final_execution.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Node.js Sandbox Examples");
    println!("=======================");

    // Run all examples
    basic_example().await?;
    error_handling_example().await?;
    module_example().await?;
    execution_chaining_example().await?;

    println!("\nAll Node.js examples completed!");

    Ok(())
}
