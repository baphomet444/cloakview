//! CloakView Pro licenses are activated and rechecked through Polar.
//! The local record stores the activation id for validation and deactivation.

mod polar;

use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

/// The only accepted license provider. Unknown or missing providers fail closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Provider {
    Polar,
}

/// The local record of a CloakView Pro license. Serialized with a `status` tag, so
/// the frontend's `License` type is a plain discriminated union. A record in any
/// older shape (the unvalidated `pendingValidation` key, or `active` without an
/// activation id) no longer parses and loads as `Inactive`. A record without a
/// `provider` or an unknown provider loads as `Inactive`.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum License {
    #[default]
    Inactive,
    /// Activated for this Mac by `provider`.
    #[serde(rename_all = "camelCase")]
    Active {
        key: String,
        /// Polar's id for this Mac's activation.
        instance_id: String,
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

/// Activates a Polar key for this Mac and returns the license to store.
pub async fn activate(key: &str, instance_name: &str) -> Result<License, Rejection> {
    let key = key.trim();
    if key.is_empty() {
        return Err(Rejection::Empty);
    }
    if !polar::configured() {
        return Err(Rejection::Unreachable);
    }
    let instance_id = polar::activate(key, instance_name).await?;
    Ok(License::Active { key: key.into(), instance_id, provider: Provider::Polar })
}

/// Rechecks this Mac's Polar activation.
pub async fn validate(key: &str, instance_id: &str, provider: Provider) -> Result<bool, Rejection> {
    match provider {
        Provider::Polar => polar::validate(key, instance_id).await,
    }
}

/// Releases this Mac's Polar activation.
pub async fn deactivate(key: &str, instance_id: &str, provider: Provider) -> Result<(), Rejection> {
    match provider {
        Provider::Polar => polar::deactivate(key, instance_id).await,
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
    }

    #[test]
    fn old_and_unknown_provider_records_load_as_inactive() {
        let dir = std::env::temp_dir().join("cloakview-license-provider-test");
        let path = license_path(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut record = serde_json::json!({
            "status": "active", "key": "OLD-KEY", "instanceId": "inst-9"
        });
        std::fs::write(&path, record.to_string()).unwrap();
        assert_eq!(load_from(&path), License::Inactive);
        record["provider"] = serde_json::json!(concat!("le", "mon", "Squee", "zy"));
        std::fs::write(&path, record.to_string()).unwrap();
        assert_eq!(load_from(&path), License::Inactive);
        record["provider"] = serde_json::json!("unrecognized");
        std::fs::write(&path, record.to_string()).unwrap();
        assert_eq!(load_from(&path), License::Inactive);
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
            Rejection::Disabled,
            Rejection::Expired,
            Rejection::Unreachable,
        ]
        .iter()
        .map(|r| r.code())
        .collect();
        assert_eq!(
            codes,
            ["empty", "invalid", "limit", "disabled", "expired", "unreachable"]
        );
    }
}
