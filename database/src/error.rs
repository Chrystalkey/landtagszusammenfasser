use axum::{http::StatusCode, response::IntoResponse};

use thiserror::Error;
// catch-all error Enumeration for the whole application
#[derive(Error, Debug)]
pub enum LTZFError {
    #[error("Database Operation Error: {0}")]
    DatabaseError(#[from] diesel::result::Error),

    #[error("Database Interaction Error: {0}")]
    DBInteractionError(#[from] diesel_interaction::DieselInteractionError),

    #[error("Database Connection error: {0}")]
    DBConnectionError(#[from] deadpool_diesel::PoolError),
    
    #[error("Database Migrations error: {0}")]
    MigrationsError(#[from] diesel_migrations::MigrationError),
    
    #[error("Network Connection Error: {0}")]
    ConnectionError(#[from] axum::Error),
    #[error("Data was received in the wrong format: {0}")]
    ParsingError(String),

    #[error("Server Error: This is the wrong endpoint, expected {0}")]
    WrongEndpoint(String),

    #[error("NotFound: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl IntoResponse for LTZFError {
    fn into_response(self) -> axum::response::Response {
        let (status, message)= match self{
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error. Please try again later."),
        };
        (status, message).into_response()
    }
}