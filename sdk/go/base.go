package msb

import (
	"errors"
	"sync/atomic"
)

// newBaseWithOptions creates a new [*baseMicroSandbox] instance with the provided configuration options.
// Language must be specified via withLanguage(). API key must be provided via WithApiKey()
// or MSB_API_KEY environment variable.
func newBaseWithOptions(options ...Option) *baseMicroSandbox {
	msb := &baseMicroSandbox{}
	for _, opt := range append(options,
		fillDefaultConfigs(),
		fillDefaultLogger(),
		fillDefaultRPCClient(),
	) {
		opt(msb)
	}
	return msb
}

// container struct that holds state, configs, underpinning all microsandboxes
type baseMicroSandbox struct {
	cfg       config
	state     atomic.Uint32 // we use a lightweight primitive to prevent racing starts / stops; every other method is safe to route concurrently to the underlying (thread-safe) http client
	rpcClient rpcClient
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
