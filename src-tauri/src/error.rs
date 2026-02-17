use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Failed to read hosts file: {0}")]
    ReadFile(String),

    #[error("Failed to write hosts file: {0}")]
    WriteFile(String),

    #[error("Failed to parse hosts file: {0}")]
    ParseError(String),

    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("Group not found: {0}")]
    GroupNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
