use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;


pub type Result<T> = std::result::Result<T, LTZFError>;

#[derive(Error, Debug)]
pub enum DatabaseError{
    #[error("Database Operation Error: {0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Async Interaction Error: {0}")]
    AsyncError(#[from] diesel_interaction::DieselInteractionError),

    #[error("Database Interaction Error: {0}")]
    DBInteractionError(#[from] deadpool_diesel::InteractError),

    #[error("Database Connection error: {0}")]
    DBConnectionError(#[from] deadpool_diesel::PoolError),
    
    #[error("Database Migrations error: {0}")]
    MigrationsError(#[from] diesel_migrations::MigrationError),

    #[error("Required Field Missing to complete Insert: {0}")]
    MissingFieldForInsert(String), 

    #[error("The same API Id was already supplied: {0}")]
    ApiIDEqual(uuid::Uuid),

    #[error("Multiple Merge Candidates found: {0:?} for {1:?}")]
    MultipleMergeCandidates(Vec<i32>, crate::infra::api::Gesetzesvorhaben)
}

#[derive(Error, Debug)] 
pub enum ParsingError{
    #[error("Uuid data was received in the wrong format: {0}")]
    UuidError(#[from] uuid::Error),
    #[error("Header Could not be converted to String{0}")]
    HeaderToStringError(#[from] axum::http::header::ToStrError),
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
}

impl IntoResponse for LTZFError {
    fn into_response(self) -> axum::response::Response {
        match self{
            LTZFError::DatabaseError(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            LTZFError::ParsingError(error) => (StatusCode::BAD_REQUEST, error.to_string()),
            LTZFError::ConnectionError(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
        }.into_response()
    }
}