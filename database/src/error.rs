use snafu::prelude::*;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, LTZFError>;
macro_rules! error_from {
    ($from:ty, $to:ident, $suberr:ident, $variant:ident) => {
        impl From<$from> for LTZFError {
            fn from(source: $from) -> Self {
                Self::$to{ source: $suberr::$variant { source }}
            }
        }
    };
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum DataValidationError {
    #[snafu(display("Required field missing: {field}"))]
    MissingField { field: String },
    
    #[snafu(display("Invalid format for field {field}: {message}"))]
    InvalidFormat { 
        field: String,
        message: String,
    },
    
    #[snafu(display("{msg}"))]
    InvalidEnumValue {
        msg: String
    },
    
    #[snafu(display("Duplicate API ID: {id}"))]
    DuplicateApiId { id: Uuid },
    
    #[snafu(display("Ambiguous match found: {message}"))]
    AmbiguousMatch { message: String },
    
    #[snafu(display("Multiple merge candidates found: {candidates:?}"))]
    MultipleMergeCandidates { candidates: Vec<Uuid> },
    
    #[snafu(display("UUID parsing error: {source}"))]
    UuidParse { source: uuid::Error },
}

error_from!(uuid::Error, Validation, DataValidationError, UuidParse);

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum DatabaseError {
    #[snafu(display("Database operation error: {source}"))]
    Operation { source: diesel::result::Error },
    
    #[snafu(display("Connection pool error: {source}"))]
    Pool { source: deadpool_diesel::PoolError },
    
    #[snafu(display("Deadpool error: {source}"))]
    DeadpoolError { source: deadpool_diesel::Error },
    
    #[snafu(display("Database interaction error: {source}"))]
    Interaction { source: deadpool_diesel::InteractError },
    
    #[snafu(display("Migration error: {source}"))]
    Migration { source: diesel_migrations::MigrationError },
    
    #[snafu(display("Pool build error: {source}"))]
    Build { source: deadpool::managed::BuildError },

    #[snafu(display("{source}"))]
    Unknown { source: Box<dyn std::error::Error + Sync + Send> },
}

error_from!(diesel::result::Error, Database, DatabaseError, Operation);
error_from!(deadpool_diesel::PoolError, Database, DatabaseError, Pool);
error_from!(deadpool_diesel::Error, Database, DatabaseError, DeadpoolError);
error_from!(deadpool_diesel::InteractError, Database, DatabaseError, Interaction);
error_from!(diesel_migrations::MigrationError, Database, DatabaseError, Migration);
error_from!(deadpool::managed::BuildError, Database, DatabaseError, Build);
error_from!(Box<dyn std::error::Error + Sync + Send>, Database, DatabaseError, Unknown);

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum InfrastructureError {
    #[snafu(display("Server error: {source}"))]
    Server { source: axum::Error },
    
    #[snafu(display("Hardware error: {source}"))]
    Hardware { source: std::io::Error },
    
    #[snafu(display("Mail delivery error: {source}"))]
    Mail { source: lettre::transport::smtp::Error },
    
    #[snafu(display("Environment variable error: {source}"))]
    Environment { source: std::env::VarError },
    
    #[snafu(display("Configuration error: {message}"))]
    Configuration { message: String, config: crate::Configuration },
}

error_from!(axum::Error, Infrastructure, InfrastructureError, Server);
error_from!(std::io::Error, Infrastructure, InfrastructureError, Hardware);
error_from!(lettre::transport::smtp::Error, Infrastructure, InfrastructureError, Mail);
error_from!(std::env::VarError, Infrastructure, InfrastructureError, Environment);

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum LTZFError {
    #[snafu(display("Validation error: {source}"))]
    Validation { source: DataValidationError },
    
    #[snafu(display("Database error: {source}"))]
    Database { source: DatabaseError },
    
    #[snafu(display("Infrastructure error: {source}"))]
    Infrastructure { source: InfrastructureError },
    
    #[snafu(display("HTTP header conversion error: {source}"))]
    HeaderConversion { source: axum::http::header::ToStrError },
    
    #[snafu(display("{message}"))]
    Other { message: String },
}
impl From<DataValidationError> for LTZFError {
    fn from(source: DataValidationError) -> Self {
        Self::Validation { source }
    }
}
impl From<DatabaseError> for LTZFError {
    fn from(source: DatabaseError) -> Self {
        Self::Database { source }
    }
}
impl From<InfrastructureError> for LTZFError {
    fn from(source: InfrastructureError) -> Self {
        Self::Infrastructure { source }
    }
}
impl From<axum::http::header::ToStrError> for LTZFError {
    fn from(source: axum::http::header::ToStrError) -> Self {
        Self::HeaderConversion { source }
    }
}

// Helper methods for creating errors
impl LTZFError {
    pub fn other<T: Into<String>>(message: T) -> Self {
        LTZFError::Other { 
            message: message.into() 
        }
    }
}