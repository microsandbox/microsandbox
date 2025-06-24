use std::sync::Arc;
use tokio::sync::RwLock;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use clap::Parser;
use microsandbox_cli::{MicrosandboxCliResult, MsbserverArgs};
use microsandbox_server::{port::PortManager, route, state::AppState, Config};
use microsandbox_utils::CHECKMARK;
use tower_http::cors::{Any, CorsLayer};

//--------------------------------------------------------------------------------------------------
// Functions: Main
//--------------------------------------------------------------------------------------------------

#[tokio::main]
pub async fn main() -> MicrosandboxCliResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = MsbserverArgs::parse();

    if args.dev_mode {
        tracing::info!("Development mode: {}", args.dev_mode);
        println!(
            "{} Running in {} mode",
            &*CHECKMARK,
            console::style("development").yellow()
        );
    }

    // Create configuration from arguments
    let config = Arc::new(Config::new(
        args.key,
        args.host,
        args.port,
        args.namespace_dir.clone(),
        args.dev_mode,
    )?);

    // Get namespace directory from config
    let namespace_dir = config.get_namespace_dir().clone();

    // Initialize the port manager
    let port_manager = PortManager::new(namespace_dir).await.map_err(|e| {
        eprintln!("Error initializing port manager: {}", e);
        e
    })?;

    let port_manager = Arc::new(RwLock::new(port_manager));

    // Create application state
    let state = AppState::new(config.clone(), port_manager);

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_origin(Any);

    // Build application
    let app = route::create_router(state).layer(cors);

    // Start server
    tracing::info!("Starting server on {}", config.get_addr());
    println!(
        "{} Server listening on {}",
        &*CHECKMARK,
        console::style(config.get_addr()).yellow()
    );

    let listener = tokio::net::TcpListener::bind(config.get_addr()).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
