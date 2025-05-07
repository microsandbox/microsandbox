//! Shared state management for the microsandbox portal server.

use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use crate::portal::{
    command::CommandHandle,
    repl::{EngineHandle, Line},
};

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// SharedState for the server
#[derive(Clone, Debug)]
pub struct SharedState {
    /// Indicates if the server is ready to process requests
    pub ready: Arc<Mutex<bool>>,

    /// Engine handle for REPL environment
    pub engine_handle: Arc<Mutex<Option<EngineHandle>>>,

    /// Command handle for command execution
    pub command_handle: Arc<Mutex<Option<CommandHandle>>>,

    /// Store outputs from REPL executions
    pub outputs: Arc<Mutex<HashMap<String, Vec<Line>>>>,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            ready: Arc::new(Mutex::new(false)),
            engine_handle: Arc::new(Mutex::new(None)),
            command_handle: Arc::new(Mutex::new(None)),
            outputs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
