package msb

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"sync/atomic"
)

// MicroSandbox provides a sandbox environment for executing code and commands safely.
// All methods are thread-safe and will block on network calls to the Microsandbox server.
// The underlying HTTP client uses connection pooling for efficient reuse.
type MicroSandbox interface {
	Starter
	Stopper
	CodeRunner
	CommandRunner
	MetricsReader
}

// NewWithOptions creates a new MicroSandbox instance with the provided configuration options.
// Language must be specified via WithLanguage(). API key must be provided via WithApiKey()
// or MSB_API_KEY environment variable.
func NewWithOptions(options ...Option) MicroSandbox {
	msb := &microSandbox{}
	for _, opt := range append(options,
		fillDefaultConfigs(),
		fillDefaultLogger(),
		fillDefaultRPCClient(),
		fillImplementations(),
	) {
		opt(msb)
	}
	return msb
}

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
		// RunCode executes the provided code and returns detailed execution results.
		// The sandbox must be started before calling this method.
		RunCode(code string) (CodeExecution, error)
	}

	// CommandRunner executes shell commands in the sandbox.
	CommandRunner interface {
		// RunCommand executes a shell command with the given arguments.
		// The sandbox must be started before calling this method.
		RunCommand(cmd string, args []string) (CommandExecution, error)
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
)

// Metrics contains resource usage information for a sandbox.
type Metrics struct {
	Name      string  // Sandbox name
	Namespace string  // Sandbox namespace
	IsRunning bool    // Whether the sandbox is currently running
	CPU       float64 // CPU usage percentage (0-100)
	MemoryMiB int     // Memory usage in mebibytes
	DiskBytes int     // Disk usage in bytes
}

// --- API Implementation ---

// container parent struct that holds state, configs, embeds implementation structs
type microSandbox struct {
	cfg       config
	state     atomic.Uint32 // we use a lightweight primitive to prevent racing starts / stops; every other method is safe to route concurrently to the underlying (thread-safe) http client
	rpcClient rpcClient
	starter
	stopper
	codeRunner
	commandRunner
	metricsReader
}

type starter struct {
	*microSandbox
}

func (s starter) Start(image string, memoryMB int, cpus int) error {
	if s.state.Load() == started {
		return ErrSandboxAlreadyStarted
	}
	if image == "" {
		image = s.cfg.lang.DefaultImage()
	}
	if memoryMB <= 0 {
		memoryMB = 512
	}
	if cpus <= 0 {
		cpus = 1
	}
	err := s.rpcClient.startSandbox(context.Background(), &s.cfg, image, memoryMB, cpus)
	if err != nil {
		return fmt.Errorf("%w: %w", ErrFailedToStartSandbox, err)
	}
	s.state.Store(started)
	return nil
}

type stopper struct {
	*microSandbox
}

func (s stopper) Stop() error {
	if s.state.Load() == off {
		return ErrSandboxNotStarted
	}
	ctx := context.Background()
	err := s.rpcClient.stopSandbox(ctx, &s.cfg)
	if err != nil {
		return fmt.Errorf("%w: %w", ErrFailedToStopSandbox, err)
	}
	s.state.Store(off)
	return nil
}

type codeRunner struct {
	*microSandbox
}

func (cr codeRunner) RunCode(code string) (CodeExecution, error) {
	if cr.state.Load() != started {
		return CodeExecution{}, ErrSandboxNotStarted
	}
	ctx := context.Background()
	result, err := cr.rpcClient.runRepl(ctx, &cr.cfg, code)
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
	*microSandbox
}

func (cr commandRunner) RunCommand(cmd string, args []string) (CommandExecution, error) {
	if cr.state.Load() != started {
		return CommandExecution{}, ErrSandboxNotStarted
	}
	ctx := context.Background()
	result, err := cr.rpcClient.runCommand(ctx, &cr.cfg, cmd, args)
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
	*microSandbox
}

func (mr metricsReader) All() (Metrics, error) {
	if mr.state.Load() != started {
		return Metrics{}, ErrSandboxNotStarted
	}

	ctx := context.Background()
	metrics, err := mr.rpcClient.getMetrics(ctx, &mr.cfg)
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

var (
	ErrSandboxAlreadyStarted = errors.New("sandbox already started")
	ErrSandboxNotStarted     = errors.New("sandbox not started")
	ErrFailedToStartSandbox  = errors.New("failed to start sandbox")
	ErrFailedToStopSandbox   = errors.New("failed to stop sandbox")
	ErrFailedToRunCode       = errors.New("failed to run code")
	ErrFailedToRunCommand    = errors.New("failed to run command")
	ErrFailedToGetMetrics    = errors.New("failed to get metrics")
)
