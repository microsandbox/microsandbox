//! Wall-clock startup stages; sandbox identities are trace fields, not metrics labels.

use std::future::Future;
use std::time::Instant;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

struct StageTiming<'a> {
    sandbox_name: &'a str,
    stage: &'static str,
    started: Instant,
    outcome: &'static str,
}

//--------------------------------------------------------------------------------------------------
// Trait Implementations
//--------------------------------------------------------------------------------------------------

impl Drop for StageTiming<'_> {
    fn drop(&mut self) {
        tracing::debug!(
            sandbox_name = self.sandbox_name,
            stage = self.stage,
            elapsed_seconds = self.started.elapsed().as_secs_f64(),
            outcome = self.outcome,
            "sandbox startup stage finished"
        );
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Measure a fallible startup stage, including errors and future cancellation.
pub(crate) async fn measure<T, E>(
    sandbox_name: &str,
    stage: &'static str,
    future: impl Future<Output = Result<T, E>>,
) -> Result<T, E> {
    let mut timing = StageTiming {
        sandbox_name,
        stage,
        started: Instant::now(),
        outcome: "cancelled",
    };
    let result = future.await;
    timing.outcome = if result.is_ok() { "success" } else { "error" };
    result
}
