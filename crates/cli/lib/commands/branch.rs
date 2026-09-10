//! Direct local execution branching without a durable full snapshot.

use clap::Args;
use microsandbox::Sandbox;

use crate::ui;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// Create an independent child from a running or user-paused local sandbox.
#[derive(Args)]
pub struct BranchArgs {
    /// Source sandbox name.
    pub source: String,
    /// Name of the new child sandbox.
    #[arg(long)]
    pub name: String,
    /// Suppress progress output.
    #[arg(short, long)]
    pub quiet: bool,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

/// Branch source execution. The child's CoW memory is inherent to this operation.
pub async fn run(args: BranchArgs) -> anyhow::Result<()> {
    let source = Sandbox::get(&args.source).await?;
    let child = source.branch(&args.name).await?;
    if !args.quiet {
        ui::success("Branched", child.name());
    }
    Ok(())
}
