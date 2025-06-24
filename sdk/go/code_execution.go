package msb

import (
	"encoding/json"
	"errors"
	"strings"
)

// ErrExecutionNotParsed is returned when execution output could not be parsed.
var ErrExecutionNotParsed = errors.New("execution output could not be parsed")

// CodeExecution represents the result of code execution in the sandbox.
// Use the Get* methods for parsed access to output, or access Output directly for raw JSON.
type CodeExecution struct {
	Output   json.RawMessage // Raw JSON response from the server
	parsed   executionData   // Parsed data for convenience methods
	parsedOK bool            // Whether parsing succeeded
}

// Internal structures for parsing execution results
type (
	executionData struct {
		OutputLines []outputLine `json:"output"`
		Status      string       `json:"status"`
		Language    string       `json:"language"`
	}

	outputLine struct {
		Stream string `json:"stream"`
		Text   string `json:"text"`
	}
)

// GetOutput returns the standard output from code execution as a string.
// Returns ErrExecutionNotParsed if the raw JSON could not be parsed.
func (ce CodeExecution) GetOutput() (string, error) {
	if !ce.parsedOK {
		return "", ErrExecutionNotParsed
	}

	var output strings.Builder
	for _, line := range ce.parsed.OutputLines {
		if line.Stream == "stdout" {
			output.WriteString(line.Text)
			output.WriteString("\n")
		}
	}
	return strings.TrimSuffix(output.String(), "\n"), nil
}

// GetError returns the error output from code execution as a string.
// Returns ErrExecutionNotParsed if the raw JSON could not be parsed.
func (ce CodeExecution) GetError() (string, error) {
	if !ce.parsedOK {
		return "", ErrExecutionNotParsed
	}

	var errorOutput strings.Builder
	for _, line := range ce.parsed.OutputLines {
		if line.Stream == "stderr" {
			errorOutput.WriteString(line.Text)
			errorOutput.WriteString("\n")
		}
	}
	return strings.TrimSuffix(errorOutput.String(), "\n"), nil
}

// HasError reports whether the code execution encountered an error.
// Checks both execution status and presence of stderr output.
func (ce CodeExecution) HasError() bool {
	if !ce.parsedOK {
		return false
	}

	// Check status for error or exception
	if ce.parsed.Status == "error" || ce.parsed.Status == "exception" {
		return true
	}

	// Check for stderr output
	for _, line := range ce.parsed.OutputLines {
		if line.Stream == "stderr" && line.Text != "" {
			return true
		}
	}
	return false
}

// GetStatus returns the execution status (e.g., "success", "error", "exception").
// Returns "unknown" if the raw JSON could not be parsed.
func (ce CodeExecution) GetStatus() string {
	if !ce.parsedOK {
		return "unknown"
	}
	return ce.parsed.Status
}

// GetLanguage returns the language used for code execution.
// Returns "unknown" if the raw JSON could not be parsed.
func (ce CodeExecution) GetLanguage() string {
	if !ce.parsedOK {
		return "unknown"
	}
	return ce.parsed.Language
}

