//! Main portal process for microsandbox.
//!
//! This binary starts a JSON-RPC server that can handle generic portal operations.
//! It serves as the main entry point for the microsandbox portal service.

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing;

use microsandbox_portal::{handler::SharedState, route::create_router};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// Default host address
const DEFAULT_HOST: &str = "127.0.0.1";

/// Default port number
const DEFAULT_PORT: u16 = 4444;

//--------------------------------------------------------------------------------------------------
// Types
//--------------------------------------------------------------------------------------------------

/// CLI arguments for Microsandbox Portal
#[derive(Debug, Parser)]
#[command(name = "portal", author, about = "JSON-RPC portal for microsandbox")]
struct PortalArgs {
    /// Port number to listen on
    #[arg(short, long)]
    port: Option<u16>,
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = PortalArgs::parse();

    // Resolve the server address
    let port = args.port.unwrap_or(DEFAULT_PORT);
    let addr = format!("{}:{}", DEFAULT_HOST, port)
        .parse::<SocketAddr>()
        .unwrap();
    let state = SharedState::default();

    tracing::info!("Starting microsandbox portal server on {}", addr);

    // Create the router
    let app = create_router(state);

    // Start the server
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
