package msb

import (
	"encoding/json"
	"strings"
)

// CommandExecution represents the result of command execution in the sandbox.
// Use the Get* methods for parsed access to output, or access Output directly for raw JSON.
type CommandExecution struct {
	Output    json.RawMessage // Raw JSON response from the server
	parsed    commandData     // Parsed data for convenience methods
	parsedOK  bool           // Whether parsing succeeded
}

// Internal structure for parsing command execution results
type commandData struct {
	OutputLines []outputLine `json:"output"`
	Command     string       `json:"command"`
	Args        []string     `json:"args"`
	ExitCode    int          `json:"exit_code"`
	Success     bool         `json:"success"`
}

// GetOutput returns the standard output from command execution as a string.
// Returns ErrExecutionNotParsed if the raw JSON could not be parsed.
func (ce CommandExecution) GetOutput() (string, error) {
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

// GetError returns the error output from command execution as a string.
// Returns ErrExecutionNotParsed if the raw JSON could not be parsed.
func (ce CommandExecution) GetError() (string, error) {
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

// GetExitCode returns the exit code of the executed command.
// Returns -1 if the raw JSON could not be parsed.
func (ce CommandExecution) GetExitCode() int {
	if !ce.parsedOK {
		return -1
	}
	return ce.parsed.ExitCode
}

// IsSuccess reports whether the command executed successfully (exit code 0).
// Returns false if the raw JSON could not be parsed.
func (ce CommandExecution) IsSuccess() bool {
	if !ce.parsedOK {
		return false
	}
	return ce.parsed.Success
}

// GetCommand returns the command that was executed.
// Returns empty string if the raw JSON could not be parsed.
func (ce CommandExecution) GetCommand() string {
	if !ce.parsedOK {
		return ""
	}
	return ce.parsed.Command
}

// GetArgs returns the arguments used for the command.
// Returns nil if the raw JSON could not be parsed.
func (ce CommandExecution) GetArgs() []string {
	if !ce.parsedOK {
		return nil
	}
	return ce.parsed.Args
}