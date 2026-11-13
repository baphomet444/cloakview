//! The CloakView Pro license: what unlocks the Pro features, kept on this Mac.
//!
//! A key unlocks Pro only once Lemon Squeezy has activated it for this Mac and
//! confirmed it belongs to the CloakView Pro product of our store. The activation's
//! instance id is kept with the key so the launch-time recheck and `Deactivate`
//! (which frees the Mac's activation slot) refer to this Mac's activation. The
//! record lives in `license.json` next to `settings.json`. The License API is
//! public: no API key is shipped in the app.

use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

/// The Lemon Squeezy store that sells CloakView Pro. A key from any other store, or
/// for any other product, is refused even if Lemon Squeezy calls it valid.
pub const LEMON_STORE_ID: u64 = 0;
/// The CloakView Pro product in that store.
pub const LEMON_PRODUCT_ID: u64 = 0;

const API: &str = "https://api.lemonsqueezy.com/v1/licenses";

/// The local record of a CloakView Pro license. Serialized with a `status` tag, so
/// the frontend's `License` type is a plain discriminated union. A record in any
/// older shape (the unvalidated `pendingValidation` key, or `active` without an
/// instance id) no longer parses and loads as `Inactive`.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum License {
    #[default]
    Inactive,
    /// Activated for this Mac by Lemon Squeezy.
    #[serde(rename_all = "camelCase")]
    Active { key: String, instance_id: String },
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
    /// Lemon Squeezy could not be reached, or answered with a server error.
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

/// The parts of a License API response CloakView reads. Every field is optional:
/// an error response (an unknown key, say) carries little more than `error`.
#[derive(Debug, Default, serde::Deserialize)]
pub struct LsResponse {
    #[serde(default)]
    pub activated: bool,
    #[serde(default)]
    pub valid: bool,
    pub error: Option<String>,
    pub license_key: Option<LsKey>,
    pub instance: Option<LsInstance>,
    pub meta: Option<LsMeta>,
}

