package msb

import (
	"crypto/rand"
	"errors"
	"fmt"
	"net/http"
	"os"
)

type Option func(*microSandbox)

func WithLanguage(lang ProgLang) Option {
	return func(msb *microSandbox) {
		msb.cfg.lang = lang
	}
}

func WithServerUrl(serverUrl string) Option {
	return func(msb *microSandbox) {
		msb.cfg.serverUrl = serverUrl
	}
}

func WithNamespace(namespace string) Option {
	return func(msb *microSandbox) {
		msb.cfg.namespace = namespace
	}
}

func WithName(name string) Option {
	return func(msb *microSandbox) {
		msb.cfg.name = name
	}
}

func WithApiKey(apiKey string) Option {
	return func(msb *microSandbox) {
		msb.cfg.apiKey = apiKey
	}
}

func WithLogger(logger Logger) Option {
	return func(msb *microSandbox) {
		msb.cfg.logger = logger
	}
}

func WithReqIdProducer(reqIdPrd ReqIdProducer) Option {
	return func(msb *microSandbox) {
		msb.cfg.reqIDPrd = reqIdPrd
	}
}

func WithHTTPClient(c *http.Client) Option {
	return func(msb *microSandbox) {
		msb.rpcClient = newJsonRPCHTTPClient(c)
	}
}

// --- internal constructor operations ---

func fillDefaultConfigs() Option {
	return func(msb *microSandbox) {
		if msb.cfg.lang == LangUnspecified {
			panic(ErrLanguageMustBeSpecified)
		}
		if msb.cfg.serverUrl == "" {
			if envUrl := os.Getenv("MSB_SERVER_URL"); envUrl != "" {
				msb.cfg.serverUrl = envUrl
			} else {
				msb.cfg.serverUrl = defaultServerUrl
			}
		}
		if msb.cfg.namespace == "" {
			if envNamespace := os.Getenv("MSB_NAMESPACE"); envNamespace != "" {
				msb.cfg.namespace = envNamespace
			} else {
				msb.cfg.namespace = defaultNamespace
			}
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
	return func(msb *microSandbox) {
		if msb.cfg.logger == nil {
			msb.cfg.logger = NoOpLogger{}
		}
	}
}

func fillImplementations() Option {
	return func(msb *microSandbox) {
		msb.starter = starter{msb}
		msb.stopper = stopper{msb}
		msb.codeRunner = codeRunner{msb}
		msb.commandRunner = commandRunner{msb}
		msb.metricsReader = metricsReader{msb}
	}
}

func fillDefaultRPCClient() Option {
	return func(msb *microSandbox) {
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
