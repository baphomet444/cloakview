//! The CloakView Pro license: what unlocks the Pro features, kept on this Mac.
//!
//! Only local state for now. `activate_license` stores the key as
//! `PendingValidation`; the one-time online check against Lemon Squeezy (which
//! flips it to `Active`, or back to `Inactive` for a rejected key) is wired when
//! the store and product IDs exist. Until then a stored key counts as Pro, so
//! the gating in `commands` is fully exercisable. The record lives in
//! `license.json` next to `settings.json`; the key never leaves the machine
//! except for that one activation call.

use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

/// The local record of a CloakView Pro license. Serialized with a `status` tag so
/// the frontend's `License` type is a plain discriminated union. Newtype
/// variants: a struct variant would nest the key one level deeper.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "status", content = "key")]
pub enum License {
    #[default]
    Inactive,
    /// A key is stored but has not been validated online yet (no Lemon Squeezy
    /// wiring). Counts as Pro so the features work; the online check may still
    /// revoke it later.
    PendingValidation(String),
    /// Validated against Lemon Squeezy. Reserved for the activation call.
    Active(String),
}

impl License {
    /// Whether the Pro features are unlocked.
    pub fn is_pro(&self) -> bool {
        matches!(self, License::PendingValidation(_) | License::Active(_))
    }

    /// The stored key, whatever the state. `None` when inactive.
    pub fn key(&self) -> Option<&str> {
        match self {
            License::Inactive => None,
            License::PendingValidation(key) | License::Active(key) => Some(key),
        }
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

pub fn license_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("license.json")
}

/// Missing or unreadable file → `Inactive`, never an error: a corrupt record
/// must not lock a paying user out of the app.
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

    #[test]
    fn only_a_stored_key_is_pro() {
        assert!(!License::Inactive.is_pro());
        assert!(License::PendingValidation("k".into()).is_pro());
        assert!(License::Active("k".into()).is_pro());
    }

    #[test]
    fn require_pro_matches_is_pro() {
        assert!(License::Inactive.require_pro().is_err());
        assert!(License::PendingValidation("k".into()).require_pro().is_ok());
        assert!(License::Active("k".into()).require_pro().is_ok());
        assert!(matches!(
            License::Inactive.require_pro(),
            Err(AppError::ProRequired)
        ));
    }

    #[test]
    fn serializes_as_a_status_tag_with_the_key_as_content() {
        assert_eq!(
            serde_json::to_value(License::Inactive).unwrap(),
            serde_json::json!({ "status": "inactive" })
        );
        assert_eq!(
            serde_json::to_value(License::PendingValidation("ABCD-1234".into())).unwrap(),
            serde_json::json!({ "status": "pendingValidation", "key": "ABCD-1234" })
        );
        assert_eq!(
            serde_json::to_value(License::Active("ABCD-1234".into())).unwrap(),
            serde_json::json!({ "status": "active", "key": "ABCD-1234" })
        );
    }

    #[test]
    fn roundtrip_and_corrupt_fallback() {
        let dir = std::env::temp_dir().join("cloakview-license-test");
        let path = license_path(&dir);
        let _ = std::fs::remove_file(&path);

        // missing file -> inactive
        assert_eq!(load_from(&path), License::Inactive);

        let license = License::PendingValidation("ABCD-1234".into());
        save_to(&path, &license).unwrap();
        assert_eq!(load_from(&path), license);

        // corrupt file -> inactive, no panic
        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(load_from(&path), License::Inactive);
    }
}
