mod api;
mod config;
mod core;
mod infra;
mod monitoring;
mod rollup;
mod utils;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::load()?;

    // Initialize infrastructure controller
    let infra_controller = infra::Controller::new(&config).await?;

    // Initialize rollup manager
    let rollup_manager = rollup::Manager::new(&config, infra_controller.clone()).await?;

    // Initialize monitoring system
    let monitoring = monitoring::System::new(&config).await?;

    // Build our application with a route
    let app = Router::new()
        .route("/health", get(api::health_check))
        .route("/api/v1/rollups", post(api::create_rollup))
        .route("/api/v1/rollups/:id", get(api::get_rollup))
        .route("/metrics", get(api::metrics))
        .layer(TraceLayer::new_for_http());

    // Run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received, starting graceful shutdown");
} 