use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "code", content = "message")]
pub enum MyDeskError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Disk full")]
    DiskFull,

    #[error("Timeout")]
    Timeout,
}

impl From<std::io::Error> for MyDeskError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => MyDeskError::FileNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => MyDeskError::PermissionDenied(err.to_string()),
            _ => MyDeskError::IoError(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, MyDeskError>;

#[derive(Serialize)]
#[derive(Debug)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl From<MyDeskError> for ErrorResponse {
    fn from(err: MyDeskError) -> Self {
        let code = match &err {
            MyDeskError::FileNotFound(_) => "FILE_NOT_FOUND",
            MyDeskError::PermissionDenied(_) => "PERMISSION_DENIED",
            MyDeskError::InvalidPath(_) => "INVALID_PATH",
            MyDeskError::IoError(_) => "IO_ERROR",
            MyDeskError::UnknownCommand(_) => "UNKNOWN_COMMAND",
            MyDeskError::InvalidRequest(_) => "INVALID_REQUEST",
            MyDeskError::DiskFull => "DISK_FULL",
            MyDeskError::Timeout => "TIMEOUT",
        };

        ErrorResponse {
            code: code.to_string(),
            message: err.to_string(),
        }
    }
}
