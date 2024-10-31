use std::{env, path::PathBuf};

use monocore::{
    config::{EnvPair, Group, Service},
    runtime::Supervisor,
    vm::MicroVm,
    MonocoreError, MonocoreResult,
};
use tokio::signal::unix::{signal, SignalKind};
use tracing::info;

//--------------------------------------------------------------------------------------------------
// Function: main
//--------------------------------------------------------------------------------------------------

/// Entry point for the runtime supervisor process.
///
/// Handles both supervisor and subprocess modes based on command line arguments.
///
/// # Arguments
///
/// Expected arguments for supervisor mode:
/// ```text
/// monokrun --run-supervisor <service_json> <group_json> <rootfs_path>
/// ```
///
/// Expected arguments for subprocess mode:
/// ```text
/// monokrun --run-microvm-subprocess <service_json> <env_json> <argv_json> <rootfs_path>
/// ```
#[tokio::main]
pub async fn main() -> MonocoreResult<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<_> = env::args().collect();

    // Check for subprocess mode first
    if args.len() >= 6 && args[1] == "--run-microvm-subprocess" {
        // Handle subprocess mode
        let service: Service = serde_json::from_str(&args[2])?;
        let env: Vec<EnvPair> = serde_json::from_str(&args[3])?;
        let argv: Vec<String> = serde_json::from_str(&args[4])?;
        let root_path = PathBuf::from(&args[5]);

        // Set up micro VM options
        let microvm = MicroVm::builder()
            .root_path(root_path)
            .num_vcpus(service.get_cpus())
            .ram_mib(service.get_ram())
            .port_map(service.get_port().cloned().into_iter())
            .workdir_path(service.get_workdir().unwrap_or("/"))
            .exec_path(service.get_command().unwrap_or("/bin/sh"))
            .argv(argv)
            .env(env)
            .build()?;

        microvm.start()?;
        return Ok(());
    }

    // Check for supervisor mode
    if args.len() >= 5 && args[1] == "run-supervisor" {
        let service: Service = serde_json::from_str(&args[2])?;
        let group: Group = serde_json::from_str(&args[3])?;
        let rootfs_path = PathBuf::from(&args[4]);

        // Create and start the supervisor
        let mut supervisor = Supervisor::new(service, group, rootfs_path).await?;

        // Set up signal handler for graceful shutdown
        let mut term_signal = signal(SignalKind::terminate())?;

        tokio::select! {
            _ = supervisor.start() => {
                info!("Supervisor exited normally");
            }
            _ = term_signal.recv() => {
                info!("Received SIGTERM signal, initiating graceful shutdown");
                supervisor.stop().await?;
            }
        }

        return Ok(());
    }

    // If we get here, no valid subcommand was provided
    Err(MonocoreError::InvalidSupervisorArgs(
        "Usage: monocore run-supervisor <service_json> <group_json> <rootfs_path>\n       monocore --run-microvm-subprocess <service_json> <env_json> <argv_json> <rootfs_path>".into(),
    ))
}
