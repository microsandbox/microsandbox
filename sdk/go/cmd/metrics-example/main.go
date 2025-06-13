package main

import (
	"fmt"
	"log"
	"time"

	msb "github.com/keithang/microsandbox/sdk/go"
)

// basicMetricsExample demonstrates how to get individual metrics for a sandbox.
func basicMetricsExample() {
	fmt.Println("\n=== Basic Metrics Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("metrics-example"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Run commands to generate some load
	fmt.Println("Running commands to generate some sandbox activity...")
	if _, err := sandbox.Command().Run("ls", []string{"-la", "/"}); err != nil {
		log.Printf("Failed to run ls command: %v", err)
	}

	if _, err := sandbox.Command().Run("dd", []string{"if=/dev/zero", "of=/tmp/testfile", "bs=1M", "count=10"}); err != nil {
		log.Printf("Failed to run dd command: %v", err)
	}

	// Sleep a moment to allow metrics to update
	time.Sleep(1 * time.Second)

	// Get individual metrics
	fmt.Println("\nGetting individual metrics for this sandbox:")

	// Get CPU usage
	if cpu, err := sandbox.Metrics().CPU(); err != nil {
		fmt.Printf("Error getting CPU metrics: %v\n", err)
	} else {
		fmt.Printf("CPU Usage: %.2f%%\n", cpu)
	}

	// Get memory usage
	if memory, err := sandbox.Metrics().MemoryMiB(); err != nil {
		fmt.Printf("Error getting memory metrics: %v\n", err)
	} else {
		fmt.Printf("Memory Usage: %d MiB\n", memory)
	}

	// Get disk usage
	if disk, err := sandbox.Metrics().DiskBytes(); err != nil {
		fmt.Printf("Error getting disk metrics: %v\n", err)
	} else {
		fmt.Printf("Disk Usage: %d bytes\n", disk)
	}

	// Check if running
	if running, err := sandbox.Metrics().IsRunning(); err != nil {
		fmt.Printf("Error checking if running: %v\n", err)
	} else {
		fmt.Printf("Is Running: %t\n", running)
	}
}

// allMetricsExample demonstrates how to get all metrics at once.
func allMetricsExample() {
	fmt.Println("\n=== All Metrics Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("all-metrics-example"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Run some commands to generate activity
	fmt.Println("Running commands to generate some sandbox activity...")
	if _, err := sandbox.Command().Run("cat", []string{"/etc/os-release"}); err != nil {
		log.Printf("Failed to run cat command: %v", err)
	}

	if _, err := sandbox.Command().Run("ls", []string{"-la", "/usr"}); err != nil {
		log.Printf("Failed to run ls command: %v", err)
	}

	// Sleep a moment to allow metrics to update
	time.Sleep(1 * time.Second)

	// Get all metrics at once
	fmt.Println("\nGetting all metrics as a single object:")
	allMetrics, err := sandbox.Metrics().All()
	if err != nil {
		log.Fatalf("Failed to get all metrics: %v", err)
	}

	// Print formatted metrics
	fmt.Printf("Sandbox: %s (namespace: %s)\n", allMetrics.Name, allMetrics.Namespace)
	fmt.Printf("  Running: %t\n", allMetrics.IsRunning)
	fmt.Printf("  CPU Usage: %.2f%%\n", allMetrics.CPU)
	fmt.Printf("  Memory Usage: %d MiB\n", allMetrics.MemoryMiB)
	fmt.Printf("  Disk Usage: %d bytes\n", allMetrics.DiskBytes)
}

// continuousMonitoringExample demonstrates how to continuously monitor sandbox metrics.
func continuousMonitoringExample() {
	fmt.Println("\n=== Continuous Monitoring Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("monitoring-example"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	fmt.Println("Starting continuous monitoring (5 seconds)...")

	// Generate load with a simple and safe command (run in background)
	if _, err := sandbox.Command().Run("sh", []string{
		"-c",
		"for i in $(seq 1 5); do ls -la / > /dev/null; sleep 0.2; done &",
	}); err != nil {
		log.Printf("Failed to start background load: %v", err)
	}

	// Monitor for 5 seconds
	startTime := time.Now()
	for time.Since(startTime) < 5*time.Second {
		// Get metrics
		cpu, cpuErr := sandbox.Metrics().CPU()
		memory, memErr := sandbox.Metrics().MemoryMiB()

		// Format and print current values
		elapsed := time.Since(startTime).Seconds()
		cpuStr := "Not available"
		if cpuErr == nil {
			cpuStr = fmt.Sprintf("%.2f%%", cpu)
		}

		memStr := "Not available"
		if memErr == nil {
			memStr = fmt.Sprintf("%d MiB", memory)
		}

		fmt.Printf("[%.1fs] CPU: %s, Memory: %s\n", elapsed, cpuStr, memStr)

		// Wait before next check
		time.Sleep(1 * time.Second)
	}

	fmt.Println("Monitoring complete.")
}

// cpuLoadTestExample generates CPU load to test CPU metrics.
func cpuLoadTestExample() {
	fmt.Println("\n=== CPU Load Test Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("cpu-load-test"),
	)

	if err := sandbox.Start("", 1024, 2); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	fmt.Println("Running CPU-intensive task...")

	// Create a Python script that will use CPU
	cpuScript := `
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
`

	// Write the script to a file
	if _, err := sandbox.Command().Run("bash", []string{"-c", fmt.Sprintf("cat > /tmp/cpu_test.py << 'EOF'\n%s\nEOF", cpuScript)}); err != nil {
		log.Fatalf("Failed to create CPU test script: %v", err)
	}

	// Run the script in the background
	fmt.Println("Starting CPU test (running for 10 seconds)...")
	if _, err := sandbox.Command().Run("python", []string{"/tmp/cpu_test.py", "&"}); err != nil {
		log.Printf("Failed to start CPU test: %v", err)
	}

	// Monitor CPU usage while the script runs
	fmt.Println("\nMonitoring CPU usage...")
	for i := 0; i < 5; i++ {
		// Wait a moment
		time.Sleep(2 * time.Second)

		// Get metrics
		cpu, cpuErr := sandbox.Metrics().CPU()
		memory, memErr := sandbox.Metrics().MemoryMiB()

		// Format and print current values
		cpuStr := "Not available"
		if cpuErr == nil {
			cpuStr = fmt.Sprintf("%.2f%%", cpu)
		}

		memStr := "Not available"
		if memErr == nil {
			memStr = fmt.Sprintf("%d MiB", memory)
		}

		fmt.Printf("[%d seconds] CPU: %s, Memory: %s\n", i*2, cpuStr, memStr)
	}

	fmt.Println("CPU load test complete.")
}

// errorHandlingExample demonstrates error handling with metrics.
func errorHandlingExample() {
	fmt.Println("\n=== Error Handling Example ===")

	// Create a sandbox without starting it immediately
	sandbox := msb.NewPythonSandbox(
		msb.WithName("error-example"),
	)

	// Try to get metrics before starting the sandbox
	fmt.Println("Trying to get metrics before starting the sandbox...")
	if _, err := sandbox.Metrics().CPU(); err != nil {
		fmt.Printf("Expected error: %v\n", err)
	}

	// Now properly start the sandbox
	fmt.Println("\nStarting the sandbox properly...")
	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Get metrics after starting
	if cpu, err := sandbox.Metrics().CPU(); err != nil {
		fmt.Printf("Error getting CPU after starting: %v\n", err)
	} else {
		fmt.Printf("CPU usage after starting: %.2f%%\n", cpu)
	}
}

func main() {
	fmt.Println("Sandbox Metrics Examples")
	fmt.Println("=======================")

	basicMetricsExample()
	allMetricsExample()
	continuousMonitoringExample()
	cpuLoadTestExample()
	errorHandlingExample()

	fmt.Println("\nAll examples completed!")
}

