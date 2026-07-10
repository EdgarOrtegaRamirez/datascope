use std::fmt;

/// Error type for datascope operations.
#[derive(Debug)]
pub enum DatascopeError {
    /// I/O error reading a file or stream.
    Io(String),
    /// CSV parsing error.
    Csv(String),
    /// JSON parsing error.
    Json(String),
    /// Invalid input format or configuration.
    InvalidInput(String),
    /// No data rows found.
    NoData,
}

impl fmt::Display for DatascopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatascopeError::Io(msg) => write!(f, "I/O error: {msg}"),
            DatascopeError::Csv(msg) => write!(f, "CSV error: {msg}"),
            DatascopeError::Json(msg) => write!(f, "JSON error: {msg}"),
            DatascopeError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            DatascopeError::NoData => write!(f, "no data rows found"),
        }
    }
}

impl std::error::Error for DatascopeError {}

impl From<std::io::Error> for DatascopeError {
    fn from(e: std::io::Error) -> Self {
        DatascopeError::Io(e.to_string())
    }
}

impl From<csv::Error> for DatascopeError {
    fn from(e: csv::Error) -> Self {
        DatascopeError::Csv(e.to_string())
    }
}

impl From<serde_json::Error> for DatascopeError {
    fn from(e: serde_json::Error) -> Self {
        DatascopeError::Json(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DatascopeError>;
