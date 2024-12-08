#![recursion_limit = "256"]

mod api;
mod db;
mod error;
mod router;
mod utils;

use std::sync::Arc;

use clap::Parser;
use deadpool_diesel::postgres::{Manager, Pool};

use lettre::{transport::smtp::authentication::Credentials, SmtpTransport};
use tokio::net::TcpListener;

pub use api::{LTZFArc, LTZFServer};
pub use error::Result;
use utils::{init_tracing, run_migrations, shutdown_signal};
#[derive(Parser, Clone, Debug)]
#[command(author, version, about)]
struct Configuration {
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
    #[arg(long, short)]
    config: Option<String>,
}

impl Configuration {
    pub async fn build_pool(&self) -> Result<Pool> {
        // Create a connection pool to the PostgreSQL database
        let manager = Manager::new(self.db_url.as_str(), deadpool_diesel::Runtime::Tokio1);
        let pool = Pool::builder(manager).build()?;
        Ok(pool)
    }

    pub async fn build_mailer(&self) -> Result<SmtpTransport> {
        let mailer = SmtpTransport::relay(&self.mail_server.as_str())?
            .credentials(Credentials::new(
                self.mail_user.clone(),
                self.mail_password.clone(),
            ))
            .build();
        Ok(mailer)
    }
    pub fn init() -> Self {
        let config = Configuration::parse();
        config
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let config = Configuration::init();
    tracing::debug!("Configuration: {:?}", &config);


    tracing::info!("Starting the Initialisation process");
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    tracing::debug!("Started Listener");
    let db_pool = config.build_pool().await?;
    tracing::debug!("Started Database Pool");
    let mailer = config.build_mailer().await;
    let mailer= if let Err(e) = mailer {
        tracing::warn!("Failed to create mailer: {}\nMailer will not be available", e);
        None
    }else{
        tracing::debug!("Started Mailer");
        Some(mailer.unwrap())
    };

    // Apply pending database migrations
    run_migrations(&db_pool).await;
    tracing::debug!("Applied Migrations");


    let state = Arc::new(LTZFServer::new(db_pool, mailer, config));
    tracing::debug!("Constructed Server State");

    // Init Axum router
    let app = openapi::server::new(state.clone());
    tracing::debug!("Constructed Router");
    tracing::info!("Starting Server on {}:{}", state.config.host, state.config.port);
    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
