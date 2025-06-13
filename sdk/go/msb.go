// Package msb provides a Go SDK for interacting with Microsandbox environments.
//
// This SDK provides thread-safe access to running microsandbox environments for code execution,
// command running, and resource monitoring, without imposing any particular concurrency paradigm.
//
// # Quick Start
//
// Create a Python sandbox:
//
//	sandbox := msb.NewPythonSandbox(msb.WithName("my-sandbox"))
//	if err := sandbox.Start("", 512, 1); err != nil {
//		log.Fatal(err)
//	}
//	defer sandbox.Stop()
//
// Execute Python code:
//
//	execution, err := sandbox.Code().Run("print('Hello World')")
//	if err != nil {
//		log.Fatal(err)
//	}
//	output, _ := execution.GetOutput()
//	fmt.Println(output)
//
// Run shell commands:
//
//	cmdExec, err := sandbox.Command().Run("ls", []string{"-la"})
//	if err != nil {
//		log.Fatal(err)
//	}
//
// Monitor resource usage:
//
//	metrics, err := sandbox.Metrics().All()
//	if err != nil {
//		log.Fatal(err)
//	}
//	fmt.Printf("CPU: %.2f%%, Memory: %d MiB\n", metrics.CPU, metrics.MemoryMiB)
package msb

import (
	"context"
	"encoding/json"
	"fmt"
)

// Core sandbox interfaces
type (
	// Starter manages sandbox lifecycle startup.
	Starter interface {
		// Start initializes the sandbox with the specified configuration.
		// If image is empty, uses the default image for the configured language.
		// If memoryMB <= 0, defaults to 512. If cpus <= 0, defaults to 1.
		Start(image string, memoryMB int, cpus int) error
	}

	// Stopper manages sandbox lifecycle shutdown.
	Stopper interface {
		// Stop terminates the sandbox and releases its resources.
		Stop() error
	}

	// CodeRunner executes code in the sandbox's REPL environment.
	CodeRunner interface {
		// Run executes the provided code and returns detailed execution results.
		// The sandbox must be started before calling this method.
		Run(code string) (CodeExecution, error)
	}

	// CommandRunner executes shell commands in the sandbox.
	CommandRunner interface {
		// Run executes a shell command with the given arguments.
		// The sandbox must be started before calling this method.
		Run(cmd string, args []string) (CommandExecution, error)
	}

	// MetricsReader provides access to sandbox resource metrics.
	MetricsReader interface {
		// All returns comprehensive metrics for the sandbox.
		All() (Metrics, error)
		// CPU returns current CPU usage as a percentage (0-100).
		CPU() (float64, error)
		// MemoryMiB returns current memory usage in mebibytes.
		MemoryMiB() (int, error)
		// DiskBytes returns current disk usage in bytes.
		DiskBytes() (int, error)
		// IsRunning reports whether the sandbox is currently running.
		IsRunning() (bool, error)
	}

	// Metrics contains resource usage information for a sandbox.
	Metrics struct {
		Name      string  // Sandbox name
		Namespace string  // Sandbox namespace
		IsRunning bool    // Whether the sandbox is currently running
		CPU       float64 // CPU usage percentage (0-100)
		MemoryMiB int     // Memory usage in mebibytes
		DiskBytes int     // Disk usage in bytes
	}
)

// --- API Implementation ---

type starter struct {
	b *baseMicroSandbox
}

func (s starter) Start(image string, memoryMB int, cpus int) error {
	if s.b.state.Load() == started {
		return ErrSandboxAlreadyStarted
	}
	if memoryMB <= 0 {
		memoryMB = 512
	}
	if cpus <= 0 {
		cpus = 1
	}
	err := s.b.rpcClient.startSandbox(context.Background(), &s.b.cfg, image, memoryMB, cpus)
	if err != nil {
		return fmt.Errorf("%w: %w", ErrFailedToStartSandbox, err)
	}
	s.b.state.Store(started)
	return nil
}

type stopper struct {
	b *baseMicroSandbox
}

func (s stopper) Stop() error {
	if s.b.state.Load() == off {
		return ErrSandboxNotStarted
	}
	ctx := context.Background()
	err := s.b.rpcClient.stopSandbox(ctx, &s.b.cfg)
	if err != nil {
		return fmt.Errorf("%w: %w", ErrFailedToStopSandbox, err)
	}
	s.b.state.Store(off)
	return nil
}

type codeRunner struct {
	b *baseMicroSandbox
	l progLang
}

func (cr codeRunner) Run(code string) (CodeExecution, error) {
	if cr.b.state.Load() != started {
		return CodeExecution{}, ErrSandboxNotStarted
	}
	ctx := context.Background()
	result, err := cr.b.rpcClient.runRepl(ctx, &cr.b.cfg, cr.l, code)
	if err != nil {
		return CodeExecution{}, fmt.Errorf("%w: %w", ErrFailedToRunCode, err)
	}

	exec := CodeExecution{Output: result.output}
	// Parse the output for convenience methods
	if err := json.Unmarshal(result.output, &exec.parsed); err == nil {
		exec.parsedOK = true
	}

	return exec, nil
}

type commandRunner struct {
	b *baseMicroSandbox
}

func (cr commandRunner) Run(cmd string, args []string) (CommandExecution, error) {
	if cr.b.state.Load() != started {
		return CommandExecution{}, ErrSandboxNotStarted
	}
	ctx := context.Background()
	result, err := cr.b.rpcClient.runCommand(ctx, &cr.b.cfg, cmd, args)
	if err != nil {
		return CommandExecution{}, fmt.Errorf("%w: %w", ErrFailedToRunCommand, err)
	}

	exec := CommandExecution{Output: result.output}
	// Parse the output for convenience methods
	if err := json.Unmarshal(result.output, &exec.parsed); err == nil {
		exec.parsedOK = true
	}

	return exec, nil
}

type metricsReader struct {
	b *baseMicroSandbox
}

func (mr metricsReader) All() (Metrics, error) {
	if mr.b.state.Load() != started {
		return Metrics{}, ErrSandboxNotStarted
	}

	ctx := context.Background()
	metrics, err := mr.b.rpcClient.getMetrics(ctx, &mr.b.cfg)
	if err != nil {
		return Metrics{}, fmt.Errorf("%w: %w", ErrFailedToGetMetrics, err)
	}

	return Metrics{
		Name:      metrics.Name,
		Namespace: metrics.Namespace,
		IsRunning: metrics.Running,
		CPU:       metrics.CPUUsage,
		MemoryMiB: metrics.MemoryUsage,
		DiskBytes: metrics.DiskUsage,
	}, nil
}

func (mr metricsReader) CPU() (float64, error) {
	metrics, err := mr.All()
	if err != nil {
		return 0, err
	}
	return metrics.CPU, nil
}

func (mr metricsReader) MemoryMiB() (int, error) {
	metrics, err := mr.All()
	if err != nil {
		return 0, err
	}
	return metrics.MemoryMiB, nil
}

func (mr metricsReader) DiskBytes() (int, error) {
	metrics, err := mr.All()
	if err != nil {
		return 0, err
	}
	return metrics.DiskBytes, nil
}

func (mr metricsReader) IsRunning() (bool, error) {
	metrics, err := mr.All()
	if err != nil {
		return false, err
	}
	return metrics.IsRunning, nil
}
