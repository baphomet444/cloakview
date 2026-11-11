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
    #[error("microphone permission denied")]
    MicPermissionDenied,
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("could not open url: {0}")]
    OpenFailed(String),
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
    MicPermissionDenied,
    InvalidUrl(String),
    OpenFailed(String),
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
            AppError::MicPermissionDenied => IpcError::MicPermissionDenied,
            AppError::InvalidUrl(m) => IpcError::InvalidUrl(m),
            AppError::OpenFailed(m) => IpcError::OpenFailed(m),
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        AppError::Keychain(e.to_string())
    }
}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Db(e.to_string())
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
    fn url_errors_serialize_with_kind_and_message() {
        let invalid: IpcError = AppError::InvalidUrl("scheme \"file\" is not allowed".into()).into();
        assert_eq!(
            serde_json::to_value(invalid).unwrap(),
            serde_json::json!({"kind": "InvalidUrl", "message": "scheme \"file\" is not allowed"})
        );
        let failed: IpcError = AppError::OpenFailed("boom".into()).into();
        assert_eq!(
            serde_json::to_value(failed).unwrap(),
            serde_json::json!({"kind": "OpenFailed", "message": "boom"})
        );
    }

    #[test]
    fn db_error_converts() {
        let raw: rusqlite::Error = rusqlite::Error::InvalidQuery;
        let e: AppError = raw.into();
        assert!(matches!(e, AppError::Db(_)));
    }
}
