/// Error taxonomy with structured, actionable messages.
#[derive(Debug, uniffi::Error)]
pub enum SeyfrError {
    Io { message: String },
    InvalidTicket { message: String },
    Network { message: String },
    Store { message: String },
    Cancelled,
    FileNotFound { path: String },
    NotADirectory { path: String },
    EmptyFolder { path: String },
    Timeout,
    Internal { message: String },
    PathTraversal { path: String, message: String },
    InvalidPath { message: String },
}


impl std::fmt::Display for SeyfrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeyfrError::Io { message } => write!(f, "io error: {}", message),
            SeyfrError::InvalidTicket { message } => write!(f, "invalid ticket: {}", message),
            SeyfrError::Network { message } => write!(f, "network error: {}", message),
            SeyfrError::Store { message } => write!(f, "store error: {}", message),
            SeyfrError::Cancelled => write!(f, "transfer cancelled"),
            SeyfrError::FileNotFound { path } => write!(f, "file not found: {}", path),
            SeyfrError::NotADirectory { path } => write!(f, "not a directory: {}", path),
            SeyfrError::EmptyFolder { path } => write!(f, "empty folder: {}", path),
            SeyfrError::Timeout => write!(f, "transfer timeout"),
            SeyfrError::Internal { message } => write!(f, "internal error: {}", message),
            SeyfrError::PathTraversal { path, message } => write!(f, "path traversal attempt: {} - {}", path, message),
            SeyfrError::InvalidPath { message } => write!(f, "invalid path: {}", message),
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
            message: e.to_string(),
        }
    }
}