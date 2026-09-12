use pyo3::prelude::*;

use crate::error::to_py_err;

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Download and install msb + libkrunfw under non-empty $MSB_HOME, or
/// ~/.microsandbox/ when the override is unset or empty.
#[pyfunction]
pub fn install<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        microsandbox::setup::install_runtime(
            &microsandbox::config::GlobalConfig::default(),
            Default::default(),
        )
        .await
        .map_err(to_py_err)?;
        Ok(())
    })
}

/// Check if msb and libkrunfw are installed and available.
#[pyfunction]
pub fn is_installed() -> bool {
    microsandbox::setup::is_runtime_installed(&microsandbox::config::GlobalConfig::default())
}

/// Register the wheel executable as a fallback after the runtime home.
#[pyfunction]
pub fn set_packaged_msb_path(path: String) {
    microsandbox::config::set_sdk_packaged_msb_path(path);
}

/// Resolve the CLI runtime without depending on the selected local/cloud backend.
#[pyfunction]
pub fn resolved_cli_msb_path() -> PyResult<String> {
    let config = microsandbox::config::load_persisted_config_or_default().map_err(to_py_err)?;
    microsandbox::setup::resolve_runtime(&config)
        .map(|runtime| runtime.msb_path.to_string_lossy().into_owned())
        .map_err(to_py_err)
}
