use std::path::PathBuf;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::error::to_napi_error;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Builder for installing the runtime binaries.
#[napi(js_name = "Setup")]
pub struct JsSetup {
    base_dir: Option<PathBuf>,
    version: Option<String>,
    skip_verify: bool,
    force: bool,
}

//--------------------------------------------------------------------------------------------------
// Methods
//--------------------------------------------------------------------------------------------------

#[napi]
impl JsSetup {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            base_dir: None,
            version: None,
            skip_verify: false,
            force: false,
        }
    }

    #[napi(js_name = "baseDir")]
    pub fn base_dir(&mut self, path: String) -> &Self {
        self.base_dir = Some(PathBuf::from(path));
        self
    }

    #[napi]
    pub fn version(&mut self, version: String) -> &Self {
        self.version = Some(version);
        self
    }

    #[napi(js_name = "skipVerify")]
    pub fn skip_verify(&mut self, enabled: bool) -> &Self {
        self.skip_verify = enabled;
        self
    }

    #[napi]
    pub fn force(&mut self, enabled: bool) -> &Self {
        self.force = enabled;
        self
    }

    #[napi]
    pub async fn install(&self) -> Result<()> {
        let config = microsandbox::config::GlobalConfig {
            home: self.base_dir.clone(),
            ..Default::default()
        };
        let mut options = microsandbox::setup::InstallOptions {
            force: self.force,
            verify: !self.skip_verify,
            ..Default::default()
        };
        if let Some(version) = &self.version {
            options.version.clone_from(version);
        }
        microsandbox::setup::install_runtime(&config, options)
            .await
            .map(|_| ())
            .map_err(to_napi_error)
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Check if msb and libkrunfw are installed and available.
#[napi]
pub fn is_installed() -> bool {
    microsandbox::setup::is_runtime_installed(&microsandbox::config::GlobalConfig::default())
}

/// Download and install msb + libkrunfw under non-empty $MSB_HOME, or
/// ~/.microsandbox/ when the override is unset or empty.
#[napi]
pub async fn install() -> Result<()> {
    microsandbox::setup::install_runtime(
        &microsandbox::config::GlobalConfig::default(),
        Default::default(),
    )
    .await
    .map(|_| ())
    .map_err(to_napi_error)
}
