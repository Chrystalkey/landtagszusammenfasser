use axum::{http::StatusCode, response::IntoResponse};
use diesel::ConnectionError;
use thiserror::Error;

// errors happening in the server part of the application
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Network Connection Error: {0}")]
    ConnectionError(#[from] axum::Error),
    #[error("Data was received in the wrong format: {0}")]
    ParsingError(String),

    #[error("NotFound: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

// errors happening in the database part of the application
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Connection error: {0}")]
    ConnectionError(#[from] deadpool_diesel::PoolError),
    #[error("Migrations error: {0}")]
    MigrationsError(#[from] diesel_migrations::MigrationError),
}

// catch-all error Enumeration for the whole application
#[derive(Error, Debug)]
pub enum LTZFError {
    #[error("Database Error: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Server Error: {0}")]
    ServerError(#[from] ServerError),
}

impl IntoResponse for LTZFError {
    fn into_response(self) -> axum::response::Response {
        let (status, message)= match self{
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database Handling Error"),
            Self::ServerError(server_error) => 
            match server_error {
                ServerError::NotFound(_) => (StatusCode::NOT_FOUND, "The requested resource was not found"),
                ServerError::ParsingError(_) => (StatusCode::BAD_REQUEST, "The request was not in the expected format"),
                ServerError::ConnectionError(_) => (StatusCode::BAD_REQUEST, "There was an error during transmission"),
                _ => {
                    #[cfg(not(debug_assertions))]
                    {(StatusCode::INTERNAL_SERVER_ERROR, "This is hopefully unreachable. If you see this, please report it.")}
                    #[cfg(debug_assertions)]
                    unreachable!("This is unknown space, please discover new civilizations and see where your life went wrong.");
                }
            }
            _ => {todo!()}
        };
        (status, message).into_response()
    }
}