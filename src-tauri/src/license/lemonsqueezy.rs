//! The Lemon Squeezy license API, kept for the keys sold before the switch to
//! Polar. New purchases never come from here; a key bought while Lemon Squeezy was
//! the store still activates, validates and releases against it, so nobody who paid
//! loses Pro because the store changed.

use super::Rejection;

/// The Lemon Squeezy store that sold CloakView Pro. A key from any other store, or
/// for any other product, is refused even if Lemon Squeezy calls it valid.
pub const LEMON_STORE_ID: u64 = 522202;
/// The CloakView Pro product in that store.
pub const LEMON_PRODUCT_ID: u64 = 1524133;

const API: &str = "https://api.lemonsqueezy.com/v1/licenses";

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
    /// True for a key issued by a store in test mode. Real purchases are false.
    /// A response that omits it (an error body) defaults to false, so only a key
    /// Lemon Squeezy positively marks as test is refused.
    #[serde(default)]
    pub test_mode: bool,
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

    /// Whether this is a test-mode key. A response without `license_key` names no
    /// mode, so it is not counted as test (it will fail for another reason).
    fn is_test(&self) -> bool {
        self.license_key.as_ref().is_some_and(|k| k.test_mode)
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

/// The activation id an activation answer grants, or why it does not.
pub fn activation_outcome(resp: &LsResponse) -> Result<String, Rejection> {
    if resp.is_foreign() {
        return Err(Rejection::WrongProduct);
    }
    if resp.is_test() {
        return Err(Rejection::TestMode);
    }
    if resp.activated {
        if let Some(instance) = &resp.instance {
            return Ok(instance.id.clone());
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
    resp.valid
        && !resp.is_foreign()
        && !resp.is_test()
        && !matches!(resp.key_status(), Some("disabled" | "expired"))
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

/// Activates `key` for this Mac and returns the instance id. A key for another
/// product that Lemon Squeezy did activate is released again, so refusing it does
/// not use up one of its slots.
pub async fn activate(key: &str, instance_name: &str) -> Result<String, Rejection> {
    let resp = post("activate", &[("license_key", key), ("instance_name", instance_name)]).await?;
    let outcome = activation_outcome(&resp);
    // A key Lemon Squeezy activated before we refused it (wrong product, or a
    // test-mode key) has taken a slot; release it so refusing costs nothing.
    if matches!(outcome, Err(Rejection::WrongProduct | Rejection::TestMode)) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn activation_grants_pro_with_the_instance_id() {
        let resp = response(serde_json::json!({ "activated": true, "instance": { "id": "inst-1", "name": "CloakView on Mac" } }));
        assert_eq!(activation_outcome(&resp), Ok("inst-1".to_string()));
    }

    #[test]
    fn activation_refuses_another_store_or_product() {
        let other_store = response(serde_json::json!({
            "activated": true,
            "instance": { "id": "inst-1" },
            "meta": { "store_id": LEMON_STORE_ID + 1, "product_id": LEMON_PRODUCT_ID }
        }));
        assert_eq!(activation_outcome(&other_store), Err(Rejection::WrongProduct));
        let other_product = response(serde_json::json!({
            "activated": true,
            "instance": { "id": "inst-1" },
            "meta": { "store_id": LEMON_STORE_ID, "product_id": LEMON_PRODUCT_ID + 1 }
        }));
        assert_eq!(activation_outcome(&other_product), Err(Rejection::WrongProduct));
    }

    #[test]
    fn activation_reports_why_a_key_was_refused() {
        let limit = response(serde_json::json!({ "activated": false, "error": "This license key has reached the activation limit." }));
        assert_eq!(activation_outcome(&limit), Err(Rejection::LimitReached));

        let disabled = response(serde_json::json!({ "activated": false, "error": "This license key is disabled.", "license_key": { "status": "disabled" } }));
        assert_eq!(activation_outcome(&disabled), Err(Rejection::Disabled));

        let expired = response(serde_json::json!({ "activated": false, "error": "This license key is expired.", "license_key": { "status": "expired" } }));
        assert_eq!(activation_outcome(&expired), Err(Rejection::Expired));

        let unknown: LsResponse = serde_json::from_str(r#"{ "activated": false, "error": "license_key not found." }"#).unwrap();
        assert_eq!(activation_outcome(&unknown), Err(Rejection::Invalid));
    }

    #[test]
    fn a_test_mode_key_is_refused_at_activation_and_at_launch() {
        // Even a key Lemon Squeezy activated for our product is refused when it is test mode.
        let activated_test = response(serde_json::json!({
            "activated": true,
            "instance": { "id": "inst-1" },
            "license_key": { "status": "active", "test_mode": true }
        }));
        assert_eq!(activation_outcome(&activated_test), Err(Rejection::TestMode));
        // And a stored test key no longer passes the launch-time recheck.
        assert!(!still_valid(&response(serde_json::json!({
            "valid": true,
            "license_key": { "status": "active", "test_mode": true }
        }))));
        // A real (production) key is unaffected.
        let real = response(serde_json::json!({ "activated": true, "instance": { "id": "inst-1" } }));
        assert_eq!(activation_outcome(&real), Ok("inst-1".to_string()));
    }

    #[test]
    fn activated_without_an_instance_is_not_trusted() {
        let resp = response(serde_json::json!({ "activated": true }));
        assert_eq!(activation_outcome(&resp), Err(Rejection::Invalid));
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
}
