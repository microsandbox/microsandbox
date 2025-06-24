package msb

import (
	"crypto/rand"
	"errors"
	"fmt"
	"net/http"
	"os"
)

// Option configures a sandbox during creation.
// Options are applied in the order they are provided to NewPythonSandbox or NewNodeSandbox.
type Option func(*baseMicroSandbox)

// WithServerUrl configures the Microsandbox server URL.
// If not specified, defaults to MSB_SERVER_URL environment variable or http://127.0.0.1:5555.
func WithServerUrl(serverUrl string) Option {
	return func(msb *baseMicroSandbox) {
		msb.cfg.serverUrl = serverUrl
	}
}

// WithNamespace configures the sandbox namespace for isolation.
// If not specified, uses the default namespace.
func WithNamespace(namespace string) Option {
	return func(msb *baseMicroSandbox) {
		msb.cfg.namespace = namespace
	}
}

// WithName sets a custom name for the sandbox instance.
// If not specified, a random name will be generated.
func WithName(name string) Option {
	return func(msb *baseMicroSandbox) {
		msb.cfg.name = name
	}
}

// WithApiKey configures the API key for server authentication.
// If not specified, uses the MSB_API_KEY environment variable.
func WithApiKey(apiKey string) Option {
	return func(msb *baseMicroSandbox) {
		msb.cfg.apiKey = apiKey
	}
}

// WithLogger configures a custom logger for the sandbox.
// If not specified, uses a no-op logger that discards all log output.
func WithLogger(logger Logger) Option {
	return func(msb *baseMicroSandbox) {
		msb.cfg.logger = logger
	}
}

// WithReqIdProducer configures a custom request ID generator for tracing.
// Request IDs are included in logs and can help with debugging.
func WithReqIdProducer(reqIdPrd ReqIdProducer) Option {
	return func(msb *baseMicroSandbox) {
		msb.cfg.reqIDPrd = reqIdPrd
	}
}

// WithHTTPClient configures a custom HTTP client for server communication.
// Useful for setting timeouts, proxies, or other HTTP-level configuration.
func WithHTTPClient(c *http.Client) Option {
	return func(msb *baseMicroSandbox) {
		msb.rpcClient = newJsonRPCHTTPClient(c)
	}
}

// --- internal constructor operations ---

func fillDefaultConfigs() Option {
	return func(msb *baseMicroSandbox) {
		if msb.cfg.serverUrl == "" {
			if envUrl := os.Getenv("MSB_SERVER_URL"); envUrl != "" {
				msb.cfg.serverUrl = envUrl
			} else {
				msb.cfg.serverUrl = defaultServerUrl
			}
		}
		if msb.cfg.namespace == "" {
			msb.cfg.namespace = defaultNamespace
		}
		if msb.cfg.name == "" {
			b := make([]byte, 4) // 4 bytes == 8 hex chars
			if _, err := rand.Read(b); err != nil {
				panic(fmt.Errorf("%w: %w", ErrFailedToGenerateRandomName, err))
			}
			msb.cfg.name = fmt.Sprintf(defaultNameTemplate, b)
		}
		if msb.cfg.apiKey == "" {
			if envApiKey := os.Getenv("MSB_API_KEY"); envApiKey != "" {
				msb.cfg.apiKey = envApiKey
			} else {
				panic(ErrAPIKeyMustBeSpecified)
			}
		}
	}
}

func fillDefaultLogger() Option {
	return func(msb *baseMicroSandbox) {
		if msb.cfg.logger == nil {
			msb.cfg.logger = NoOpLogger{}
		}
	}
}

func fillDefaultRPCClient() Option {
	return func(msb *baseMicroSandbox) {
		if msb.rpcClient == nil {
			msb.rpcClient = newDefaultJsonRPCHTTPClient()
		}
	}
}

// Option-related errors
var (
	ErrLanguageMustBeSpecified    = errors.New("language must be specified")
	ErrFailedToGenerateRandomName = errors.New("failed to generate random name")
	ErrAPIKeyMustBeSpecified      = errors.New("API key must be specified either via WithApiKey() or MSB_API_KEY environment variable")
)
