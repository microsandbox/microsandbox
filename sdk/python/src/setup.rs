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
