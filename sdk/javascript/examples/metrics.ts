/**
 * Example demonstrating how to retrieve sandbox metrics.
 *
 * Before running this example:
 *     1. Install the package: npm install
 *     2. Start the Microsandbox server (microsandbox-server)
 *     3. Run this script: npx ts-node examples/metrics.ts
 */

import { PythonSandbox } from "../src";

/**
 * Example showing how to get individual metrics for a sandbox.
 */
async function basicMetricsExample() {
  console.log("\n=== Basic Metrics Example ===");

  // Create a sandbox using try/finally pattern
  const sandbox = await PythonSandbox.create({ name: "metrics-example" });

  try {
    // Run a command to generate some load
    console.log("Running commands to generate some sandbox activity...");
    await sandbox.command.run("ls", ["-la", "/"]);
    await sandbox.command.run("dd", [
      "if=/dev/zero",
      "of=/tmp/testfile",
      "bs=1M",
      "count=10",
    ]);

    // Sleep a moment to allow metrics to update
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Get individual metrics
    console.log("\nGetting individual metrics for this sandbox:");

    try {
      // Get CPU usage
      const cpu = await sandbox.metrics.cpu();

      // CPU metrics may be 0.0 when idle or undefined if unavailable
      if (cpu === undefined) {
        console.log("CPU Usage: Not available");
      } else {
        console.log(`CPU Usage: ${cpu}%`);
      }

      // Get memory usage
      const memory = await sandbox.metrics.memory();
      console.log(`Memory Usage: ${memory || "Not available"} MiB`);

      // Get disk usage
      const disk = await sandbox.metrics.disk();
      console.log(`Disk Usage: ${disk || "Not available"} bytes`);

      // Check if running
      const running = await sandbox.metrics.isRunning();
      console.log(`Is Running: ${running}`);
    } catch (e) {
      console.log(
        `Error getting metrics: ${e instanceof Error ? e.message : e}`
      );
    }
  } finally {
    await sandbox.stop();
  }
}

/**
 * Example showing how to get all metrics at once.
 */
async function allMetricsExample() {
  console.log("\n=== All Metrics Example ===");

  // Create a sandbox
  const sandbox = await PythonSandbox.create({ name: "all-metrics-example" });

  try {
    // Run some commands to generate activity
    console.log("Running commands to generate some sandbox activity...");
    await sandbox.command.run("cat", ["/etc/os-release"]);

    // Using a simpler command that won't time out or cause errors
    await sandbox.command.run("ls", ["-la", "/usr"]);

    // Sleep a moment to allow metrics to update
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Get all metrics at once
    console.log("\nGetting all metrics as a dictionary:");
    const allMetrics = await sandbox.metrics.all();

    // Print formatted metrics
    console.log(
      `Sandbox: ${allMetrics.name} (namespace: ${allMetrics.namespace})`
    );
    console.log(`  Running: ${allMetrics.running}`);

    // Handle CPU metrics which may be 0.0 or undefined
    const cpu = allMetrics.cpu_usage;
    if (cpu === undefined) {
      console.log("  CPU Usage: Not available");
    } else {
      console.log(`  CPU Usage: ${cpu}%`);
    }

    console.log(
      `  Memory Usage: ${allMetrics.memory_usage || "Not available"} MiB`
    );
    console.log(
      `  Disk Usage: ${allMetrics.disk_usage || "Not available"} bytes`
    );
  } catch (e) {
    console.log(
      `Error in allMetricsExample: ${e instanceof Error ? e.message : e}`
    );
  } finally {
    await sandbox.stop();
  }
}
/**
 * Example showing how to continuously monitor sandbox metrics.
 */
