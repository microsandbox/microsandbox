//! Node.js engine implementation for code execution in a sandboxed environment.
//!
//! This module provides a Node.js-based code execution engine that:
//! - Runs Node.js code in a subprocess with a custom REPL configuration
//! - Captures and streams stdout/stderr output
//! - Manages process lifecycle and cleanup
//! - Provides non-blocking evaluation of JavaScript code
//!
//! The engine uses a custom REPL configuration that disables terminal features
//! and prompts for cleaner output handling.

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    time::{sleep, Duration},
};

use super::types::{Engine, EngineError, Resp, Stream};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Node.js engine implementation using subprocess
pub struct NodeEngine {
    process_control_tx: Option<Sender<ProcessControl>>,
    eval_tx: Option<Sender<EvalRequest>>,
}

/// Commands for controlling the Node.js process
enum ProcessControl {
    Shutdown,
}

/// Request for code evaluation
struct EvalRequest {
    id: String,
    code: String,
    resp_tx: Sender<Resp>,
    done_tx: oneshot::Sender<Result<(), EngineError>>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl NodeEngine {
    fn new() -> Self {
        NodeEngine {
            process_control_tx: None,
            eval_tx: None,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

#[async_trait]
impl Engine for NodeEngine {
    async fn initialize(&mut self) -> Result<(), EngineError> {
        // Create channels for process control and evaluation requests
        let (process_control_tx, mut process_control_rx) = mpsc::channel::<ProcessControl>(10);
        let (eval_tx, mut eval_rx) = mpsc::channel::<EvalRequest>(10);

        self.process_control_tx = Some(process_control_tx);
        self.eval_tx = Some(eval_tx);

        // Start the Node.js process manager in a separate task
        tokio::spawn(async move {
            // Start Node.js process with custom REPL
            // Custom REPL starts with no prompt, no terminal features, and ignores undefined
            let mut process = match Command::new("node")
                .args(&[
                    "-e",
                    "require('repl').start({prompt:'', terminal:false, ignoreUndefined:true})",
                ])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to start Node.js process: {}", e);
                    return;
                }
            };

            // Get stdin handle
            let mut stdin = match process.stdin.take() {
                Some(s) => s,
                None => {
                    eprintln!("Failed to open Node.js stdin");
                    return;
                }
            };

            // Get stdout and stderr handles
            let stdout = match process.stdout.take() {
                Some(s) => s,
                None => {
                    eprintln!("Failed to open Node.js stdout");
                    return;
                }
            };

            let stderr = match process.stderr.take() {
                Some(s) => s,
                None => {
                    eprintln!("Failed to open Node.js stderr");
                    return;
                }
            };

            // Start stdout handler in a separate task
            let stdout_reader = BufReader::new(stdout);
            let (stdout_done_tx, mut stdout_done_rx) = mpsc::channel::<()>(1);
            let stdout_current_eval = Arc::new(Mutex::new(None::<(String, Sender<Resp>)>));
            let stdout_eval_clone = Arc::clone(&stdout_current_eval);

            tokio::task::spawn_blocking(move || {
                let mut lines_future = stdout_reader.lines();
                let runtime = tokio::runtime::Handle::current();

                loop {
                    // Use the runtime to execute the async call in the blocking thread
                    let line_result = runtime.block_on(lines_future.next_line());

                    match line_result {
                        Ok(Some(line)) => {
                            // Skip Node.js REPL response tags '>' and '..'
                            if !line.trim().is_empty()
                                && !line.starts_with('>')
                                && !line.starts_with("..")
                            {
                                // Send to active evaluation if one exists
                                if let Some((id, sender)) =
                                    stdout_eval_clone.lock().unwrap().as_ref()
                                {
                                    // Use block_on to send the message
                                    let _ = runtime.block_on(sender.send(Resp::Line {
                                        id: id.clone(),
                                        stream: Stream::Stdout,
                                        text: line,
                                    }));
                                }
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(_) => break,   // Error reading
                    }
                }

                // Signal that we're done
                let _ = runtime.block_on(stdout_done_tx.send(()));
            });

            // Start stderr handler in a separate task
            let stderr_reader = BufReader::new(stderr);
            let (stderr_done_tx, mut stderr_done_rx) = mpsc::channel::<()>(1);
            let stderr_current_eval = Arc::clone(&stdout_current_eval);

            tokio::task::spawn_blocking(move || {
                let mut lines_future = stderr_reader.lines();
                let runtime = tokio::runtime::Handle::current();

                loop {
                    // Use the runtime to execute the async call in the blocking thread
                    let line_result = runtime.block_on(lines_future.next_line());

                    match line_result {
                        Ok(Some(line)) => {
                            // Send to active evaluation if one exists
                            if let Some((id, sender)) = stderr_current_eval.lock().unwrap().as_ref()
                            {
                                // Use block_on to send the message
                                let _ = runtime.block_on(sender.send(Resp::Line {
                                    id: id.clone(),
                                    stream: Stream::Stderr,
                                    text: line,
                                }));
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(_) => break,   // Error reading
                    }
                }

                // Signal that we're done
                let _ = runtime.block_on(stderr_done_tx.send(()));
            });

            // Process control and evaluation loop
            loop {
                tokio::select! {
                    Some(ctrl) = process_control_rx.recv() => {
                        match ctrl {
                            ProcessControl::Shutdown => {
                                break;
                            }
                        }
                    }
                    Some(eval_req) = eval_rx.recv() => {
                        let EvalRequest { id, code, resp_tx, done_tx } = eval_req;

                        // Set as current evaluation
                        {
                            let mut eval_guard = stdout_current_eval.lock().unwrap();
                            *eval_guard = Some((id.clone(), resp_tx.clone()));
                        }

                        // Execute the code
                        let result = async {
                            // Write code to Node.js process
                            stdin.write_all(code.as_bytes()).await.map_err(|e| {
                                EngineError::Evaluation(format!("Failed to send code to Node.js: {}", e))
                            })?;

                            // Add newline to execute the code
                            stdin.write_all(b"\n").await.map_err(|e| {
                                EngineError::Evaluation(format!("Failed to send newline to Node.js: {}", e))
                            })?;

                            // Flush to ensure code is processed
                            stdin.flush().await.map_err(|e| {
                                EngineError::Evaluation(format!("Failed to flush code to Node.js: {}", e))
                            })?;

                            // Give time for execution
                            sleep(Duration::from_millis(500)).await;

                            // Signal completion
                            let _ = resp_tx.send(Resp::Done { id }).await;
                            Ok(())
                        }.await;

                        // Clear current evaluation
                        {
                            let mut eval_guard = stdout_current_eval.lock().unwrap();
                            *eval_guard = None;
                        }

                        // Signal completion to caller
                        let _ = done_tx.send(result);
                    }
                    _ = stdout_done_rx.recv() => {
                        eprintln!("Node.js stdout handler exited");
                        break;
                    }
                    _ = stderr_done_rx.recv() => {
                        eprintln!("Node.js stderr handler exited");
                        break;
                    }
                }
            }

            // Cleanup: kill the process
            let _ = process.kill().await;
            let _ = process.wait().await;
        });

        // Wait a bit for initialization
        sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    async fn eval(
        &mut self,
        id: String,
        code: String,
        sender: &Sender<Resp>,
    ) -> Result<(), EngineError> {
        let eval_tx = self.eval_tx.as_ref().ok_or_else(|| {
            EngineError::Unavailable("Node.js engine not initialized".to_string())
        })?;

        // Create a oneshot channel for the result
        let (done_tx, done_rx) = oneshot::channel();

        // Send evaluation request
        eval_tx
            .send(EvalRequest {
                id,
                code,
                resp_tx: sender.clone(),
                done_tx,
            })
            .await
            .map_err(|_| EngineError::Unavailable("Node.js process channel closed".to_string()))?;

        // Wait for completion
        done_rx
            .await
            .map_err(|_| EngineError::Unavailable("Node.js evaluation cancelled".to_string()))?
    }

    async fn shutdown(&mut self) {
        if let Some(tx) = self.process_control_tx.take() {
            // Send shutdown command
            let _ = tx.send(ProcessControl::Shutdown).await;
        }

        // Clear channels
        self.eval_tx = None;

        // Wait a bit for clean shutdown
        sleep(Duration::from_millis(100)).await;
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Create a new Node.js engine instance
pub fn create_engine() -> Result<Box<dyn Engine>, EngineError> {
    Ok(Box::new(NodeEngine::new()))
}
