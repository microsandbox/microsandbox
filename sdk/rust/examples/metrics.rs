//! Example demonstrating how to retrieve sandbox metrics.
//!
//! This example shows:
//! 1. Basic metrics retrieval for individual metrics
//! 2. Getting all metrics at once
//! 3. Continuous monitoring of metrics
//! 4. Generating CPU load to test metrics
//! 5. Error handling with metrics
//!
//! Before running this example:
//!     1. Install the package as a dependency
//!     2. Start the Microsandbox server (microsandbox-server)
//!     3. Run this script: cargo run --example metrics

use microsandbox::{BaseSandbox, PythonSandbox};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use tokio::time::sleep;

/// Example showing how to get individual metrics for a sandbox.
async fn basic_metrics_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Basic Metrics Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("metrics-example").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    // Run commands to generate some load
    println!("Running commands to generate some sandbox activity...");
    cmd.run("ls", Some(vec!["-la", "/"]), None).await?;
    cmd.run(
        "dd",
        Some(vec![
            "if=/dev/zero",
            "of=/tmp/testfile",
            "bs=1M",
            "count=10",
        ]),
        None,
    )
    .await?;

    // Sleep a moment to allow metrics to update
    sleep(Duration::from_secs(1)).await;

    // Get the metrics interface
    let metrics = sandbox.metrics().await?;

    // Get individual metrics
    println!("\nGetting individual metrics for this sandbox:");

    // Get CPU usage
    let cpu = metrics.cpu().await?;
    // CPU metrics may be 0.0 when idle or None if unavailable
    match cpu {
        Some(value) => println!("CPU Usage: {}%", value),
        None => println!("CPU Usage: Not available"),
    }

    // Get memory usage
    let memory = metrics.memory().await?;
    match memory {
        Some(value) => println!("Memory Usage: {} MiB", value),
        None => println!("Memory Usage: Not available"),
    }

    // Get disk usage
    let disk = metrics.disk().await?;
    match disk {
        Some(value) => println!("Disk Usage: {} bytes", value),
        None => println!("Disk Usage: Not available"),
    }

    // Check if running
    let running = metrics.is_running().await?;
    println!("Is Running: {}", running);

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing how to get all metrics at once.
async fn all_metrics_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== All Metrics Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("all-metrics-example").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    // Run some commands to generate activity
    println!("Running commands to generate some sandbox activity...");
    cmd.run("cat", Some(vec!["/etc/os-release"]), None).await?;
    cmd.run("ls", Some(vec!["-la", "/usr"]), None).await?;

    // Sleep a moment to allow metrics to update
    sleep(Duration::from_secs(1)).await;

    // Get the metrics interface
    let metrics = sandbox.metrics().await?;

    // Get all metrics at once
    println!("\nGetting all metrics as a JSON object:");
    let all_metrics = metrics.all().await?;

    // Print formatted metrics
    println!(
        "Sandbox: {} (namespace: {})",
        all_metrics
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown"),
        all_metrics
            .get("namespace")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
    );
    println!(
        "  Running: {}",
        all_metrics
            .get("running")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    );

    // Handle CPU metrics which may be 0.0 or None
    match all_metrics.get("cpu_usage").and_then(|v| v.as_f64()) {
        Some(cpu) => println!("  CPU Usage: {}%", cpu),
        None => println!("  CPU Usage: Not available"),
    }

    match all_metrics.get("memory_usage").and_then(|v| v.as_u64()) {
        Some(mem) => println!("  Memory Usage: {} MiB", mem),
        None => println!("  Memory Usage: Not available"),
    }

    match all_metrics.get("disk_usage").and_then(|v| v.as_u64()) {
        Some(disk) => println!("  Disk Usage: {} bytes", disk),
        None => println!("  Disk Usage: Not available"),
    }

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing how to continuously monitor sandbox metrics.
async fn continuous_monitoring_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Continuous Monitoring Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("monitoring-example").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    println!("Starting continuous monitoring (5 seconds)...");

    // Generate load with a simple and safe command
    cmd.run(
        "sh",
        Some(vec![
            "-c",
            "for i in $(seq 1 5); do ls -la / > /dev/null; sleep 0.2; done &",
        ]),
        None,
    )
    .await?;

    // Get the metrics interface
    let metrics = sandbox.metrics().await?;

    // Monitor for 5 seconds
    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(5) {
        // Get metrics
        let cpu = metrics.cpu().await?;
        let memory = metrics.memory().await?;

        // Format CPU usage (could be 0.0 or None)
        let cpu_str = match cpu {
            Some(value) => format!("{}%", value),
            None => "Not available".to_string(),
        };

        // Format memory usage
        let memory_str = match memory {
            Some(value) => format!("{} MiB", value),
            None => "Not available".to_string(),
        };

        // Print current values
        println!(
            "[{:.1}s] CPU: {}, Memory: {}",
            start_time.elapsed().as_secs_f32(),
            cpu_str,
            memory_str
        );

        // Wait before next check
        sleep(Duration::from_secs(1)).await;
    }

    println!("Monitoring complete.");

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example generating CPU load to test CPU metrics.
async fn cpu_load_test_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== CPU Load Test Example ===");

    // Create a sandbox
    let mut sandbox = PythonSandbox::create("cpu-load-test").await?;

    // Start the sandbox
    sandbox.start(None).await?;

    // Get the command interface
    let cmd = sandbox.command().await?;

    // Run a CPU-intensive Python script
    println!("Running CPU-intensive task...");

    // First create a Python script that will use CPU
    let cpu_script = r#"
