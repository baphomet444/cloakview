//! Keys live in the macOS Keychain and are read ONLY from Rust.
//! Tauri commands must never return a key to the webview, and the
//! frontend must never send one back after the initial save.
//!
//! Wire this to the `keyring` crate in the app (kept out of this crate
//! so it compiles and tests anywhere).

pub trait SecretStore: Send + Sync {
    fn get(&self, provider_id: &str) -> Option<String>;
    fn set(&self, provider_id: &str, key: &str) -> Result<(), String>;
    fn delete(&self, provider_id: &str) -> Result<(), String>;
}
