//! Startup telemetry on the existing PID pipe, independent of VM work.

use std::fs::File;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::io::AsyncWriteExt;
use tokio::sync::watch;

use crate::startup_progress::{StartupPhase, StartupProgress, StartupProgressCallback};

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

pub(super) fn start(
    file: File,
    runtime: &tokio::runtime::Runtime,
    initial_phase: StartupPhase,
) -> StartupProgressCallback {
    let initial = StartupProgress::phase(initial_phase);
    let (sender, mut receiver) = watch::channel(initial.clone());
    runtime.spawn(async move {
        let mut file = tokio::fs::File::from_std(file);
        loop {
            // A watch retains the newest cumulative state even when readers are slow. In
            // particular Activating is terminal on this channel and cannot be overwritten.
            let progress = receiver.borrow_and_update().clone();
            let Ok(mut bytes) = serde_json::to_vec(&progress) else {
                break;
            };
            bytes.push(b'\n');
            if file.write_all(&bytes).await.is_err() || file.flush().await.is_err() {
                // An older launcher (or an exited observer) may close after the PID reply.
                // Telemetry loss is never a reason to fail a healthy runtime.
                break;
            }
            if progress.phase == StartupPhase::Activating || receiver.changed().await.is_err() {
                break;
            }
        }
    });
    let last = Mutex::new((Instant::now(), initial.phase));
    Arc::new(move |progress| {
        let mut last = last.lock().unwrap_or_else(|error| error.into_inner());
        if progress.phase != last.1
            || last.0.elapsed() >= Duration::from_millis(100)
            || progress.total_bytes == Some(progress.completed_bytes)
        {
            *last = (Instant::now(), progress.phase);
            sender.send_replace(progress);
        }
    })
}
