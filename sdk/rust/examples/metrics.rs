//! Example demonstrating how to use sandbox.metrics.get to retrieve sandbox metrics.
//!
//! This example shows:
//! 1. Basic metrics retrieval
//! 2. Monitoring metrics over time
//! 3. Resources usage monitoring during different operations
//!
//! Before running this example:
//!     1. Install the package as a dependency
//!     2. Start the Microsandbox server (microsandbox-server)
//!     3. Run this script: cargo run --example metrics
//!
//! Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.

use microsandbox::{PythonSandbox, SandboxOptions};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use tokio::time::sleep;

/// Example showing basic metrics retrieval.
async fn basic_metrics_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Basic Metrics Example ===");

    // Create a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("metrics-example").build()).await?;

    // Get initial sandbox metrics
    println!("Getting initial sandbox metrics...");
    let metrics = sandbox.metrics().get().await?;

    println!("CPU usage: {:.2}%", metrics.cpu_usage());
    println!(
        "Memory usage: {:.2} MB",
        metrics.memory_usage() as f64 / 1_000_000.0
    );
    println!(
        "Disk usage: {:.2} MB",
        metrics.disk_usage() as f64 / 1_000_000.0
    );

    // Run some code to generate some activity
    println!("\nRunning a CPU-intensive task...");
    let _ = sandbox
        .run(
            r#"
import time
import math

# Perform some CPU-intensive calculations
start = time.time()
for i in range(1000000):
    math.sqrt(i)
end = time.time()
print(f"Calculation took {end - start:.2f} seconds")
    "#,
        )
        .await?;

    // Wait a moment for metrics to update
    sleep(Duration::from_secs(1)).await;

    // Get updated metrics after running the code
    println!("\nGetting updated metrics after CPU task...");
    let metrics = sandbox.metrics().get().await?;

    println!("CPU usage: {:.2}%", metrics.cpu_usage());
    println!(
        "Memory usage: {:.2} MB",
        metrics.memory_usage() as f64 / 1_000_000.0
    );
    println!(
        "Disk usage: {:.2} MB",
        metrics.disk_usage() as f64 / 1_000_000.0
    );

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing how to monitor sandbox metrics over time.
async fn monitoring_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Monitoring Metrics Example ===");

    // Create a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("monitoring-example").build()).await?;

    // Monitor metrics while running various operations
    println!("Starting monitoring...");

    // Get baseline metrics
    let baseline = sandbox.metrics().get().await?;
    println!(
        "Baseline - Memory: {:.2} MB, CPU: {:.2}%",
        baseline.memory_usage() as f64 / 1_000_000.0,
        baseline.cpu_usage()
    );

    // Run a memory-intensive operation
    println!("\nRunning memory-intensive operation...");
    let _ = sandbox
        .run(
            r#"
# Allocate a large list in memory
large_list = ["x" * 1000 for _ in range(1000000)]
print(f"List created with {len(large_list)} elements")
    "#,
        )
        .await?;

    // Check metrics after memory operation
    sleep(Duration::from_secs(1)).await;
    let mem_metrics = sandbox.metrics().get().await?;
    println!(
        "After memory operation - Memory: {:.2} MB, CPU: {:.2}%",
        mem_metrics.memory_usage() as f64 / 1_000_000.0,
        mem_metrics.cpu_usage()
    );

    // Run a disk operation
    println!("\nRunning disk operation...");
    let _ = sandbox
        .command()
        .run(
            "dd",
            Some(vec![
                "if=/dev/zero",
                "of=/tmp/testfile",
                "bs=1M",
                "count=100",
            ]),
            None,
        )
        .await?;
    let _ = sandbox.command().run("sync", None, None).await?;

    // Check metrics after disk operation
    sleep(Duration::from_secs(1)).await;
    let disk_metrics = sandbox.metrics().get().await?;
    println!(
        "After disk operation - Disk: {:.2} MB, Memory: {:.2} MB",
        disk_metrics.disk_usage() as f64 / 1_000_000.0,
        disk_metrics.memory_usage() as f64 / 1_000_000.0
    );

    // Clean up
    let _ = sandbox
        .command()
        .run("rm", Some(vec!["-f", "/tmp/testfile"]), None)
        .await?;

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing more advanced metrics usage.
async fn advanced_metrics_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Advanced Metrics Example ===");

    // Create a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("advanced-example").build()).await?;

    // Perform continuous monitoring for a short period
    println!("Continuous monitoring for 10 seconds:");

    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(10) {
        // Run a short task to create some activity
        let _ = sandbox
            .run("import random; [random.random() for _ in range(100000)]")
            .await?;

        // Get current metrics
        let metrics = sandbox.metrics().get().await?;
        println!(
            "Time: {:.1}s, CPU: {:.2}%, Memory: {:.2} MB",
            start_time.elapsed().as_secs_f64(),
            metrics.cpu_usage(),
            metrics.memory_usage() as f64 / 1_000_000.0
        );

        // Short pause between measurements
        sleep(Duration::from_secs(1)).await;
    }

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

/// Example showing network usage metrics.
async fn network_metrics_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n=== Network Metrics Example ===");

    // Create a sandbox
    let mut sandbox =
        PythonSandbox::create(SandboxOptions::builder().name("network-example").build()).await?;

    // Get initial network metrics
    let initial_metrics = sandbox.metrics().get().await?;
    println!(
        "Initial network usage: {:.2} MB",
        initial_metrics.network_usage() as f64 / 1_000_000.0
    );

    // Run some code that generates network traffic
    println!("\nGenerating network traffic...");
    let _ = sandbox
        .run(
            r#"
import urllib.request
try:
    # Download a small file to generate network traffic
    urllib.request.urlretrieve('http://example.com/', '/tmp/example.html')
    print("File downloaded successfully")

    # Read the file to confirm download
    with open('/tmp/example.html', 'r') as f:
        content = f.read()
        print(f"Downloaded {len(content)} bytes")
except Exception as e:
    print(f"Error: {e}")
    "#,
        )
        .await?;

    // Wait a moment for metrics to update
    sleep(Duration::from_secs(1)).await;

    // Get updated network metrics
    let updated_metrics = sandbox.metrics().get().await?;
    println!(
        "Updated network usage: {:.2} MB",
        updated_metrics.network_usage() as f64 / 1_000_000.0
    );

    // Calculate the difference
    let network_diff =
        updated_metrics.network_usage() as i64 - initial_metrics.network_usage() as i64;
    println!(
        "Network usage difference: {:.2} KB",
        network_diff as f64 / 1_000.0
    );

    // Stop the sandbox
    sandbox.stop().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Sandbox Metrics Examples");
    println!("=======================");

    // Run each example
    basic_metrics_example().await?;
    monitoring_example().await?;
    network_metrics_example().await?;
    advanced_metrics_example().await?;

    println!("\nAll examples completed!");
    Ok(())
}
