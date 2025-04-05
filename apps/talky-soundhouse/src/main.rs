mod config;
mod error;
mod handler;
mod message;
mod server;
mod state;

use crate::config::Config;
use crate::error::AppResult;
use crate::state::AppState;
use server::build_routes;
use talky_auth::JwtService;

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt().init();

    tracing::info!("Starting Soundhouse Server...");

    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully.");
    tracing::debug!("Server Address: {}", config.server_addr);

    let app_state = AppState::new(&config.database_url).await;
    tracing::info!("Application state initialized.");

    let routes = build_routes(app_state.clone());

    tracing::info!("Server routes configured.");

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server_addr = config.server_addr;

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(server_addr, async {
        rx.await.ok();
    });

    tracing::info!("Server running at ws://{}", server_addr);

    tokio::task::spawn(server);

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("Received Ctrl+C, initiating shutdown...");
        }
        Err(err) => {
            tracing::error!("Failed to listen for shutdown signal: {}", err);
        }
    }

    let _ = tx.send(());
    tracing::info!("Shutdown signal sent. Server exiting.");

    Ok(())
}
