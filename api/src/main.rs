//! CLI entry point. Thin by design: parse the command and start the server.

use std::net::SocketAddr;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use api::{adapter::crypto::hash_sync, xmodule};

#[derive(Debug, Parser)]
#[command(
    name = "rust-react-boilerplate-api",
    about = "Axum API for the Rust + React boilerplate"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the HTTP server.
    Serve,
    /// Apply the embedded migrations.
    Migrate,
    /// Seed permissions, roles, settings and demo users.
    Seed,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    init_tracing();

    let command = Cli::parse().command.unwrap_or(Command::Serve);

    match command {
        Command::Serve => serve().await,
        Command::Migrate => {
            let dependencies = xmodule::build().await?;
            xmodule::migrate(&dependencies.postgres).await?;
            tracing::info!("migrations applied");
            Ok(())
        }
        Command::Seed => {
            let dependencies = xmodule::build().await?;
            xmodule::migrate(&dependencies.postgres).await?;
            let password_hash = hash_sync("password").context("hash seed password")?;
            let summary = dependencies.postgres.seed(&password_hash).await?;
            tracing::info!(
                permissions = summary.permissions,
                roles = summary.roles,
                users_created = summary.users_created,
                "seed complete"
            );
            Ok(())
        }
    }
}

async fn serve() -> Result<()> {
    let dependencies = xmodule::build().await?;

    if dependencies.config.run_migrations {
        xmodule::migrate(&dependencies.postgres).await?;
    }

    let app = api::router(&dependencies.state);
    let address = SocketAddr::from(([0, 0, 0, 0], dependencies.config.port));
    let listener = TcpListener::bind(address)
        .await
        .with_context(|| format!("could not bind {address}"))?;

    tracing::info!(
        port = dependencies.config.port,
        app_url = %dependencies.config.app_url,
        "API listening"
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("server error")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("shutdown signal received");
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,api=debug,rust_react_boilerplate_api=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
