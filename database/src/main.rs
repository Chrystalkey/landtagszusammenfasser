use std::net::SocketAddr;

use clap::Parser;
use deadpool_diesel::postgres::{Manager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use lettre::{transport::smtp::authentication::Credentials, SmtpTransport};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::router::app_router;
mod error;
mod external;
mod handlers;
mod infra;
mod router;

pub use diesel;
use infra::db::schema;

// Define embedded database migrations
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

#[derive(Parser, Clone)]
#[command(author, version, about)]
pub struct Configuration {
    #[arg(long, env = "MAIL_SERVER")]
    mail_server: String,
    #[arg(long, env = "MAIL_USER")]
    mail_user: String,
    #[arg(long, env = "MAIL_PASSWORD")]
    mail_password: String,
    #[arg(long, env = "MAIL_SENDER")]
    mail_sender: String,
    #[arg(long, env = "MAIL_RECIPIENT")]
    mail_recipient: String,
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "SERVER_PORT", default_value = "8080")]
    port: u16,
    #[arg(long, short, env = "DATABASE_URL")]
    db_url: String,
}

// Struct to hold the application state
#[derive(Clone)]
pub struct AppState {
    pool: Pool,
    config: Configuration,
    mailer: SmtpTransport,
}

// Main function, the entry point of the application
#[tokio::main]
async fn main() {
    // Initialize tracing for logging
    init_tracing();

    // Load configuration settings
    let config = Configuration::parse();

    // Create a connection pool to the PostgreSQL database
    let manager = Manager::new(config.db_url.as_str(), deadpool_diesel::Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    // Apply pending database migrations
    run_migrations(&pool).await;

    // Create an instance of the application state
    let state = AppState {
        pool,
        mailer: lettre::SmtpTransport::relay(&config.mail_server.as_str())
            .unwrap()
            .credentials(Credentials::new(
                config.mail_user.clone(),
                config.mail_password.clone(),
            ))
            .build(),
        config,
    };
    let arc_state = std::sync::Arc::new(state);

    // Configure the application router
    let app = app_router(arc_state.clone()).with_state(arc_state.clone());
    let address = format!("{}:{}", arc_state.config.host, arc_state.config.port);

    // Parse the socket address
    let socket_addr: SocketAddr = address.parse().expect(format!("Could not Parse Address: {}", address).as_str());
    // Log the server's listening address
    tracing::info!("listening on http://{}", socket_addr);

    // Start the Axum server
    let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|error| return error)
        .unwrap();
}

// Function to initialize tracing for logging
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "RUST_LOG=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Function to run database migrations
async fn run_migrations(pool: &Pool) {
    let conn: deadpool_diesel::postgres::Connection = pool.get().await.unwrap();
    conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
        .await
        .unwrap()
        .unwrap();
}
