use thiserror::Error;

pub type Result<T> = std::result::Result<T, SubLensError>;

#[derive(Debug, Error)]
pub enum SubLensError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("fetch failed for {origin}: {message}")]
    Fetch { origin: String, message: String },

    #[error("decode failed: {0}")]
    Decode(String),

    #[error("extraction failed: {0}")]
    Extract(String),

    #[error("parse failed for {origin}: {message}")]
    Parse { origin: String, message: String },

    #[error("limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("operation cancelled")]
    Cancelled,

    #[error("diagnostics failed: {0}")]
    Diagnostics(String),

    #[error("export failed: {0}")]
    Export(String),
}
