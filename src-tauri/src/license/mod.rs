//! The CloakView Pro license: what unlocks the Pro features, kept on this Mac.
//!
//! A key unlocks Pro only once the store that sold it has activated it for this Mac
//! and confirmed it belongs to the CloakView Pro product. Polar sells it today;
//! keys bought while Lemon Squeezy was the store keep working against Lemon Squeezy,
//! so a change of store never revokes a purchase. Each store has its own module
//! (`polar`, `lemonsqueezy`) speaking its own HTTP API; this module owns the local
//! record and the order the stores are asked in.
//!
//! The activation's id is kept with the key so the launch-time recheck and
//! `Deactivate` (which frees the Mac's activation slot) refer to this Mac's
//! activation. The record lives in `license.json` next to `settings.json`. Both
//! license APIs are public: no API key is shipped in the app.

mod lemonsqueezy;
mod polar;

use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

/// Which store sold the key. Stored with the license so the launch-time recheck and
/// the release ask the store the activation was made with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Provider {
    /// The store CloakView Pro is sold through now.
    Polar,
    /// The store it was sold through before Polar. Kept for keys bought then; a new
    /// purchase never comes from here.
    #[default]
    LemonSqueezy,
}

/// The local record of a CloakView Pro license. Serialized with a `status` tag, so
/// the frontend's `License` type is a plain discriminated union. A record in any
/// older shape (the unvalidated `pendingValidation` key, or `active` without an
/// activation id) no longer parses and loads as `Inactive`. A record without a
/// `provider` was written before the switch, so it is a Lemon Squeezy one.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum License {
    #[default]
    Inactive,
    /// Activated for this Mac by `provider`.
    #[serde(rename_all = "camelCase")]
    Active {
        key: String,
        /// The store's id for this Mac's activation: Polar's activation id, or the
        /// Lemon Squeezy instance id.
        instance_id: String,
        #[serde(default)]
        provider: Provider,
    },
}

impl License {
    /// Whether the Pro features are unlocked.
    pub fn is_pro(&self) -> bool {
        matches!(self, License::Active { .. })
    }

    /// The gate every Pro command runs first: free users get a typed error the
    /// frontend shows as an upgrade prompt, not a generic failure.
    pub fn require_pro(&self) -> AppResult<()> {
        if self.is_pro() {
            Ok(())
        } else {
            Err(AppError::ProRequired)
        }
    }
}

/// Why a key was not activated. Sent to the frontend as the `License` error's
/// message (`code`), which picks the sentence to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    Empty,
    Invalid,
    LimitReached,
    WrongProduct,
    Disabled,
    Expired,
    /// The store could not be reached, or answered with a server error.
    Unreachable,
}

impl Rejection {
    pub fn code(self) -> &'static str {
        match self {
            Rejection::Empty => "empty",
            Rejection::Invalid => "invalid",
            Rejection::LimitReached => "limit",
            Rejection::WrongProduct => "wrongProduct",
            Rejection::Disabled => "disabled",
            Rejection::Expired => "expired",
            Rejection::Unreachable => "unreachable",
        }
    }
}

impl From<Rejection> for AppError {
    fn from(r: Rejection) -> Self {
        AppError::License(r.code().into())
    }
}

/// Activates `key` for this Mac and returns the license to store.
///
/// Polar is asked first, because that is where a new purchase comes from. A key it
/// does not know is not a verdict on the key — it may have been bought in the old
/// store — so Lemon Squeezy gets asked second. Any other refusal (limit reached,
/// disabled, unreachable) is final: it is a statement about a key Polar does know.
pub async fn activate(key: &str, instance_name: &str) -> Result<License, Rejection> {
    let key = key.trim();
    if key.is_empty() {
        return Err(Rejection::Empty);
    }
    if polar::configured() {
        match polar::activate(key, instance_name).await {
            Ok(instance_id) => {
                return Ok(License::Active { key: key.into(), instance_id, provider: Provider::Polar })
            }
            Err(Rejection::Invalid | Rejection::WrongProduct) => {}
            Err(other) => return Err(other),
        }
    }
    let instance_id = lemonsqueezy::activate(key, instance_name).await?;
    Ok(License::Active { key: key.into(), instance_id, provider: Provider::LemonSqueezy })
}

/// Asks the store that sold the key whether this Mac's activation still stands.
/// `Err` only when it could not be asked.
pub async fn validate(key: &str, instance_id: &str, provider: Provider) -> Result<bool, Rejection> {
    match provider {
        Provider::Polar => polar::validate(key, instance_id).await,
        Provider::LemonSqueezy => lemonsqueezy::validate(key, instance_id).await,
    }
}