async function continuousMonitoringExample() {
  console.log("\n=== Continuous Monitoring Example ===");

  // Create a sandbox
  const sandbox = await PythonSandbox.create({ name: "monitoring-example" });

  try {
    console.log("Starting continuous monitoring (5 seconds)...");

    // Generate load with a simple and safe command
    await sandbox.command.run("sh", [
      "-c",
      "for i in $(seq 1 5); do ls -la / > /dev/null; sleep 0.2; done &",
    ]);

    // Monitor for 5 seconds
    const startTime = Date.now();
    while (Date.now() - startTime < 5000) {
      try {
        // Get metrics
        const cpu = await sandbox.metrics.cpu();
        const memory = await sandbox.metrics.memory();

        // Format CPU usage (could be 0.0 or undefined)
        const cpuStr = cpu !== undefined ? `${cpu}%` : "Not available";

        // Print current values
        console.log(
          `[${((Date.now() - startTime) / 1000).toFixed(
            1
          )}s] CPU: ${cpuStr}, Memory: ${memory || "Not available"} MiB`
        );
      } catch (e) {
        console.log(
          `Error getting metrics: ${e instanceof Error ? e.message : e}`
        );
      }

      // Wait before next check
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    console.log("Monitoring complete.");
  } catch (e) {
    console.log(
      `Error in continuousMonitoringExample: ${
        e instanceof Error ? e.message : e
      }`
    );
  } finally {
    await sandbox.stop();
  }
}

/**
 * Example generating CPU load to test CPU metrics.
 */
async function cpuLoadTestExample() {

  console.log("\n=== CPU Load Test Example ===");

  // Create a sandbox
  const sandbox = await PythonSandbox.create({ name: "cpu-load-test" });

  try {
    // Run a CPU-intensive Python script
    console.log("Running CPU-intensive task...");

    // First create a Python script that will use CPU
    const cpuScript = `
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
`;
    // Write the script to a file
    await sandbox.command.run("bash", [
      "-c",
      `cat > /tmp/cpu_test.py << 'EOF'\n${cpuScript}\nEOF`,
    ]);

    // Run the script in the background
    console.log("Starting CPU test (running for 10 seconds)...");
    await sandbox.command.run("python", ["/tmp/cpu_test.py", "&"]);

    // Monitor CPU usage while the script runs
    console.log("\nMonitoring CPU usage...");
    for (let i = 0; i < 5; i++) {
      // Wait a moment
      await new Promise((resolve) => setTimeout(resolve, 2000));

      // Get metrics
      const cpu = await sandbox.metrics.cpu();
      const memory = await sandbox.metrics.memory();

      // Format CPU usage (could be 0.0 or undefined)
      const cpuStr = cpu !== undefined ? `${cpu}%` : "Not available";

      // Print current values
      console.log(
        `[${i * 2} seconds] CPU: ${cpuStr}, Memory: ${
          memory || "Not available"
        } MiB`
      );
    }

    console.log("CPU load test complete.");
  } catch (e) {
    console.log(
      `Error in cpuLoadTestExample: ${e instanceof Error ? e.message : e}`
    );
  } finally {
    await sandbox.stop();
  }
}

/**
 * Example showing error handling with metrics.
 */
async function errorHandlingExample() {
  console.log("\n=== Error Handling Example ===");

  // Create a sandbox without starting it immediately
  const sandbox = new PythonSandbox({ name: "error-example" });

  try {
    // Try to get metrics before starting the sandbox
    console.log("Trying to get metrics before starting the sandbox...");
    try {
      const cpu = await sandbox.metrics.cpu();
      console.log(`CPU: ${cpu}%`); // This shouldn't be reached
    } catch (e) {
      console.log(`Expected error: ${e instanceof Error ? e.message : e}`);
    }

    // Now properly start the sandbox
    console.log("\nStarting the sandbox properly...");
    await sandbox.start();

    // Get metrics after starting
    const cpu = await sandbox.metrics.cpu();

    // Format CPU usage (could be 0.0 or undefined)
    const cpuStr = cpu !== undefined ? `${cpu}%` : "Not available";
    console.log(`CPU usage after starting: ${cpuStr}`);
  } catch (e) {
    console.log(`Error: ${e instanceof Error ? e.message : e}`);
  } finally {
    // Clean up
    if (sandbox.isStarted) {
      await sandbox.stop();
    }
  }
}

async function main() {
  console.log("Sandbox Metrics Examples");
  console.log("=======================");

  try {
    await basicMetricsExample();
    await allMetricsExample();
    await continuousMonitoringExample();
    await cpuLoadTestExample();
    await errorHandlingExample();

    console.log("\nAll metrics examples completed!");
  } catch (e) {
    console.error(
      `Error running examples: ${e instanceof Error ? e.message : e}`
    );
  }
}

main();
