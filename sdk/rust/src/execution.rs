//! Execution results for code run in sandboxes

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

/// Represents a code execution in a sandbox environment
///
/// This struct provides access to the results and output of code
/// that was executed in a sandbox.
#[derive(Debug, Clone)]
pub struct Execution {
    /// Output lines from the execution
    output_lines: Vec<OutputLine>,
    /// Status of the execution
    status: String,
    /// Language used for the execution
    language: String,
    /// Whether the execution encountered an error
    has_error: bool,
}

/// A single line of output from an execution
#[derive(Debug, Clone)]
struct OutputLine {
    /// Stream type (stdout or stderr)
    stream: String,
    /// Text content
    text: String,
}

impl Execution {
    /// Create a new execution instance from output data
    pub(crate) fn new(output_data: HashMap<String, Value>) -> Self {
        let mut output_lines = Vec::new();
        let mut has_error = false;
        let status = output_data
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let language = output_data
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Check if status indicates an error
        if status == "error" || status == "exception" {
            has_error = true;
        }

        // Process output lines
        if let Some(output) = output_data.get("output") {
            if let Some(lines) = output.as_array() {
                for line in lines {
                    if let Some(line_obj) = line.as_object() {
                        let stream = line_obj
                            .get("stream")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let text = line_obj
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        // Check for errors in stderr
                        if stream == "stderr" && !text.is_empty() {
                            has_error = true;
                        }

                        output_lines.push(OutputLine { stream, text });
                    }
                }
            }
        }

        Self {
            output_lines,
            status,
            language,
            has_error,
        }
    }

    /// Get the standard output from the execution
    pub async fn output(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut output_text = String::new();

        for line in &self.output_lines {
            if line.stream == "stdout" {
                output_text.push_str(&line.text);
                output_text.push('\n');
            }
        }

        // Remove trailing newline if present
        if output_text.ends_with('\n') {
            output_text.pop();
        }

        Ok(output_text)
    }

    /// Get the error output from the execution
    pub async fn error(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut error_text = String::new();

        for line in &self.output_lines {
            if line.stream == "stderr" {
                error_text.push_str(&line.text);
                error_text.push('\n');
            }
        }

        // Remove trailing newline if present
        if error_text.ends_with('\n') {
            error_text.pop();
        }

        Ok(error_text)
    }

    /// Check if the execution contains an error
    pub fn has_error(&self) -> bool {
        self.has_error
    }

    /// Get the status of the execution
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Get the language used for the execution
    pub fn language(&self) -> &str {
        &self.language
    }
}
