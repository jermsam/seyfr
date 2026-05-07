/// Error taxonomy with structured, actionable messages.
#[derive(Debug, uniffi::Error)]
pub enum SeyfrError {
    Io { details: String },
    InvalidTicket { details: String },
    Network { details: String },
    Store { details: String },
    Cancelled,
    FileNotFound { path: String },
    NotADirectory { path: String },
    EmptyFolder { path: String },
    Timeout,
    Internal { details: String },
    PathTraversal { path: String, details: String },
    InvalidPath { details: String },
}


impl std::fmt::Display for SeyfrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeyfrError::Io { details } => write!(f, "io error: {}", details),
            SeyfrError::InvalidTicket { details } => write!(f, "invalid ticket: {}", details),
            SeyfrError::Network { details } => write!(f, "network error: {}", details),
            SeyfrError::Store { details } => write!(f, "store error: {}", details),
            SeyfrError::Cancelled => write!(f, "transfer cancelled"),
            SeyfrError::FileNotFound { path } => write!(f, "file not found: {}", path),
            SeyfrError::NotADirectory { path } => write!(f, "not a directory: {}", path),
            SeyfrError::EmptyFolder { path } => write!(f, "empty folder: {}", path),
            SeyfrError::Timeout => write!(f, "transfer timeout"),
            SeyfrError::Internal { details } => write!(f, "internal error: {}", details),
            SeyfrError::PathTraversal { path, details } => write!(f, "path traversal attempt: {} - {}", path, details),
            SeyfrError::InvalidPath { details } => write!(f, "invalid path: {}", details),
        }
    }
}

impl std::error::Error for SeyfrError {}

impl SeyfrError {
    pub fn code(&self) -> u16 {
        match self {
            SeyfrError::Io { .. } => 1001,
            SeyfrError::InvalidTicket { .. } => 2001,
            SeyfrError::Network { .. } => 3001,
            SeyfrError::Store { .. } => 4001,
            SeyfrError::Cancelled => 5001,
            SeyfrError::FileNotFound { .. } => 1002,
            SeyfrError::NotADirectory { .. } => 1003,
            SeyfrError::EmptyFolder { .. } => 1004,
            SeyfrError::Timeout => 3002,
            SeyfrError::Internal { .. } => 9001,
            SeyfrError::PathTraversal { .. } => 1005,
            SeyfrError::InvalidPath { .. } => 1006,
        }
    }
}

impl From<std::io::Error> for SeyfrError {
    fn from(e: std::io::Error) -> Self {
        SeyfrError::Io {
            details: e.to_string(),
        }
    }
}