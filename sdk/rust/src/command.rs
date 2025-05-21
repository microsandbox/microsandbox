//! Command execution interface for sandboxes

use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

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
    /// Standard output from the command
    stdout: String,
    /// Standard error from the command
    stderr: String,
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
            .unwrap_or(0) as i32;

        let stdout = output_data
            .get("stdout")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let stderr = output_data
            .get("stderr")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Self {
            command,
            args,
            exit_code,
            stdout,
            stderr,
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
        Ok(self.stdout.clone())
    }

    /// Get the standard error from the command
    pub async fn error(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.stderr.clone())
    }

    /// Check if the command was successful (exit code 0)
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Command interface for executing shell commands in a sandbox
pub struct Command<'a> {
    sandbox: &'a SandboxBase,
}

impl<'a> Command<'a> {
    /// Create a new command instance
    pub(crate) fn new(sandbox: &'a SandboxBase) -> Self {
        Self { sandbox }
    }

    /// Execute a shell command in the sandbox
    pub async fn run(
        &self,
        command: &str,
        args: Option<Vec<&str>>,
        timeout: Option<i32>,
    ) -> Result<CommandExecution, Box<dyn Error + Send + Sync>> {
        if !self.sandbox.is_started {
            return Err(Box::new(SandboxError::NotStarted));
        }

        // Convert args to strings
        let args_vec = args
            .unwrap_or_default()
            .iter()
            .map(|&s| s.to_string())
            .collect::<Vec<_>>();

        // Build parameters
        let mut params = serde_json::json!({
            "sandbox": self.sandbox.name,
            "namespace": self.sandbox.namespace,
            "command": command,
            "args": args_vec,
        });

        // Add timeout if specified
        if let Some(t) = timeout {
            params["timeout"] = serde_json::json!(t);
        }

        // Execute command
        let result: HashMap<String, Value> = self
            .sandbox
            .make_request("sandbox.command.execute", params)
            .await?;

        Ok(CommandExecution::new(result))
    }
}
