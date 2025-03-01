use deadpool_diesel::postgres::Pool;
use diesel_migrations::MigrationHarness;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};

use crate::{error::*, Result};

pub mod notify;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub async fn shutdown_signal() {
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
}
pub fn as_option<T>(v: Vec<T>) -> Option<Vec<T>> {
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}
// Function to initialize tracing for logging
pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "RUST_LOG=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Function to run database migrations
pub async fn run_migrations(pool: &Pool) -> Result<()> {
    let conn: deadpool_diesel::postgres::Connection = pool.get().await?;
    conn.interact(|conn| 
        conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await?
        .map_err(|e| 
                DatabaseError::Unknown { source: e })?;
    Ok(())
}