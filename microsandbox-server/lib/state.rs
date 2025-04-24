use std::sync::Arc;

use getset::Getters;

use crate::config::Config;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Getters)]
#[getset(get = "pub with_prefix")]
pub struct AppState {
    /// The application configuration
    config: Arc<Config>,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

impl AppState {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}
