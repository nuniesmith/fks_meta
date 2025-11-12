//! FKS Meta Service - MetaTrader 5 Execution Plugin
//!
//! Standalone service that provides MT5 integration via HTTP API
//! Can be used directly or as a plugin for fks_execution

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post, delete},
    Json, Router,
};
use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

use fks_meta::{AppState, Settings, MT5Client};

#[derive(Parser, Debug)]
#[command(version, about = "FKS Meta - MetaTrader 5 Plugin Service")]
struct Cli {
    #[arg(long, default_value = "0.0.0.0:8005")]
    listen: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let settings = Arc::new(Settings::from_env()?);
    
    info!(
        service = "fks_meta",
        version = env!("CARGO_PKG_VERSION"),
        "Starting FKS Meta service"
    );

    // Initialize MT5 client
    let mt5_client = Arc::new(MT5Client::new(settings.clone()).await?);
    
    let app_state = AppState {
        mt5_client,
        settings,
    };

    // Build router
    let app = Router::new()
        .route("/health", get(fks_meta::api::health::health_check))
        .route("/metrics", get(fks_meta::api::health::metrics))
        .route("/status", get(fks_meta::api::health::mt5_status))
        .route("/orders", post(fks_meta::api::orders::create_order))
        .route("/orders/:order_id", get(fks_meta::api::orders::get_order))
        .route("/orders/:order_id", delete(fks_meta::api::orders::cancel_order))
        .route("/positions", get(fks_meta::api::positions::list_positions))
        .route("/positions/:symbol", get(fks_meta::api::positions::get_position))
        .route("/positions/:symbol", delete(fks_meta::api::positions::close_position))
        .route("/market/:symbol", get(fks_meta::api::market::get_market_data))
        .with_state(app_state);

    // Parse address
    let addr: SocketAddr = cli.listen.parse()?;
    
    info!(
        service = "fks_meta",
        address = %addr,
        "Listening on"
    );

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, shutting down gracefully");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down gracefully");
        },
    }
}

