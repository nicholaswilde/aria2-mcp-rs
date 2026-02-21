use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Aria2 RPC error: {0}")]
    Aria2(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Internal("test".to_string());
        assert_eq!(err.to_string(), "Internal error: test");

        let err = Error::Config("test".to_string());
        assert_eq!(err.to_string(), "Configuration error: test");

        let err = Error::Aria2("test".to_string());
        assert_eq!(err.to_string(), "Aria2 RPC error: test");

        let err = Error::Mcp("test".to_string());
        assert_eq!(err.to_string(), "MCP error: test");
    }
}