/// Releases this Mac's activation at the store that sold the key. Any answer from
/// the store (including "no such activation") means there is nothing left to
/// release; only `Unreachable` leaves the slot possibly still taken.
pub async fn deactivate(key: &str, instance_id: &str, provider: Provider) -> Result<(), Rejection> {
    match provider {
        Provider::Polar => polar::deactivate(key, instance_id).await,
        Provider::LemonSqueezy => lemonsqueezy::deactivate(key, instance_id).await,
    }
}

/// The name this Mac's activation gets in the store's dashboard, so the buyer (and
/// support) can tell their Macs apart.
pub fn instance_name() -> String {
    let computer = std::process::Command::new("scutil")
        .args(["--get", "ComputerName"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Mac".into());
    format!("CloakView on {computer}")
}

pub fn license_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("license.json")
}

/// Missing, unreadable or outdated file → `Inactive`, never an error.
pub fn load_from(path: &Path) -> License {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_to(path: &Path, license: &License) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Serialize(e.to_string()))?;
    }
    let raw = serde_json::to_string_pretty(license)?;
    std::fs::write(path, raw).map_err(|e| AppError::Serialize(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active() -> License {
        License::Active { key: "ABCD-1234".into(), instance_id: "act-1".into(), provider: Provider::Polar }
    }

    fn legacy() -> License {
        License::Active { key: "ABCD-1234".into(), instance_id: "inst-1".into(), provider: Provider::LemonSqueezy }
    }

    #[test]
    fn only_an_activated_license_is_pro() {
        assert!(!License::Inactive.is_pro());
        assert!(active().is_pro());
        assert!(matches!(License::Inactive.require_pro(), Err(AppError::ProRequired)));
        assert!(active().require_pro().is_ok());
    }

    #[test]
    fn serializes_as_a_status_tag_with_camel_case_fields() {
        assert_eq!(serde_json::to_value(License::Inactive).unwrap(), serde_json::json!({ "status": "inactive" }));
        assert_eq!(
            serde_json::to_value(active()).unwrap(),
            serde_json::json!({
                "status": "active",
                "key": "ABCD-1234",
                "instanceId": "act-1",
                "provider": "polar"
            })
        );
        assert_eq!(
            serde_json::to_value(legacy()).unwrap(),
            serde_json::json!({
                "status": "active",
                "key": "ABCD-1234",
                "instanceId": "inst-1",
                "provider": "lemonSqueezy"
            })
        );
    }

    #[test]
    fn a_record_written_before_the_switch_reads_as_lemon_squeezy() {
        let dir = std::env::temp_dir().join("cloakview-license-provider-test");
        let path = license_path(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, r#"{ "status": "active", "key": "OLD-KEY", "instanceId": "inst-9" }"#).unwrap();
        assert_eq!(
            load_from(&path),
            License::Active { key: "OLD-KEY".into(), instance_id: "inst-9".into(), provider: Provider::LemonSqueezy }
        );
    }

    #[test]
    fn older_unvalidated_records_load_as_inactive() {
        let dir = std::env::temp_dir().join("cloakview-license-legacy-test");
        let path = license_path(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, r#"{ "status": "pendingValidation", "key": "anything" }"#).unwrap();
        assert_eq!(load_from(&path), License::Inactive);
        std::fs::write(&path, r#"{ "status": "active", "key": "anything" }"#).unwrap();
        assert_eq!(load_from(&path), License::Inactive);
    }

    #[test]
    fn roundtrip_and_corrupt_fallback() {
        let dir = std::env::temp_dir().join("cloakview-license-test");
        let path = license_path(&dir);
        let _ = std::fs::remove_file(&path);
        assert_eq!(load_from(&path), License::Inactive);
        save_to(&path, &active()).unwrap();
        assert_eq!(load_from(&path), active());
        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(load_from(&path), License::Inactive);
    }

    #[test]
    fn rejection_codes_are_what_the_frontend_matches() {
        let codes: Vec<&str> = [
            Rejection::Empty,
            Rejection::Invalid,
            Rejection::LimitReached,
            Rejection::WrongProduct,
            Rejection::Disabled,
            Rejection::Expired,
            Rejection::Unreachable,
        ]
        .iter()
        .map(|r| r.code())
        .collect();
        assert_eq!(
            codes,
            ["empty", "invalid", "limit", "wrongProduct", "disabled", "expired", "unreachable"]
        );
    }
}