#[derive(Debug, serde::Deserialize)]
pub struct LsKey {
    pub status: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct LsInstance {
    pub id: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct LsMeta {
    pub store_id: u64,
    pub product_id: u64,
}

impl LsResponse {
    /// Whether the key is for a store or product other than CloakView Pro. A
    /// response without `meta` names no product, so it is not counted as foreign.
    fn is_foreign(&self) -> bool {
        self.meta
            .as_ref()
            .is_some_and(|m| m.store_id != LEMON_STORE_ID || m.product_id != LEMON_PRODUCT_ID)
    }

    fn key_status(&self) -> Option<&str> {
        self.license_key.as_ref().map(|k| k.status.as_str())
    }
}

/// Reads an HTTP answer from the License API. A 4xx carries a JSON verdict on the
/// key (unknown key, limit reached) and is returned like a success; anything the
/// API did not decide (a 5xx, a non-JSON body) is `Unreachable`.
pub fn interpret_http(status: u16, body: &str) -> Result<LsResponse, Rejection> {
    if status >= 500 {
        return Err(Rejection::Unreachable);
    }
    serde_json::from_str(body).map_err(|_| Rejection::Unreachable)
}

/// The license an activation answer grants, or why it does not.
pub fn activation_outcome(resp: &LsResponse, key: &str) -> Result<License, Rejection> {
    if resp.is_foreign() {
        return Err(Rejection::WrongProduct);
    }
    if resp.activated {
        if let Some(instance) = &resp.instance {
            return Ok(License::Active { key: key.into(), instance_id: instance.id.clone() });
        }
    }
    match resp.key_status() {
        Some("disabled") => return Err(Rejection::Disabled),
        Some("expired") => return Err(Rejection::Expired),
        _ => {}
    }
    let error = resp.error.as_deref().unwrap_or_default().to_ascii_lowercase();
    if error.contains("activation limit") {
        Err(Rejection::LimitReached)
    } else {
        Err(Rejection::Invalid)
    }
}

/// Whether a launch-time validation answer still allows Pro. Refunded or disabled
/// keys, expired keys and deactivated instances all come back not `valid`.
pub fn still_valid(resp: &LsResponse) -> bool {
    resp.valid && !resp.is_foreign() && !matches!(resp.key_status(), Some("disabled" | "expired"))
}

async fn post(endpoint: &str, form: &[(&str, &str)]) -> Result<LsResponse, Rejection> {
    let resp = reqwest::Client::new()
        .post(format!("{API}/{endpoint}"))
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(15))
        .form(form)
        .send()
        .await
        .map_err(|_| Rejection::Unreachable)?;
    let status = resp.status().as_u16();
    let body = resp.text().await.map_err(|_| Rejection::Unreachable)?;
    interpret_http(status, &body)
}

/// Activates `key` for this Mac. A key for another product that Lemon Squeezy did
/// activate is released again, so refusing it does not use up one of its slots.
pub async fn activate(key: &str, instance_name: &str) -> Result<License, Rejection> {
    let key = key.trim();
    if key.is_empty() {
        return Err(Rejection::Empty);
    }
    let resp = post("activate", &[("license_key", key), ("instance_name", instance_name)]).await?;
    let outcome = activation_outcome(&resp, key);
    if outcome == Err(Rejection::WrongProduct) {
        if let Some(instance) = &resp.instance {
            let _ = post("deactivate", &[("license_key", key), ("instance_id", &instance.id)]).await;
        }
    }
    outcome
}

/// Asks Lemon Squeezy whether this Mac's activation still stands. `Err` only when
/// it could not be asked.
pub async fn validate(key: &str, instance_id: &str) -> Result<bool, Rejection> {
    let resp = post("validate", &[("license_key", key), ("instance_id", instance_id)]).await?;
    Ok(still_valid(&resp))
}

/// Releases this Mac's activation. Any answer from Lemon Squeezy (including "no
/// such instance") means there is nothing left to release; only `Unreachable`
/// leaves the slot possibly still taken.
pub async fn deactivate(key: &str, instance_id: &str) -> Result<(), Rejection> {
    post("deactivate", &[("license_key", key), ("instance_id", instance_id)]).await.map(|_| ())
}

/// The name this Mac's activation gets in the Lemon Squeezy dashboard, so the buyer
/// (and support) can tell their Macs apart.
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
        License::Active { key: "ABCD-1234".into(), instance_id: "inst-1".into() }
    }

    /// A response shaped like Lemon Squeezy's documented examples, for our product.
    fn response(extra: serde_json::Value) -> LsResponse {
        let mut base = serde_json::json!({
            "error": null,
            "license_key": { "id": 1, "status": "active", "key": "ABCD-1234", "activation_limit": 2, "activation_usage": 1 },
            "meta": { "store_id": LEMON_STORE_ID, "product_id": LEMON_PRODUCT_ID, "variant_id": 5 }
        });
        for (k, v) in extra.as_object().unwrap() {
            base[k] = v.clone();
        }
        serde_json::from_value(base).unwrap()
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
            serde_json::json!({ "status": "active", "key": "ABCD-1234", "instanceId": "inst-1" })
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
    fn activation_grants_pro_with_the_instance_id() {
        let resp = response(serde_json::json!({ "activated": true, "instance": { "id": "inst-1", "name": "CloakView on Mac" } }));
        assert_eq!(activation_outcome(&resp, "ABCD-1234"), Ok(active()));
    }

    #[test]
    fn activation_refuses_another_store_or_product() {
        let other_store = response(serde_json::json!({
            "activated": true,
            "instance": { "id": "inst-1" },
            "meta": { "store_id": LEMON_STORE_ID + 1, "product_id": LEMON_PRODUCT_ID }
        }));
        assert_eq!(activation_outcome(&other_store, "k"), Err(Rejection::WrongProduct));
        let other_product = response(serde_json::json!({
            "activated": true,
            "instance": { "id": "inst-1" },
            "meta": { "store_id": LEMON_STORE_ID, "product_id": LEMON_PRODUCT_ID + 1 }
        }));
        assert_eq!(activation_outcome(&other_product, "k"), Err(Rejection::WrongProduct));
    }

    #[test]
    fn activation_reports_why_a_key_was_refused() {
        let limit = response(serde_json::json!({ "activated": false, "error": "This license key has reached the activation limit." }));
        assert_eq!(activation_outcome(&limit, "k"), Err(Rejection::LimitReached));

        let disabled = response(serde_json::json!({ "activated": false, "error": "This license key is disabled.", "license_key": { "status": "disabled" } }));
        assert_eq!(activation_outcome(&disabled, "k"), Err(Rejection::Disabled));

        let expired = response(serde_json::json!({ "activated": false, "error": "This license key is expired.", "license_key": { "status": "expired" } }));
        assert_eq!(activation_outcome(&expired, "k"), Err(Rejection::Expired));

        let unknown: LsResponse = serde_json::from_str(r#"{ "activated": false, "error": "license_key not found." }"#).unwrap();
        assert_eq!(activation_outcome(&unknown, "k"), Err(Rejection::Invalid));
    }

    #[test]
    fn activated_without_an_instance_is_not_trusted() {
        let resp = response(serde_json::json!({ "activated": true }));
        assert_eq!(activation_outcome(&resp, "k"), Err(Rejection::Invalid));
    }

    #[test]
    fn validation_keeps_pro_only_for_a_live_key_of_our_product() {
        assert!(still_valid(&response(serde_json::json!({ "valid": true }))));
        assert!(!still_valid(&response(serde_json::json!({ "valid": false, "error": "instance not found" }))));
        assert!(!still_valid(&response(serde_json::json!({ "valid": true, "license_key": { "status": "disabled" } }))));
        assert!(!still_valid(&response(serde_json::json!({ "valid": true, "license_key": { "status": "expired" } }))));
        assert!(!still_valid(&response(serde_json::json!({
            "valid": true,
            "meta": { "store_id": LEMON_STORE_ID + 1, "product_id": LEMON_PRODUCT_ID }
        }))));
    }

    #[test]
    fn http_errors_without_a_verdict_are_unreachable() {
        assert_eq!(interpret_http(503, "{}").err(), Some(Rejection::Unreachable));
        assert_eq!(interpret_http(200, "<html>").err(), Some(Rejection::Unreachable));
        let not_found = interpret_http(404, r#"{ "error": "license_key not found." }"#).unwrap();
        assert_eq!(not_found.error.as_deref(), Some("license_key not found."));
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
        assert_eq!(codes, ["empty", "invalid", "limit", "wrongProduct", "disabled", "expired", "unreachable"]);
    }
}
