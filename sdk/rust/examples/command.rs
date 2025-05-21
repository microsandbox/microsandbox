//! Example demonstrating how to use sandbox commands to execute shell commands.
//!
//! This example shows:
//! 1. Basic command execution
//! 2. Error handling
//! 3. Working with command arguments
//! 4. Handling command timeouts
//! 5. Complex command execution with pipelines
//!
//! Before running this example:
//!     1. Install the package as a dependency
//!     2. Start the Microsandbox server (microsandbox-server)
//!     3. Run this script: cargo run --example command
//!
//! Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.

use microsandbox::{PythonSandbox, SandboxOptions};
use std::{error::Error, time::Duration};

/// Example showing basic command execution.
async fn basic_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Basic Command Example ===");

    // Create a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("command-example").build()).await?;

    // Run a simple command
    let ls_execution = sandbox
        .command()
        .run("ls", Some(vec!["-la", "/"]), None)
        .await?;
    println!("$ ls -la /");
    println!("Exit code: {}", ls_execution.exit_code());
    println!("Output:");
    println!("{}", ls_execution.output().await?);

    // Execute a command with string arguments
    let echo_execution = sandbox
        .command()
        .run("echo", Some(vec!["Hello from", "sandbox command!"]), None)
        .await?;
    println!("\n$ echo Hello from sandbox command!");
    println!("Output: {}", echo_execution.output().await?);

    // Get system information
    let uname_execution = sandbox
        .command()
        .run("uname", Some(vec!["-a"]), None)
        .await?;
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
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("error-example").build()).await?;

    // Run a command that generates an error
    let error_execution = sandbox
        .command()
        .run("ls", Some(vec!["/nonexistent"]), None)
        .await?;

    println!("$ ls /nonexistent");
    println!("Exit code: {}", error_execution.exit_code());
    println!("Success: {}", error_execution.is_success());
    println!("Error output:");
    println!("{}", error_execution.error().await?);

    // Deliberately cause a command not found error
    println!("\nTrying a nonexistent command...");
    match sandbox
        .command()
        .run("nonexistentcommand", None, None)
        .await
    {
        Ok(_) => println!("Command succeeded unexpectedly"),
        Err(e) => println!("Caught exception for nonexistent command: {}", e),
    }

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing how to use command timeouts.
async fn timeout_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Timeout Example ===");

    // Create a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("timeout-example").build()).await?;

    // Run a command that takes longer than the specified timeout
    println!("Running command with timeout...");
    match sandbox
        .command()
        .run("sleep", Some(vec!["10"]), Some(2))
        .await
    {
        Ok(_) => println!("Command completed (unexpected!)"),
        Err(e) => println!("Command timed out as expected: {}", e),
    }

    // Show that the sandbox is still usable after a timeout
    let echo_execution = sandbox
        .command()
        .run("echo", Some(vec!["Still working!"]), None)
        .await?;
    println!("\nSandbox still works: {}", echo_execution.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing more advanced command usage.
async fn advanced_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Advanced Example ===");

    // Create a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("advanced-example").build()).await?;

    // Write a file
    let write_cmd = sandbox
        .command()
        .run(
            "bash",
            Some(vec!["-c", "echo 'Hello, file content!' > /tmp/test.txt"]),
            None,
        )
        .await?;
    println!("Created file, exit code: {}", write_cmd.exit_code());

    // Read the file back
    let read_cmd = sandbox
        .command()
        .run("cat", Some(vec!["/tmp/test.txt"]), None)
        .await?;
    println!("File content: {}", read_cmd.output().await?);

    // Run a more complex pipeline
    let pipeline_cmd = sandbox
        .command()
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
    let script_content = r#"cat > /tmp/test.py << 'EOF'
import sys
print("Python script executed!")
print(f"Arguments: {sys.argv[1:]}")
EOF"#;

    let create_script = sandbox
        .command()
        .run("bash", Some(vec!["-c", script_content]), None)
        .await?;

    if create_script.is_success() {
        // Run the script with arguments
        let script_cmd = sandbox
            .command()
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

/// Example demonstrating running multiple commands in sequence.
async fn sequential_commands_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Sequential Commands Example ===");

    // Create a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("sequential-example").build()).await?;

    // Execute several commands in sequence
    println!("Running multiple commands in sequence...");

    // Get hostname
    let hostname_cmd = sandbox.command().run("hostname", None, None).await?;
    println!("Hostname: {}", hostname_cmd.output().await?);

    // Get date
    let date_cmd = sandbox.command().run("date", None, None).await?;
    println!("Date: {}", date_cmd.output().await?);

    // Check kernel version
    let kernel_cmd = sandbox
        .command()
        .run("uname", Some(vec!["-r"]), None)
        .await?;
    println!("Kernel version: {}", kernel_cmd.output().await?);

    // Run a command that creates some files
    let _ = sandbox
        .command()
        .run(
            "bash",
            Some(vec![
                "-c",
                "for i in {1..3}; do echo \"File $i\" > \"/tmp/file_$i.txt\"; done",
            ]),
            None,
        )
        .await?;

    // List the files we created
    let ls_cmd = sandbox
        .command()
        .run("ls", Some(vec!["-l", "/tmp/file_*.txt"]), None)
        .await?;
    println!("\nCreated files:");
    println!("{}", ls_cmd.output().await?);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Command Execution Examples");
    println!("=========================");

    // Run all examples, capturing errors but continuing
    let examples = vec![
        tokio::spawn(basic_example()),
        tokio::spawn(error_handling_example()),
        tokio::spawn(timeout_example()),
        tokio::spawn(advanced_example()),
        tokio::spawn(sequential_commands_example()),
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

    println!("\nAll examples completed!");
    Ok(())
}
