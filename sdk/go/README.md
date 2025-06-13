# Microsandbox Go SDK

A Go SDK for interacting with Microsandbox environments.
This SDK provides thread-safe access to running microsandbox environments for code execution, command running, and resource monitoring, without imposing any particular concurrency paradigm.

## Installation

```bash
go get github.com/microsandbox/microsandbox/sdk/go
```

## Quick Start

### Basic Code Execution

```go
package main

import (
    "fmt"
    "log"

    "github.com/microsandbox/microsandbox/sdk/go"
)

func main() {
    // Create a Python sandbox
    sandbox := msb.NewPythonSandbox(
        msb.WithName("my-sandbox"),
        msb.WithReqIdProducer(
            func() string {
                return "Optionally, import UUID package here if desired."
            }),
    )

    // Start the sandbox
    if err := sandbox.Start("", 512, 1); err != nil {
        log.Fatal(err)
    }
    defer sandbox.Stop()

    // Execute code
    execution, err := sandbox.Code().Run("print('Hello from Go SDK!')")
    if err != nil {
        log.Fatal(err)
    }

    // Get the output using rich parsing
    output, err := execution.GetOutput()
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println(output) // prints: Hello from Go SDK!
}
```

### Command Execution

```go
// Execute shell commands
cmdExecution, err := sandbox.Command().Run("ls", []string{"-la", "/"})
if err != nil {
    log.Fatal(err)
}

// Check command success and get output
if cmdExecution.IsSuccess() {
    output, _ := cmdExecution.GetOutput()
    fmt.Println("Directory listing:", output)
} else {
    errorOutput, _ := cmdExecution.GetError()
    fmt.Printf("Command failed with exit code %d: %s\n",
        cmdExecution.GetExitCode(), errorOutput)
}
```

### Resource Metrics

```go
// Get comprehensive metrics
metrics, err := sandbox.Metrics().All()
if err != nil {
    log.Fatal(err)
}

fmt.Printf("CPU: %.2f%%, Memory: %d MiB, Disk: %d bytes\n",
    metrics.CPU, metrics.MemoryMiB, metrics.DiskBytes)

// Or get individual metrics
cpu, err := sandbox.Metrics().CPU()
memory, err := sandbox.Metrics().MemoryMiB()
```

## Advanced Usage

### Concurrent Execution

The SDK is thread-safe and designed for easy integration with a variety of concurrency models:

```go
// Concurrent execution with goroutines
var wg sync.WaitGroup
results := make(chan string, 3)

for i := 0; i < 3; i++ {
    wg.Add(1)
    go func(taskID int) {
        defer wg.Done()

        code := fmt.Sprintf("print('Task %d completed')", taskID)
        execution, err := sandbox.Code().Run(code)
        if err != nil {
            results <- fmt.Sprintf("Task %d failed: %v", taskID, err)
            return
        }

        output, _ := execution.GetOutput()
        results <- fmt.Sprintf("Task %d: %s", taskID, output)
    }(i)
}

go func() {
    wg.Wait()
    close(results)
}()

for result := range results {
    fmt.Println(result)
}
```

### Worker Pool Pattern

```go
// Create a worker pool
tasks := make(chan string, 10)
results := make(chan string, 10)

// Start workers
for i := 0; i < 3; i++ {
    go func(workerID int) {
        for code := range tasks {
            execution, err := sandbox.Code().Run(code)
            if err != nil {
                results <- fmt.Sprintf("Worker %d error: %v", workerID, err)
                continue
            }

            output, _ := execution.GetOutput()
            results <- fmt.Sprintf("Worker %d: %s", workerID, output)
        }
    }(i)
}

// Send tasks
for i := 0; i < 10; i++ {
    tasks <- fmt.Sprintf("print('Processing item %d')", i)
}
close(tasks)
```

### Configuration Options

```go
// Comprehensive configuration
sandbox := msb.NewNodeSandbox(
    msb.WithName("advanced-sandbox"),
    msb.WithServerUrl("http://localhost:5555"),
    msb.WithNamespace("production"),
    msb.WithApiKey("your-api-key"),
    msb.WithLogger(msb.NewDefaultSlogAdapter()),
    msb.WithHTTPClient(&http.Client{
        Timeout: 30 * time.Second,
    }),
)
```

### Logging

The SDK features a lightweight, pluggable logging adapter that allows users to freely configure any logger of their choice.
By default, no logging is applied.

```go
// Enable structured logging
logger := msb.NewDefaultSlogAdapter()
sandbox := msb.NewPythonSandbox(
    msb.WithLogger(logger),
)

// Or use a custom logger
customLogger := msb.NewSlogAdapter(slog.New(slog.NewJSONHandler(os.Stdout, nil)))
```

### Error Handling

```go
execution, err := sandbox.Code().Run("1/0")  // Will cause a Python error
if err != nil {
    log.Printf("Execution failed: %v", err)
    return
}

// Check for execution errors
if execution.HasError() {
    errorOutput, _ := execution.GetError()
    fmt.Printf("Code execution error: %s\n", errorOutput)
    fmt.Printf("Status: %s\n", execution.GetStatus())
}
```

## Configuration

### Environment Variables

- `MSB_API_KEY`: API key for Microsandbox server authentication
- `MSB_SERVER_URL`: Microsandbox server URL (default: `http://127.0.0.1:5555`)

### Start Parameters

```go
// Start with custom resources
err := sandbox.Start(
    "custom-image:latest", // Docker image (empty = language default)
    1024,                  // Memory in MB (0 = default 512MB)
    2,                     // CPU cores (0 = default 1)
)
```

## Examples

See the [examples directory](./cmd/) for comprehensive examples:

- **[command-example.go](./cmd/command-example.go)**: Shell command execution patterns
- **[metrics-example.go](./cmd/metrics-example.go)**: Resource monitoring and load testing
- **[node-example.go](./cmd/node-example.go)**: Node.js code execution and module usage
- **[repl-example.go](./cmd/repl-example.go)**: Python REPL patterns and data processing
- **[concurrent-example.go](./cmd/concurrent-example.go)**: Go-specific concurrency patterns

## Requirements

- Go 1.24
- Running Microsandbox server (default: http://127.0.0.1:5555)
- API key (if authentication is enabled on the server)

## Performance

- **Connection Pooling**: Reuses HTTP connections for efficiency
- **Memory Efficient**: Value types avoid unnecessary heap allocations
- **Structured Parsing**: Parse execution results once, access multiple times
- **Zero Dependencies**: Only uses Go standard library

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
