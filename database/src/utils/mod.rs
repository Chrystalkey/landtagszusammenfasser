use deadpool_diesel::postgres::Pool;
use diesel_migrations::MigrationHarness;
use lettre::{Message, Transport};
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use diesel_migrations::{embed_migrations, EmbeddedMigrations};

use crate::{error::LTZFError, LTZFServer, Result};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn send_email(subject: String, body: String, state: &LTZFServer) -> Result<()> {
    if state.mailer.is_none() {
        return Ok(());
    }
    let email = Message::builder()
        .from(
            format!("Landtagszusammenfasser <{}>", state.config.mail_sender.as_ref().unwrap())
                .parse()
                .unwrap(),
        )
        .to(state.config.mail_recipient.as_ref().unwrap().parse().unwrap())
        .subject(subject.clone())
        .body(body.clone())
        .unwrap();
    tracing::info!("Mail was Sent. Subject: {}", subject);
    tracing::debug!("Mail Contents:\n{}", body);
    state.mailer.as_ref().unwrap().send(&email)?;
    Ok(())
}

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

// Function to initialize tracing for logging
pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "RUST_LOG=debug".into()),
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
        .map_err(|e| LTZFError::FallbackError(e))?;
    Ok(())
}