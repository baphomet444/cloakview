use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("openai api key not configured")]
    MissingApiKey,
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("database error: {0}")]
    Db(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("serialization error: {0}")]
    Serialize(String),
    #[error("screen recording permission denied")]
    PermissionDenied,
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum IpcError {
    MissingApiKey,
    Keychain(String),
    Db(String),
    Audio(String),
    Network(String),
    Serialize(String),
    PermissionDenied,
}

impl From<AppError> for IpcError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::MissingApiKey => IpcError::MissingApiKey,
            AppError::Keychain(m) => IpcError::Keychain(m),
            AppError::Db(m) => IpcError::Db(m),
            AppError::Audio(m) => IpcError::Audio(m),
            AppError::Network(m) => IpcError::Network(m),
            AppError::Serialize(m) => IpcError::Serialize(m),
            AppError::PermissionDenied => IpcError::PermissionDenied,
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        AppError::Keychain(e.to_string())
    }
}
impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Db(e.to_string())
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialize(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_key_serializes_as_typed_tag() {
        let ipc: IpcError = AppError::MissingApiKey.into();
        let json = serde_json::to_value(ipc).unwrap();
        assert_eq!(json["kind"], "MissingApiKey");
    }

    #[test]
    fn db_error_converts() {
        let raw: rusqlite::Error = rusqlite::Error::InvalidQuery;
        let e: AppError = raw.into();
        assert!(matches!(e, AppError::Db(_)));
    }
}
