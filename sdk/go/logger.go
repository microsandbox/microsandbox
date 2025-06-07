package msb

import (
	"io"
	"log/slog"
)

// Logger provides a minimal logging interface for the SDK.
// This allows users to plug in their preferred logging library while keeping
// the SDK dependency-free by default.
type Logger interface {
	// Debug logs debug-level messages with optional key-value pairs.
	Debug(msg string, args ...any)
	// Info logs info-level messages with optional key-value pairs.
	Info(msg string, args ...any)
	// Error logs error-level messages with optional key-value pairs.
	Error(msg string, args ...any)
}

// NoOpLogger is a logger that discards all log messages.
// This is used as the default logger to avoid forcing logging on users.
type NoOpLogger struct{}

// Debug discards the debug message.
func (NoOpLogger) Debug(msg string, args ...any) {}

// Info discards the info message.
func (NoOpLogger) Info(msg string, args ...any) {}

// Error discards the error message.
func (NoOpLogger) Error(msg string, args ...any) {}

// SlogAdapter adapts the standard library's slog.Logger to the SDK's Logger interface.
// This allows users to easily integrate with the structured logging provided by slog.
type SlogAdapter struct {
	*slog.Logger
}

// Debug logs a debug-level message using slog.
func (s SlogAdapter) Debug(msg string, args ...any) {
	s.Logger.Debug(msg, args...)
}

// Info logs an info-level message using slog.
func (s SlogAdapter) Info(msg string, args ...any) {
	s.Logger.Info(msg, args...)
}

// Error logs an error-level message using slog.
func (s SlogAdapter) Error(msg string, args ...any) {
	s.Logger.Error(msg, args...)
}

// NewSlogAdapter creates a new SlogAdapter with the given slog.Logger.
// If logger is nil, it creates a new slog.Logger that discards all output.
func NewSlogAdapter(logger *slog.Logger) SlogAdapter {
	if logger == nil {
		// Create a no-op slog logger
		logger = slog.New(slog.NewTextHandler(io.Discard, nil))
	}
	return SlogAdapter{Logger: logger}
}

// NewDefaultSlogAdapter creates a new SlogAdapter with a default text handler
// that writes to the given writer. If w is nil, output is discarded.
func NewDefaultSlogAdapter() SlogAdapter {
	return SlogAdapter{Logger: slog.Default()}
}