import time
start = time.time()
duration = 10  # seconds

# CPU-intensive calculation
while time.time() - start < duration:
    # Calculate prime numbers - CPU intensive
    for i in range(1, 100000):
        is_prime = True
        for j in range(2, int(i ** 0.5) + 1):
            if i % j == 0:
                is_prime = False
                break

    # Print progress every second
    elapsed = time.time() - start
    if int(elapsed) == elapsed:
        print(f"Running for {int(elapsed)} seconds...")

print("CPU load test complete")
"#;

    // Write the script to a file
    cmd.run(
        "bash",
        Some(vec![
            "-c",
            &format!("cat > /tmp/cpu_test.py << 'EOF'\n{}\nEOF", cpu_script),
        ]),
        None,
    )
    .await?;

    // Run the script in the background
    println!("Starting CPU test (running for 10 seconds)...");
    cmd.run("python", Some(vec!["/tmp/cpu_test.py", "&"]), None)
        .await?;

    // Get the metrics interface
    let metrics = sandbox.metrics().await?;

    // Monitor CPU usage while the script runs
    println!("\nMonitoring CPU usage...");
    for i in 0..5 {
        // Wait a moment
        sleep(Duration::from_secs(2)).await;

        // Get metrics
        let cpu = metrics.cpu().await?;
        let memory = metrics.memory().await?;

        // Format CPU usage (could be 0.0 or None)
        let cpu_str = match cpu {
            Some(value) => format!("{}%", value),
            None => "Not available".to_string(),
        };

        // Format memory usage
        let memory_str = match memory {
            Some(value) => format!("{} MiB", value),
            None => "Not available".to_string(),
        };

        // Print current values
        println!(
            "[{} seconds] CPU: {}, Memory: {}",
            i * 2,
            cpu_str,
            memory_str
        );
    }

    println!("CPU load test complete.");

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing error handling with metrics.
async fn error_handling_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Error Handling Example ===");

    // Create a sandbox without starting it
    let sandbox = PythonSandbox::create("error-example").await?;

    // Try to get metrics before starting the sandbox
    println!("Trying to get metrics before starting the sandbox...");
    match sandbox.metrics().await {
        Ok(metrics) => match metrics.cpu().await {
            Ok(cpu) => match cpu {
                Some(value) => println!("CPU: {}%", value),
                None => println!("CPU: Not available"),
            },
            Err(e) => println!("Expected error: {}", e),
        },
        Err(e) => println!("Expected error when getting metrics: {}", e),
    }

    // Now properly start the sandbox
    println!("\nStarting the sandbox properly...");
    let mut sandbox = PythonSandbox::create("error-example").await?;
    sandbox.start(None).await?;

    // Get metrics after starting
    match sandbox.metrics().await {
        Ok(metrics) => {
            match metrics.cpu().await {
                Ok(cpu) => {
                    // Format CPU usage (could be 0.0 or None)
                    let cpu_str = match cpu {
                        Some(value) => format!("{}%", value),
                        None => "Not available".to_string(),
                    };
                    println!("CPU usage after starting: {}", cpu_str);
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Sandbox Metrics Examples");
    println!("=======================");

    // Run all examples
    basic_metrics_example().await?;
    all_metrics_example().await?;
    continuous_monitoring_example().await?;
    cpu_load_test_example().await?;
    error_handling_example().await?;

    println!("\nAll examples completed!");

    Ok(())
}
