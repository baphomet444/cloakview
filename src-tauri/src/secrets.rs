use crate::error::{AppError, AppResult};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

const SERVICE: &str = "cloakview";

/// The Keychain account every provider's key lives under: `{id}-api-key`.
/// For "openai" this spells `openai-api-key` — the exact account pre-0.3
/// builds used — so existing installs keep their key with no migration and
/// rollback keeps working. Entries are never deleted on upgrade, only when
/// the user clears a key.
fn account(provider_id: &str) -> String {
    format!("{provider_id}-api-key")
}

static ENTRIES: OnceLock<Mutex<HashMap<String, keyring::Entry>>> = OnceLock::new();

/// Guards the process-global keyring mock state that every test in this
/// crate touches after calling
/// `keyring::set_default_credential_builder(keyring::mock::default_credential_builder())`:
/// the mock stores data in the entry itself, so tests must not run their
/// keychain phases concurrently.
#[cfg(test)]
pub(crate) static TEST_LOCK: Mutex<()> = Mutex::new(());

/// Runs `f` against the cached `Entry` for `provider_id`, creating it on
/// first use. Entries are cached per provider id: the real Keychain doesn't
/// care, but the test mock stores data in the entry itself, so a fresh
/// `Entry` per call would lose the key between save and read.
fn with_entry<T>(provider_id: &str, f: impl FnOnce(&keyring::Entry) -> keyring::Result<T>) -> keyring::Result<T> {
    let map = ENTRIES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = map
        .lock()
        .map_err(|e| keyring::Error::Invalid("entry cache".into(), e.to_string()))?;
    if !map.contains_key(provider_id) {
        let created = keyring::Entry::new(SERVICE, &account(provider_id))?;
        map.insert(provider_id.to_string(), created);
    }
    let entry = map.get(provider_id).expect("entry just ensured");
    f(entry)
}

fn read(provider_id: &str) -> AppResult<String> {
    match with_entry(provider_id, |e| e.get_password()) {
        Ok(k) if !k.trim().is_empty() => Ok(k),
        _ => Err(AppError::ProviderKeyMissing(provider_id.into())),
    }
}

/// The key for `provider_id`, read only by Rust, never sent to the webview.
pub fn get_key(provider_id: &str) -> AppResult<String> {
    read(provider_id)
}

/// Stores the key for `provider_id` in the Keychain. Blank is refused.
pub fn save_key(provider_id: &str, key: String) -> AppResult<()> {
    let trimmed = key.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::ProviderKeyMissing(provider_id.into()));
    }
    with_entry(provider_id, |e| e.set_password(&trimmed)).map_err(AppError::from)
}

pub fn has_key(provider_id: &str) -> AppResult<bool> {
    Ok(read(provider_id).is_ok())
}

pub fn clear_key(provider_id: &str) -> AppResult<()> {
    match with_entry(provider_id, |e| e.delete_credential()) {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

// --- the pre-provider API, kept as aliases over the "openai" slot ---
// The frontend migrated to save_key/clear_key; these remain so the legacy
// IPC commands keep working. `openai-api-key` is the same Keychain account
// either way.

pub fn save_api_key(key: String) -> AppResult<()> {
    save_key("openai", key)
}

pub fn has_api_key() -> AppResult<bool> {
    has_key("openai")
}

pub fn clear_api_key() -> AppResult<()> {
    clear_key("openai")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock() {
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
    }

    /// Every provider gets its own Keychain account: saving one key leaves
    /// the others alone, clearing one leaves the others alone. The openai
    /// slot's account IS the legacy `openai-api-key` entry — asserting the
    /// name is the whole migration; existing installs' keys sit in exactly
    /// that account and nothing is moved or copied.
    #[test]
    fn per_provider_slots_are_isolated_and_the_openai_account_is_the_legacy_name() {
        let _guard = TEST_LOCK.lock().unwrap();
        mock();
        let _ = clear_key("openai");
        let _ = clear_key("groq");
        let _ = clear_key("together");

        // The legacy account name is what the openai slot resolves to.
        assert_eq!(account("openai"), "openai-api-key");

        save_key("openai", "sk-legacy".into()).unwrap();
        assert_eq!(get_key("openai").unwrap(), "sk-legacy");
        // No other provider sees it.
        assert!(matches!(get_key("groq"), Err(AppError::ProviderKeyMissing(_))));

        save_key("groq", " gsk-groq ".into()).unwrap();
        assert_eq!(get_key("groq").unwrap(), "gsk-groq");
        assert_eq!(get_key("openai").unwrap(), "sk-legacy", "openai untouched");

        clear_key("openai").unwrap();
        assert!(matches!(get_key("openai"), Err(AppError::ProviderKeyMissing(_))));
        assert_eq!(get_key("groq").unwrap(), "gsk-groq", "groq untouched");

        // Blank keys are refused, never stored.
        assert!(matches!(save_key("together", "   ".into()), Err(AppError::ProviderKeyMissing(_))));
        assert!(matches!(get_key("together"), Err(AppError::ProviderKeyMissing(_))));

        clear_key("groq").unwrap();
    }

    #[test]
    fn clear_missing_key_is_ok() {
        let _guard = TEST_LOCK.lock().unwrap();
        mock();
        let _ = clear_key("together");
        assert!(clear_key("together").is_ok());
    }

    /// Round-trip through the legacy aliases lands in the same slot.
    #[test]
    fn legacy_aliases_use_the_openai_slot() {
        let _guard = TEST_LOCK.lock().unwrap();
        mock();
        let _ = clear_api_key();
        assert!(!has_api_key().unwrap());
        save_api_key(" sk-alias ".into()).unwrap();
        assert!(has_api_key().unwrap());
        assert_eq!(get_key("openai").unwrap(), "sk-alias");
        clear_api_key().unwrap();
        assert!(matches!(get_key("openai"), Err(AppError::ProviderKeyMissing(_))));
    }
}
