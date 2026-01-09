use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to execute defaults command: {0}")]
    DefaultsCommand(String),

    #[error("Failed to parse plist: {0}")]
    PlistParse(#[from] plist::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
