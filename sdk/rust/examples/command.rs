//! Example demonstrating how to use sandbox.command.run to execute shell commands.
//!
//! This example shows:
//! 1. Basic command execution
//! 2. Error handling for commands
//! 3. Using command timeouts
//! 4. Advanced command usage
//! 5. Explicit lifecycle management
//!
//! Before running this example:
//!     1. Install the package as a dependency
//!     2. Start the Microsandbox server (microsandbox-server)
//!     3. Run this script: cargo run --example command

use microsandbox::{BaseSandbox, PythonSandbox};
use std::error::Error;

/// Example showing basic command execution.
async fn basic_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Basic Command Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("command-example").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    // Run a simple command
    let ls_execution = cmd.run("ls", Some(vec!["-la", "/"]), None).await?;
    println!("$ ls -la /");
    println!("Exit code: {}", ls_execution.exit_code());
    println!("Output:");
    println!("{}", ls_execution.output().await?);

    // Execute a command with string arguments
    let echo_execution = cmd
        .run("echo", Some(vec!["Hello from", "sandbox command!"]), None)
        .await?;
    println!("\n$ echo Hello from sandbox command!");
    println!("Output: {}", echo_execution.output().await?);

    // Get system information
    let uname_execution = cmd.run("uname", Some(vec!["-a"]), None).await?;
    println!("\n$ uname -a");
    println!("Output: {}", uname_execution.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing how to handle command errors.
async fn error_handling_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Error Handling Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("error-example").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    // Run a command that generates an error
    let error_execution = cmd.run("ls", Some(vec!["/nonexistent"]), None).await?;

    println!("$ ls /nonexistent");
    println!("Exit code: {}", error_execution.exit_code());
    println!("Success: {}", error_execution.is_success());
    println!("Error output:");
    println!("{}", error_execution.error().await?);

    // Deliberately cause a command not found error
    match cmd.run("nonexistentcommand", None, None).await {
        Ok(_) => println!("Command succeeded unexpectedly"),
        Err(e) => println!("\nCaught exception for nonexistent command: {}", e),
    }

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing how to use command timeouts.
async fn timeout_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Timeout Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("timeout-example").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    println!("Running command with timeout...");
    // Run a command that takes longer than the specified timeout
    match cmd.run("sleep", Some(vec!["10"]), Some(2)).await {
        Ok(_) => println!("Command completed (unexpected!)"),
        Err(e) => println!("Command timed out as expected: {}", e),
    }

    // Show that the sandbox is still usable after a timeout
    let echo_execution = cmd.run("echo", Some(vec!["Still working!"]), None).await?;
    println!("\nSandbox still works: {}", echo_execution.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing more advanced command usage.
async fn advanced_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Advanced Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("advanced-example").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    // Write a file
    let write_cmd = cmd
        .run(
            "bash",
            Some(vec!["-c", "echo 'Hello, file content!' > /tmp/test.txt"]),
            None,
        )
        .await?;
    println!("Created file, exit code: {}", write_cmd.exit_code());

    // Read the file back
    let read_cmd = cmd.run("cat", Some(vec!["/tmp/test.txt"]), None).await?;
    println!("File content: {}", read_cmd.output().await?);

    // Run a more complex pipeline
    let pipeline_cmd = cmd
        .run(
            "bash",
            Some(vec![
                "-c",
                "mkdir -p /tmp/test_dir && \
                 echo 'Line 1' > /tmp/test_dir/data.txt && \
                 echo 'Line 2' >> /tmp/test_dir/data.txt && \
                 cat /tmp/test_dir/data.txt | grep 'Line' | wc -l",
            ]),
            None,
        )
        .await?;
    println!(
        "\nPipeline output (should be 2): {}",
        pipeline_cmd.output().await?
    );

    // Create and run a Python script
    let create_script = cmd
        .run(
            "bash",
            Some(vec![
                "-c",
                r#"cat > /tmp/test.py << 'EOF'
import sys
print("Python script executed!")
print(f"Arguments: {sys.argv[1:]}")
EOF"#,
            ]),
            None,
        )
        .await?;

    if create_script.is_success() {
        // Run the script with arguments
        let script_cmd = cmd
            .run(
                "python",
                Some(vec!["/tmp/test.py", "arg1", "arg2", "arg3"]),
                None,
            )
            .await?;
        println!("\nPython script output:");
        println!("{}", script_cmd.output().await?);
    }

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing explicit lifecycle management.
async fn explicit_lifecycle_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Explicit Lifecycle Example ===");

    // Create sandbox without automatically starting
    let mut sandbox = PythonSandbox::create("explicit-lifecycle").await?;

    // Manually start the sandbox
    println!("Starting sandbox...");
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    // Execute commands
    let hostname_cmd = cmd.run("hostname", None, None).await?;
    println!("Hostname: {}", hostname_cmd.output().await?);

    let date_cmd = cmd.run("date", None, None).await?;
    println!("Date: {}", date_cmd.output().await?);

    // Manually stop the sandbox
    println!("Stopping sandbox...");
    sandbox.stop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Command Execution Examples");
    println!("=========================");

    // Run all examples
    basic_example().await?;
    error_handling_example().await?;
    timeout_example().await?;
    advanced_example().await?;
    explicit_lifecycle_example().await?;

    println!("\nAll examples completed!");

    Ok(())
}
