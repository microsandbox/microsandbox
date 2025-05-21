//! A simple example of using the Python sandbox
//!
//! This example demonstrates how to create a Python sandbox,
//! run some code, and print the output.

use microsandbox::{PythonSandbox, SandboxOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Python sandbox
    let mut sb = PythonSandbox::create(SandboxOptions::builder().name("test").build()).await?;
    println!("Sandbox created successfully");

    // Run a simple computation in Python
    let exec = sb.run(r#"result = 2 + 2"#).await?;
    println!("First execution completed");

    // Print the result of the computation
    let exec = sb.run(r#"print(f"2 + 2 = {result}")"#).await?;
    println!("Output: {}", exec.output().await?);

    // Run multiple commands and see the state is maintained
    let exec = sb
        .run(
            r#"
import random
random.seed(42)  # Set seed for reproducibility
nums = [random.randint(1, 100) for _ in range(5)]
    "#,
        )
        .await?;

    let exec = sb.run(r#"print(f"Random numbers: {nums}")"#).await?;
    println!("Random numbers: {}", exec.output().await?);

    // Demonstrate error handling
    let exec = sb.run(r#"print(undefined_variable)"#).await?;
    if exec.has_error() {
        println!("Error detected: {}", exec.error().await?);
    }

    // Execute a shell command
    let cmd_exec = sb.command().run("ls", Some(vec!["-la"]), None).await?;
    println!("\nDirectory listing:\n{}", cmd_exec.output().await?);

    // Stop the sandbox
    sb.stop().await?;
    println!("Sandbox stopped successfully");

    Ok(())
}
