#![forbid(unsafe_code)]
pub(crate) mod api;
pub(crate) mod db;
pub(crate) mod error;
pub(crate) mod utils;

use std::{num::NonZeroU64, sync::Arc};

use clap::Parser;
use sqlx;

use error::LTZFError;
use lettre::{transport::smtp::authentication::Credentials, SmtpTransport};
use sha256::digest;
use tokio::net::TcpListener;

pub use api::{LTZFArc, LTZFServer};
pub use error::Result;
use utils::{init_tracing, shutdown_signal};

pub type DateTime = chrono::DateTime<chrono::Utc>;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Parser, Clone, Debug, Default)]
#[command(author, version, about)]
pub struct Configuration {
    #[arg(long, env = "MAIL_SERVER")]
    pub mail_server: Option<String>,
    #[arg(long, env = "MAIL_USER")]
    pub mail_user: Option<String>,
    #[arg(long, env = "MAIL_PASSWORD")]
    pub mail_password: Option<String>,
    #[arg(long, env = "MAIL_SENDER")]
    pub mail_sender: Option<String>,
    #[arg(long, env = "MAIL_RECIPIENT")]
    pub mail_recipient: Option<String>,
    #[arg(long, env = "LTZF_HOST", default_value = "0.0.0.0")]
    pub host: String,
    #[arg(long, env = "LTZF_PORT", default_value = "80")]
    pub port: u16,
    #[arg(long, short, env = "DATABASE_URL")]
    pub db_url: String,
    #[arg(long, short)]
    pub config: Option<String>,

    #[arg(
        long,
        env = "LTZF_KEYADDER_KEY",
        help = "The API Key that is used to add new Keys. This is not saved in the database."
    )]
    pub keyadder_key: String,

    #[arg(long, env = "MERGE_TITLE_SIMILARITY", default_value = "0.8")]
    pub merge_title_similarity: f32,
}

impl Configuration {
    pub async fn build_mailer(&self) -> Result<SmtpTransport> {
        if self.mail_server.is_none()
            || self.mail_user.is_none()
            || self.mail_password.is_none()
            || self.mail_sender.is_none()
            || self.mail_recipient.is_none()
        {
            return Err(LTZFError::Infrastructure {
                source: error::InfrastructureError::Configuration {
                    message: "Mail Configuration is incomplete".into(),
                    config: self.clone(),
                },
            });
        }
        let mailer = SmtpTransport::relay(self.mail_server.as_ref().unwrap().as_str())?
            .credentials(Credentials::new(
                self.mail_user.clone().unwrap(),
                self.mail_password.clone().unwrap(),
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
    dotenv::dotenv().ok();
    init_tracing();

    let config = Configuration::init();
    tracing::debug!("Configuration: {:?}", &config);

    tracing::info!("Starting the Initialisation process");
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    tracing::debug!("Started Listener");
    let sqlx_db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.db_url)
        .await?;

    let mut available = false;
    for i in 0..14 {
        let r = sqlx_db.acquire().await;
        match r {
            Ok(_) => {
                available = true;
                break;
            }
            Err(sqlx::Error::PoolTimedOut) => {
                tracing::warn!("Connection to Database `{}` timed out", config.db_url);
            }
            _ => {
                let _ = r?;
            }
        }
        let milliseconds = 2i32.pow(i) as u64;
        tracing::info!("DB Unavailable, Retrying in {} ms...", milliseconds);
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
    }
    if !available {
        return Err(LTZFError::Other {
            message: "Server Connection failed after 10 retries".into(),
        });
    }
    tracing::debug!("Started Database Pool");
    sqlx::migrate!().run(&sqlx_db).await?;
    tracing::debug!("Executed Migrations");

    let mailer = config.build_mailer().await;
    let mailer = if let Err(e) = mailer {
        tracing::warn!(
            "Failed to create mailer: {}\nMailer will not be available",
            e
        );
        None
    } else {
        tracing::debug!("Started Mailer");
        Some(mailer.unwrap())
    };

    // Run Key Administrative Functions
    let keyadder_hash = digest(config.keyadder_key.as_str());

    sqlx::query!(
        "INSERT INTO api_keys(key_hash, scope, created_by)
        VALUES
        ($1, (SELECT id FROM api_scope WHERE value = 'keyadder' LIMIT 1), (SELECT last_value FROM api_keys_id_seq))
        ON CONFLICT DO NOTHING;", keyadder_hash)
    .execute(&sqlx_db).await?;

    let state = Arc::new(LTZFServer::new(sqlx_db, mailer, config));
    tracing::debug!("Constructed Server State");

    // Init Axum router
    let rate_limiter = axum_gcra::RateLimitLayer::<()>::builder()
    .with_default_quota(axum_gcra::gcra::Quota::new(std::time::Duration::from_secs(1), NonZeroU64::new(256).unwrap()))
    .with_global_fallback(true)
    .with_extension(true)
    .default_handle_error();
    let request_size_limit = tower_http::limit::RequestBodyLimitLayer::new(1024*1024*256); // 256 MB
    let app = openapi::server::new(state.clone())
    .layer(request_size_limit)
    .layer(rate_limiter);
    tracing::debug!("Constructed Router");
    tracing::info!(
        "Starting Server on {}:{}",
        state.config.host,
        state.config.port
    );
    // Run the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
