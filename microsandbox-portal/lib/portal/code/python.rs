//! Python engine implementation for code execution in a sandboxed environment.
//!
//! This module provides a Python-based code execution engine that:
//! - Runs Python code in an interactive subprocess
//! - Captures and streams stdout/stderr output
//! - Manages process lifecycle and cleanup
//! - Provides non-blocking evaluation of Python code
//!
//! The engine uses Python's interactive mode with customized settings to
//! disable prompts and ensure unbuffered output for real-time streaming.

use crossbeam_channel::{bounded, Sender};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::types::{Engine, EngineError, Resp, Stream};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Python engine implementation using subprocess
pub struct PythonEngine {
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<std::process::ChildStdin>>>,
    stdout_thread: Option<thread::JoinHandle<()>>,
    stderr_thread: Option<thread::JoinHandle<()>>,
    shutdown_signal: Option<Sender<()>>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl PythonEngine {
    fn new() -> Self {
        PythonEngine {
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout_thread: None,
            stderr_thread: None,
            shutdown_signal: None,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl Engine for PythonEngine {
    fn initialize(&mut self) -> Result<(), EngineError> {
        // Start Python process with interactive mode
        // -q: hide banner, -u: unbuffered, -i: interactive, clear prompts
        let mut process = Command::new("python")
            .args(&["-q", "-u", "-i", "-c", "import sys; sys.ps1=sys.ps2=''"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                EngineError::Initialization(format!("Failed to start Python process: {}", e))
            })?;

        // Get stdin handle
        let stdin = process.stdin.take().ok_or_else(|| {
            EngineError::Initialization("Failed to open Python stdin".to_string())
        })?;

        // Get stdout and stderr handles
        let stdout = process.stdout.take().ok_or_else(|| {
            EngineError::Initialization("Failed to open Python stdout".to_string())
        })?;

        let stderr = process.stderr.take().ok_or_else(|| {
            EngineError::Initialization("Failed to open Python stderr".to_string())
        })?;

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = bounded::<()>(1);
        self.shutdown_signal = Some(shutdown_tx);

        // Store process and stdin
        *self.process.lock().unwrap() = Some(process);
        *self.stdin.lock().unwrap() = Some(stdin);

        // Create a channel for active evaluation
        let (_eval_tx, eval_rx) = bounded::<(String, Sender<Resp>)>(1);

        // Start stdout handler thread
        let stdout_reader = BufReader::new(stdout);
        let shutdown_rx_stdout = shutdown_rx.clone();
        let eval_rx_stdout = eval_rx.clone();

        self.stdout_thread = Some(thread::spawn(move || {
            let mut lines = stdout_reader.lines();

            loop {
                // Check if shutdown was requested
                if shutdown_rx_stdout.try_recv().is_ok() {
                    break;
                }

                // Get the current evaluation ID and sender
                let current_eval: Option<(String, Sender<Resp>)> = match eval_rx_stdout.try_recv() {
                    Ok((id, sender)) => Some((id, sender)),
                    Err(_) => None,
                };

                // Process stdout if there's an active evaluation
                if let Some((id, sender)) = &current_eval {
                    if let Some(Ok(line)) = lines.next() {
                        // Send the line through the response channel
                        let _ = sender.send(Resp::Line {
                            id: id.clone(),
                            stream: Stream::Stdout,
                            text: line,
                        });
                    } else {
                        // EOF or error
                        break;
                    }
                } else {
                    // No active evaluation, just wait
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }));

        // Start stderr handler thread
        let stderr_reader = BufReader::new(stderr);
        let shutdown_rx_stderr = shutdown_rx;
        let eval_rx_stderr = eval_rx;

        self.stderr_thread = Some(thread::spawn(move || {
            let mut lines = stderr_reader.lines();

            loop {
                // Check if shutdown was requested
                if shutdown_rx_stderr.try_recv().is_ok() {
                    break;
                }

                // Get the current evaluation ID and sender
                let current_eval: Option<(String, Sender<Resp>)> = match eval_rx_stderr.try_recv() {
                    Ok((id, sender)) => Some((id, sender)),
                    Err(_) => None,
                };

                // Process stderr if there's an active evaluation
                if let Some((id, sender)) = &current_eval {
                    if let Some(Ok(line)) = lines.next() {
                        // Send the line through the response channel
                        let _ = sender.send(Resp::Line {
                            id: id.clone(),
                            stream: Stream::Stderr,
                            text: line,
                        });
                    } else {
                        // EOF or error
                        break;
                    }
                } else {
                    // No active evaluation, just wait
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }));

        Ok(())
    }

    fn eval(&mut self, id: String, code: String, sender: &Sender<Resp>) -> Result<(), EngineError> {
        // Get stdin handle
        let mut stdin_guard = self.stdin.lock().unwrap();
        let stdin = stdin_guard
            .as_mut()
            .ok_or_else(|| EngineError::Unavailable("Python process not available".to_string()))?;

        // Write code to Python process
        writeln!(stdin, "{}", code).map_err(|e| {
            EngineError::Evaluation(format!("Failed to send code to Python: {}", e))
        })?;

        // Flush to ensure code is processed
        stdin.flush().map_err(|e| {
            EngineError::Evaluation(format!("Failed to flush code to Python: {}", e))
        })?;

        // Allow some time for execution and output capturing
        thread::sleep(Duration::from_millis(100));

        // Mark evaluation as complete
        let _ = sender.send(Resp::Done { id });

        Ok(())
    }

    fn shutdown(&mut self) {
        // Signal shutdown to IO threads
        if let Some(tx) = self.shutdown_signal.take() {
            let _ = tx.send(());
        }

        // Terminate Python process
        if let Ok(mut guard) = self.process.lock() {
            if let Some(mut process) = guard.take() {
                let _ = process.kill();
                let _ = process.wait();
            }
        }

        // Wait for threads to complete
        if let Some(handle) = self.stdout_thread.take() {
            let _ = handle.join();
        }

        if let Some(handle) = self.stderr_thread.take() {
            let _ = handle.join();
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Create a new Python engine instance
pub fn create_engine() -> Result<Box<dyn Engine>, EngineError> {
    Ok(Box::new(PythonEngine::new()))
}
