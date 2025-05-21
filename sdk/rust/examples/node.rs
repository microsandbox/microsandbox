//! Example demonstrating how to use NodeSandbox to execute JavaScript code.
//!
//! This example shows:
//! 1. Basic Node.js code execution
//! 2. JavaScript module usage
//! 3. Error output handling
//! 4. Execution chaining with variables
//!
//! Before running this example:
//!     1. Install the package as a dependency
//!     2. Start the Microsandbox server (microsandbox-server)
//!     3. Run this script: cargo run --example node
//!
//! Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.

use microsandbox::{NodeSandbox, SandboxOptions};
use std::error::Error;

/// Example showing basic JavaScript code execution.
async fn basic_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Basic Node.js Example ===");

    // Create a sandbox
    let mut sandbox =
        NodeSandbox::create(SandboxOptions::builder().name("node-basic").build()).await?;

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
    let mut sandbox =
        NodeSandbox::create(SandboxOptions::builder().name("node-error").build()).await?;

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

    // Run code with an uncaught error
    println!("\nTrying code with an uncaught error...");
    let uncaught_error_code = "console.log(undefinedVariable);";
    match sandbox.run(uncaught_error_code).await {
        Ok(execution) => {
            println!("Has error: {}", execution.has_error());
            println!("Error output: {}", execution.error().await?);
        }
        Err(e) => {
            println!("Execution failed: {}", e);
        }
    }

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing Node.js module usage.
async fn module_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Module Usage Example ===");

    // Create a sandbox
    let mut sandbox =
        NodeSandbox::create(SandboxOptions::builder().name("node-module").build()).await?;

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
    let mut sandbox =
        NodeSandbox::create(SandboxOptions::builder().name("node-chain").build()).await?;

    // Execute a sequence of related code blocks that maintain state
    let _ = sandbox.run("const name = 'Node.js';").await?;
    let _ = sandbox.run("const version = process.version;").await?;
    let _ = sandbox.run("const numbers = [1, 2, 3, 4, 5];").await?;

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

/// Example showing JSON and complex object handling in JavaScript.
async fn json_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== JSON Handling Example ===");

    // Create a sandbox
    let mut sandbox =
        NodeSandbox::create(SandboxOptions::builder().name("node-json").build()).await?;

    // Create a complex object with JSON
    let json_code = r#"
// Create a complex object
const data = {
    name: 'JSON Example',
    created: new Date().toISOString(),
    items: [
        { id: 1, value: 'One' },
        { id: 2, value: 'Two' },
        { id: 3, value: 'Three' }
    ],
    settings: {
        active: true,
        timeout: 5000,
        debug: false
    }
};

// Manipulate the data
data.items.push({ id: 4, value: 'Four' });
data.count = data.items.length;

// Print with formatting
console.log('JSON Data:');
console.log(JSON.stringify(data, null, 2));

// Access nested properties
console.log(`Total items: ${data.count}`);
console.log('Item values:');
data.items.forEach(item => console.log(`- ${item.value}`));
"#;
    let json_execution = sandbox.run(json_code).await?;
    println!("{}", json_execution.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Node.js Sandbox Examples");
    println!("=======================");

    // Run all examples
    let examples = vec![
        tokio::spawn(basic_example()),
        tokio::spawn(error_handling_example()),
        tokio::spawn(module_example()),
        tokio::spawn(execution_chaining_example()),
        tokio::spawn(json_example()),
    ];

    for (i, example) in futures::future::join_all(examples)
        .await
        .into_iter()
        .enumerate()
    {
        match example {
            Ok(result) => {
                if let Err(e) = result {
                    eprintln!("Error in example {}: {}", i + 1, e);
                }
            }
            Err(e) => eprintln!("Failed to run example {}: {}", i + 1, e),
        }
    }

    println!("\nAll Node.js examples completed!");
    Ok(())
}
