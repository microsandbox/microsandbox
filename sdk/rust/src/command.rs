//! Command execution interface for sandboxes

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::SandboxBase;
use crate::SandboxError;

/// Result of a command execution in a sandbox
#[derive(Debug, Clone)]
pub struct CommandExecution {
    /// The command that was executed
    command: String,

    /// Arguments passed to the command
    args: Vec<String>,

    /// Exit code from the command
    exit_code: i32,

    /// Whether the command was successful
    success: bool,

    /// Output lines from the execution
    output_lines: Vec<OutputLine>,
}

/// A single line of output from a command execution
#[derive(Debug, Clone)]
struct OutputLine {
    /// Stream type (stdout or stderr)
    stream: String,
    /// Text content
    text: String,
}

impl CommandExecution {
    /// Create a new command execution result
    fn new(output_data: HashMap<String, Value>) -> Self {
        let command = output_data
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let args = if let Some(args_val) = output_data.get("args") {
            if let Some(args_arr) = args_val.as_array() {
                args_arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let exit_code = output_data
            .get("exit_code")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1) as i32;

        let success = output_data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Process output lines
        let mut output_lines = Vec::new();
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

                        output_lines.push(OutputLine { stream, text });
                    }
                }
            }
        }

        Self {
            command,
            args,
            exit_code,
            success,
            output_lines,
        }
    }

    /// Get the command that was executed
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Get the arguments passed to the command
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Get the exit code from the command
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Get the standard output from the command
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

    /// Get the standard error from the command
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

    /// Check if the command was successful (exit code 0)
    pub fn is_success(&self) -> bool {
        self.success
    }
}

/// Command interface for executing shell commands in a sandbox
pub struct Command {
    sandbox: Arc<Mutex<SandboxBase>>,
}

impl Command {
    /// Create a new command instance
    pub(crate) fn new(sandbox: Arc<Mutex<SandboxBase>>) -> Self {
        Self { sandbox }
    }

    /// Execute a shell command in the sandbox
    pub async fn run(
        &self,
        command: &str,
        args: Option<Vec<&str>>,
        timeout: Option<i32>,
    ) -> Result<CommandExecution, Box<dyn Error + Send + Sync>> {
        let is_started = {
            let base = self.sandbox.lock().await;
            base.is_started
        };

        if !is_started {
            return Err(Box::new(SandboxError::NotStarted));
        }

        // Convert args to strings
        let args_vec = args
            .unwrap_or_default()
            .iter()
            .map(|&s| s.to_string())
            .collect::<Vec<_>>();

        // Get the sandbox name and namespace
        let (name, namespace) = {
            let base = self.sandbox.lock().await;
            (base.name.clone(), base.namespace.clone())
        };

        // Build parameters
        let mut params = serde_json::json!({
            "sandbox": name,
            "namespace": namespace,
            "command": command,
            "args": args_vec,
        });

        // Add timeout if specified
        if let Some(t) = timeout {
            params["timeout"] = serde_json::json!(t);
        }

        // Execute command
        let base = self.sandbox.lock().await;
        let result: HashMap<String, Value> =
            base.make_request("sandbox.command.run", params).await?;

        Ok(CommandExecution::new(result))
    }
}
