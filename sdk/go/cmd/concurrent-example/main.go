package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"sync"
	"time"

	msb "github.com/keithang/microsandbox/sdk/go"
)

// sequentialExample demonstrates basic sequential usage.
func sequentialExample() {
	fmt.Println("\n=== Sequential Usage Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("sequential-example"),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Sequential execution
	start := time.Now()

	for i := range 3 {
		code := fmt.Sprintf("print('Task %d completed')", i+1)
		execution, err := sandbox.Code().Run(code)
		if err != nil {
			log.Printf("Failed to run task %d: %v", i+1, err)
			continue
		}

		if output, err := execution.GetOutput(); err != nil {
			log.Printf("Failed to get output for task %d: %v", i+1, err)
		} else {
			fmt.Printf("Task %d: %s\n", i+1, output)
		}
	}

	fmt.Printf("Sequential execution took: %v\n", time.Since(start))
}

// goroutineConcurrentExample demonstrates concurrent usage with goroutines.
func goroutineConcurrentExample() {
	fmt.Println("\n=== Goroutine Concurrent Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("concurrent-example"),
	)

	if err := sandbox.Start("", 1024, 2); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Concurrent execution with goroutines
	start := time.Now()
	var wg sync.WaitGroup
	results := make(chan string, 3)

	for i := range 3 {
		wg.Add(1)
		go func(taskID int) {
			defer wg.Done()

			code := fmt.Sprintf(`
import time
time.sleep(0.1)  # Simulate some work
print(f'Concurrent taskID {%v} completed')
`, taskID+1)

			execution, err := sandbox.Code().Run(code)
			if err != nil {
				results <- fmt.Sprintf("Task %d failed: %v", taskID+1, err)
				return
			}

			if output, err := execution.GetOutput(); err != nil {
				results <- fmt.Sprintf("Task %d output error: %v", taskID+1, err)
			} else {
				results <- fmt.Sprintf("Task %d: %s", taskID+1, output)
			}
		}(i)
	}

	// Wait for completion and collect results
	go func() {
		wg.Wait()
		close(results)
	}()

	for result := range results {
		fmt.Println(result)
	}

	fmt.Printf("Concurrent execution took: %v\n", time.Since(start))
}

// workerPoolExample demonstrates the worker pool pattern.
func workerPoolExample() {
	fmt.Println("\n=== Worker Pool Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("worker-pool-example"),
	)

	if err := sandbox.Start("", 1024, 2); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Worker pool pattern
	const numWorkers = 3
	const numTasks = 10

	tasks := make(chan int, numTasks)
	results := make(chan string, numTasks)

	// Start workers
	var wg sync.WaitGroup
	for i := range numWorkers {
		wg.Add(1)
		go func(workerID int) {
			defer wg.Done()

			for taskID := range tasks {
				code := fmt.Sprintf(`
import time
import random
time.sleep(random.uniform(0.1, 0.3))  # Simulate variable work
result = %d * %d
print(f'Worker %d processed task %d: result = {result}')
`, taskID, taskID, workerID, taskID)

				execution, err := sandbox.Code().Run(code)
				if err != nil {
					results <- fmt.Sprintf("Worker %d, Task %d failed: %v", workerID, taskID, err)
					continue
				}

				if output, err := execution.GetOutput(); err != nil {
					results <- fmt.Sprintf("Worker %d, Task %d output error: %v", workerID, taskID, err)
				} else {
					results <- output
				}
			}
		}(i)
	}

	// Send tasks
	start := time.Now()
	for i := 1; i <= numTasks; i++ {
		tasks <- i
	}
	close(tasks)

	// Collect results
	go func() {
		wg.Wait()
		close(results)
	}()

	resultCount := 0
	for result := range results {
		fmt.Println(result)
		resultCount++
	}

	fmt.Printf("Worker pool processed %d tasks in %v\n", resultCount, time.Since(start))
}

// contextCancellationExample demonstrates context-based cancellation.
func contextCancellationExample() {
	fmt.Println("\n=== Context Cancellation Example ===")

	// Create sandbox with custom HTTP client that respects context
	client := &http.Client{
		Timeout: 30 * time.Second,
	}

	sandbox := msb.NewPythonSandbox(
		msb.WithName("context-example"),
		msb.WithHTTPClient(client),
	)

	if err := sandbox.Start("", 512, 1); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Create context with timeout
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	// Channel to signal completion
	done := make(chan struct{})
	var executionErr error

	go func() {
		defer close(done)

		// This would normally take longer than our context timeout
		code := `
import time
print("Starting long-running task...")
time.sleep(5)  # This will be interrupted by context timeout
print("Task completed")  # This won't be reached
`
		_, executionErr = sandbox.Code().Run(code)
	}()

	// Wait for either completion or context cancellation
	select {
	case <-done:
		if executionErr != nil {
			fmt.Printf("Execution completed with error: %v\n", executionErr)
		} else {
			fmt.Println("Execution completed successfully")
		}
	case <-ctx.Done():
		fmt.Printf("Operation cancelled due to context: %v\n", ctx.Err())
		// In a real application, you might want to handle cleanup here
	}
}

// channelCoordinationExample demonstrates using channels for coordination.
func channelCoordinationExample() {
	fmt.Println("\n=== Channel Coordination Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("channel-coordination"),
	)

	if err := sandbox.Start("", 1024, 2); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Channels for coordination
	dataChannel := make(chan string, 5)
	resultChannel := make(chan string, 5)
	errorChannel := make(chan error, 5)

	// Data producer
	go func() {
		defer close(dataChannel)

		for i := 1; i <= 5; i++ {
			data := fmt.Sprintf("data_item_%d", i)
			dataChannel <- data
			fmt.Printf("Produced: %s\n", data)
			time.Sleep(100 * time.Millisecond)
		}
	}()

	// Data processor
	go func() {
		defer close(resultChannel)
		defer close(errorChannel)

		for data := range dataChannel {
			code := fmt.Sprintf(`
# Process data: %s
import json
import hashlib

data = "%s"
processed = {
    "original": data,
    "length": len(data),
    "hash": hashlib.md5(data.encode()).hexdigest()[:8],
    "uppercase": data.upper()
}
print(json.dumps(processed))
`, data, data)

			execution, err := sandbox.Code().Run(code)
			if err != nil {
				errorChannel <- fmt.Errorf("failed to process %s: %w", data, err)
				continue
			}

			if output, err := execution.GetOutput(); err != nil {
				errorChannel <- fmt.Errorf("failed to get output for %s: %w", data, err)
			} else {
				resultChannel <- output
			}
		}
	}()

	// Result collector
	var wg sync.WaitGroup
	wg.Add(2)

	go func() {
		defer wg.Done()
		for result := range resultChannel {
			fmt.Printf("Processed result: %s\n", result)
		}
	}()

	go func() {
		defer wg.Done()
		for err := range errorChannel {
			fmt.Printf("Processing error: %v\n", err)
		}
	}()

	wg.Wait()
	fmt.Println("Channel coordination example completed")
}

// metricsMonitoringExample demonstrates concurrent metrics monitoring.
func metricsMonitoringExample() {
	fmt.Println("\n=== Concurrent Metrics Monitoring Example ===")

	sandbox := msb.NewPythonSandbox(
		msb.WithName("metrics-monitoring"),
	)

	if err := sandbox.Start("", 1024, 2); err != nil {
		log.Fatalf("Failed to start sandbox: %v", err)
	}
	defer func() {
		if err := sandbox.Stop(); err != nil {
			log.Printf("Failed to stop sandbox: %v", err)
		}
	}()

	// Start background workload
	go func() {
		for i := range 10 {
			code := fmt.Sprintf(`
import time
import random

# Simulate variable workload
work_duration = random.uniform(0.2, 0.8)
print(f"Starting work iteration %d (duration: {work_duration:.2f}s)")

# CPU-intensive work
start = time.time()
while time.time() - start < work_duration:
    # Simple CPU work
    sum(range(10000))

print(f"Completed work iteration %d")
`, i+1, i+1)

			if _, err := sandbox.Code().Run(code); err != nil {
				log.Printf("Workload iteration %d failed: %v", i+1, err)
			}
			time.Sleep(200 * time.Millisecond)
		}
	}()

	// Concurrent metrics monitoring
	ctx, cancel := context.WithTimeout(context.Background(), 8*time.Second)
	defer cancel()

	var wg sync.WaitGroup

	// CPU monitor
	wg.Add(1)
	go func() {
		defer wg.Done()
		ticker := time.NewTicker(500 * time.Millisecond)
		defer ticker.Stop()

		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				if cpu, err := sandbox.Metrics().CPU(); err != nil {
					log.Printf("CPU monitoring error: %v", err)
				} else {
					fmt.Printf("[CPU Monitor] CPU: %.2f%%\n", cpu)
				}
			}
		}
	}()

	// Memory monitor
	wg.Add(1)
	go func() {
		defer wg.Done()
		ticker := time.NewTicker(750 * time.Millisecond)
		defer ticker.Stop()

		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				if memory, err := sandbox.Metrics().MemoryMiB(); err != nil {
					log.Printf("Memory monitoring error: %v", err)
				} else {
					fmt.Printf("[Memory Monitor] Memory: %d MiB\n", memory)
				}
			}
		}
	}()

	// Comprehensive metrics monitor
	wg.Add(1)
	go func() {
		defer wg.Done()
		ticker := time.NewTicker(1 * time.Second)
		defer ticker.Stop()

		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				if metrics, err := sandbox.Metrics().All(); err != nil {
					log.Printf("All metrics error: %v", err)
				} else {
					fmt.Printf("[All Metrics] CPU: %.2f%%, Memory: %d MiB, Running: %t\n",
						metrics.CPU, metrics.MemoryMiB, metrics.IsRunning)
				}
			}
		}
	}()

	wg.Wait()
	fmt.Println("Concurrent metrics monitoring completed")
}

func main() {
	fmt.Println("Go Concurrency Examples with Microsandbox")
	fmt.Println("=========================================")

	sequentialExample()
	goroutineConcurrentExample()
	workerPoolExample()
	contextCancellationExample()
	channelCoordinationExample()
	metricsMonitoringExample()

	fmt.Println("\nAll concurrency examples completed!")
}
