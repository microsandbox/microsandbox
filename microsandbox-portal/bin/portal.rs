//! Main portal process for microsandbox.
//!
//! This binary starts a JSON-RPC server that can handle generic portal operations.
//! It serves as the main entry point for the microsandbox portal service.

use anyhow::Result;
use clap::Parser;
use microsandbox_utils::DEFAULT_PORTAL_GUEST_PORT;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, signal};
use tracing;

use microsandbox_portal::{
    portal::repl::{start_engines, EngineHandle},
    route::create_router,
    state::SharedState,
};

//--------------------------------------------------------------------------------------------------
// Constants
//--------------------------------------------------------------------------------------------------

/// Default host address
const DEFAULT_HOST: &str = "0.0.0.0";

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

/// Shutdown signal handler
async fn shutdown_signal(engine_handle: Option<EngineHandle>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, cleaning up...");

    // Shutdown the engine if it exists
    if let Some(handle) = engine_handle {
        if let Err(e) = handle.shutdown().await {
            tracing::error!("Error shutting down engines: {}", e);
        } else {
            tracing::info!("Engines shutdown successfully");
        }
    }

    tracing::info!("Server shutdown complete");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = PortalArgs::parse();

    // Resolve the server address
    let port = args.port.unwrap_or(DEFAULT_PORTAL_GUEST_PORT);
    let addr = format!("{}:{}", DEFAULT_HOST, port)
        .parse::<SocketAddr>()
        .unwrap();

    // Initialize the engine handle
    let state = SharedState::default();
    let engine_handle_for_shutdown = Arc::clone(&state.engine_handle);

    // Try to start the REPL engines
    match start_engines().await {
        Ok(engine_handle) => {
            tracing::info!("REPL engines started successfully");
            *engine_handle_for_shutdown.lock().await = Some(engine_handle.clone());
            *state.engine_handle.lock().await = Some(engine_handle);
            *state.ready.lock().await = true;
        }
        Err(e) => {
            tracing::warn!("Failed to start REPL engines: {}", e);
            // Continue without engines, some functionality will be limited
        }
    }

    tracing::info!("Starting microsandbox portal server on {}", addr);

    // Create the router
    let app = create_router(state);

    // Clone for shutdown
    let engine_handle_clone = engine_handle_for_shutdown.lock().await.clone();

    // Start the server with graceful shutdown
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(engine_handle_clone))
        .await?;

    Ok(())
}
