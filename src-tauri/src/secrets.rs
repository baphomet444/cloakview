use crate::error::{AppError, AppResult};
use std::sync::OnceLock;

const SERVICE: &str = "cloakview";
const ACCOUNT: &str = "openai-api-key";

static ENTRY: OnceLock<keyring::Entry> = OnceLock::new();

fn entry() -> AppResult<&'static keyring::Entry> {
    if let Some(e) = ENTRY.get() {
        return Ok(e);
    }
    let created = keyring::Entry::new(SERVICE, ACCOUNT)?;
    Ok(ENTRY.get_or_init(|| created))
}

pub(crate) fn get_api_key() -> AppResult<String> {
    let e = entry()?;
    match e.get_password() {
        Ok(k) if !k.trim().is_empty() => Ok(k),
        _ => Err(AppError::MissingApiKey),
    }
}

pub fn save_api_key(key: String) -> AppResult<()> {
    let trimmed = key.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::MissingApiKey);
    }
    Ok(entry()?.set_password(&trimmed)?)
}

pub fn has_api_key() -> AppResult<bool> {
    Ok(get_api_key().is_ok())
}

pub fn clear_api_key() -> AppResult<()> {
    match entry()?.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Both tests share the single process-global entry (and its mock store),
    // so they must not run concurrently.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn roundtrip_save_has_get_clear() {
        let _guard = TEST_LOCK.lock().unwrap();
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let _ = clear_api_key();

        assert!(!has_api_key().unwrap());
        save_api_key(" sk-test-123 ".into()).unwrap(); // trims whitespace
        assert!(has_api_key().unwrap());
        assert_eq!(get_api_key().unwrap(), "sk-test-123");
        clear_api_key().unwrap();
        assert!(!has_api_key().unwrap());
    }

    #[test]
    fn empty_key_rejected_and_missing_key_typed() {
        let _guard = TEST_LOCK.lock().unwrap();
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let _ = clear_api_key();

        assert!(matches!(save_api_key("   ".into()), Err(AppError::MissingApiKey)));
        assert!(matches!(get_api_key(), Err(AppError::MissingApiKey)));
    }
}
