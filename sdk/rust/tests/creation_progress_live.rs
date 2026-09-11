//! Opt-in SDK qualification against an installed full checkpoint containing /work/hash.
//! Set MSB_PROGRESS_SNAPSHOT plus MSB_HOME/MSB_PATH/MSB_AGENTD_PATH/MSB_LIBKRUNFW_PATH.

#![cfg(feature = "local")]

use microsandbox::{CreationProgress, Sandbox, StartupPhase};

#[tokio::test]
#[ignore = "requires a matching runtime and prepared full snapshot fixture"]
async fn restored_creation_progress_and_ignored_observer() {
    let snapshot = std::env::var("MSB_PROGRESS_SNAPSHOT").expect("snapshot fixture");
    for observed in [true, false] {
        let name = format!("progress-live-{}-{observed}", std::process::id());
        let started = std::time::Instant::now();
        let (mut progress, task) = Sandbox::builder(&name)
            .from_snapshot(&snapshot)
            .forked()
            .create_with_progress()
            .unwrap();
        if observed {
            let mut activating = false;
            while let Some(event) = progress.recv().await {
                println!(
                    "{} {}",
                    started.elapsed().as_millis(),
                    serde_json::to_string(&event).unwrap()
                );
                if let CreationProgress::Startup(event) = event {
                    activating |= event.phase == StartupPhase::Activating;
                    assert!(
                        event
                            .total_bytes
                            .is_none_or(|total| event.completed_bytes <= total)
                    );
                }
            }
            assert!(activating, "EOF cannot replace the activation signal");
        } else {
            drop(progress);
        }
        let sandbox = task.await.unwrap().unwrap();
        println!(
            "created observed={observed} elapsed_ms={}",
            started.elapsed().as_millis()
        );
        let result = sandbox.exec("sha256sum", ["-c", "/work/hash"]).await;
        // Always stop our child before asserting workload results.
        sandbox.kill().await.unwrap();
        let result = result.unwrap();
        assert!(result.status().success);
    }
}
