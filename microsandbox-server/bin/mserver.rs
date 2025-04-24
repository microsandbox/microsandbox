use std::{path::PathBuf, sync::Arc};

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use clap::Parser;
use microsandbox_server::{
    config::{Config, DEFAULT_PORT},
    route,
    state::AppState,
};
use microsandbox_utils::CHECKMARK;
use tower_http::cors::{Any, CorsLayer};

//--------------------------------------------------------------------------------------------------
// Types: Args
//--------------------------------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(author, version, about = "Microsandbox Server")]
struct Args {
    /// Secret key used for JWT token generation and validation
    #[arg(short = 'k', long = "key")]
    key: Option<String>,

    /// Port number to listen on
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Directory for storing namespaces
    #[arg(short = 'p', long = "path")]
    namespace_path: Option<PathBuf>,

    /// Run in development mode
    #[arg(long, default_value_t = false)]
    dev: bool,
}

//--------------------------------------------------------------------------------------------------
// Functions: Main
//--------------------------------------------------------------------------------------------------

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args = Args::parse();

    if args.dev {
        tracing::info!("Development mode: {}", args.dev);
        #[cfg(feature = "cli")]
        println!(
            "{} Running in {} mode",
            &*CHECKMARK,
            console::style("development").yellow()
        );
    }

    // Create configuration from arguments
    let config = Arc::new(Config::new(
        args.key,
        args.port,
        args.namespace_path,
        args.dev,
    )?);

    // Create application state
    let state = AppState::new(config.clone());

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_origin(Any);

    // Build application
    let app = route::create_router(state).layer(cors);

    // Start server
    tracing::info!("Starting server on {}", config.get_addr());
    #[cfg(feature = "cli")]
    println!(
        "{} Server listening on {}",
        &*CHECKMARK,
        console::style(config.get_addr()).yellow()
    );

    let listener = tokio::net::TcpListener::bind(config.get_addr()).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
