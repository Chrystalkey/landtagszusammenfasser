use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RetrievalError{
    #[error("No matching Database Entry found")]
    NoMatch,
    #[error("Multiple matching Database Entries found: {0}")]
    AmbiguousMatch(String),
}
#[derive(Error, Debug)]
pub enum DatabaseError{
    #[error("Database Operation Error: {0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Database Operation Error: {0}")]
    DatabaseError(String),
    #[error("Async Interaction Error: {0}")]
    AsyncError(#[from] diesel_interaction::DieselInteractionError),

    #[error("Database Interaction Error: {0}")]
    DBInteractionError(#[from] deadpool_diesel::InteractError),

    #[error("Database Connection error: {0}")]
    DBConnectionError(#[from] deadpool_diesel::PoolError),
    
    #[error("Database Migrations error: {0}")]
    MigrationsError(#[from] diesel_migrations::MigrationError),

    #[error("Required Field Missing to complete Insert: {0}")]
    MissingFieldForInsert(String)
}
#[derive(Error, Debug)] 
pub enum ParsingError{
    #[error("Uuid data was received in the wrong format: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Data was received in the wrong format: {0}")]
    Internal(String),
}

// catch-all error Enumeration for the whole application
#[derive(Error, Debug)]
pub enum LTZFError {
    #[error("Database Error: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Parsing Error: {0}")]
    ParsingError(#[from] ParsingError),
    #[error("Network Connection Error: {0}")]
    ConnectionError(#[from] axum::Error),
    #[error("Rerieval Error: {0}")]
    RetrievalError(#[from] RetrievalError),

    #[error("Server Error: This is the wrong endpoint, expected {0}")]
    WrongEndpoint(String),

    #[error("NotFound: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}
pub type Result<T> = std::result::Result<T, LTZFError>;

impl IntoResponse for LTZFError {
    fn into_response(self) -> axum::response::Response {
        match self{
            LTZFError::DatabaseError(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            LTZFError::ParsingError(error) => (StatusCode::BAD_REQUEST, error.to_string()),
            LTZFError::ConnectionError(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            LTZFError::RetrievalError(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            LTZFError::WrongEndpoint(endpoint) => (StatusCode::BAD_REQUEST, format!("Wrong Endpoint: {}", endpoint)),
            LTZFError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            LTZFError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
        }.into_response()
    }
}