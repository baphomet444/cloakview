//! The Polar customer-portal license API: where CloakView Pro is sold now.
//!
//! Polar issues the key itself through the **license keys** benefit on the
//! CloakView Pro product (no key issuance code lives here), and its customer-portal
//! endpoints need no authentication, which is what makes them usable from a public
//! desktop binary. Activation reserves a device slot on the key, so the
//! "release Mac" control works.
//!
//! API: <https://polar.sh/docs/api-reference> (version 2026-10).

use super::Rejection;

/// Production. A sandbox key only exists under `sandbox-api.polar.sh`, so a test
/// purchase can never unlock Pro in a shipped build: it simply is not a key this
/// host knows. Override at build time (`CLOAKVIEW_POLAR_API=https://sandbox-api.polar.sh`)
/// to exercise a sandbox key against a sandbox organization.
const API: &str = match option_env!("CLOAKVIEW_POLAR_API") {
    Some(base) => base,
    None => "https://api.polar.sh",
};

/// The Polar organization that sells CloakView Pro. The id is not a secret — every
/// licence call carries it — so the production one is compiled in and a release build
/// needs no environment. `CLOAKVIEW_POLAR_ORG_ID` exists only to point a build at a
/// sandbox organization; setting it empty still disables Polar on purpose, and then
/// activation reports `unreachable` instead of pretending the key is bad.
const ORGANIZATION_ID: &str = match option_env!("CLOAKVIEW_POLAR_ORG_ID") {
    Some(id) => id,
    None => "90b662c1-9ae1-4b1b-8299-b2a1efd51419",
};

/// The activation Polar granted this Mac. Only the fields CloakView acts on.
#[derive(Debug, serde::Deserialize)]
struct Activation {
    id: String,
    license_key: PolarKey,
}

/// A Polar license key. Every field is optional: a refusal carries almost nothing.
/// Polar also reports `limit_activations` and `usage`, but it enforces the limit
/// itself (an activation past it is refused, see `refusal`), so they are not read.
#[derive(Debug, Default, serde::Deserialize)]
struct PolarKey {
    #[serde(default)]
    status: String,
    expires_at: Option<String>,
}

impl PolarKey {
    /// Whether this key still entitles the Mac to Pro.
    fn is_live(&self) -> Result<(), Rejection> {
        if self.status != "granted" {
            return Err(match self.status.as_str() {
                "expired" => Rejection::Expired,
                _ => Rejection::Disabled,
            });
        }
        if self.expired() {
            return Err(Rejection::Expired);
        }
        Ok(())
    }

    /// Whether `expires_at` is in the past. A one-time purchase has no expiry, and
    /// an unparseable date is treated as no expiry rather than as a lock-out.
    fn expired(&self) -> bool {
        let Some(raw) = self.expires_at.as_deref() else { return false };
        match chrono::DateTime::parse_from_rfc3339(raw) {
            Ok(at) => at.with_timezone(&chrono::Utc) <= chrono::Utc::now(),
            Err(_) => false,
        }
    }

    /// The activation-limit wording Polar uses on a 4xx body, when it says so.
    fn reports_limit(body: &str) -> bool {
        let body = body.to_ascii_lowercase();
        body.contains("activation") && (body.contains("limit") || body.contains("maximum"))
    }
}

/// Whether this build was given an organization id. An unconfigured build cannot
/// sell, so activation reports that the service is unreachable.
pub fn configured() -> bool {
    !ORGANIZATION_ID.is_empty()
}

/// The organization id this build was given, or `Unreachable` when it has none.
fn organization() -> Result<&'static str, Rejection> {
    if ORGANIZATION_ID.is_empty() {
        return Err(Rejection::Unreachable);
    }
    Ok(ORGANIZATION_ID)
}

/// POSTs `payload` to a customer-portal license endpoint. A 5xx, a timeout, a
/// rate limit or a non-JSON body is `Unreachable`: Polar did not decide anything
/// about the key, so the app keeps whatever it already knew.
async fn post(endpoint: &str, payload: serde_json::Value) -> Result<(u16, String), Rejection> {
    let resp = reqwest::Client::new()
        .post(format!("{API}/v1/customer-portal/license-keys/{endpoint}"))
        .header("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(15))
        .json(&payload)
        .send()
        .await
        .map_err(|_| Rejection::Unreachable)?;
    let status = resp.status().as_u16();
    let body = resp.text().await.map_err(|_| Rejection::Unreachable)?;
    // 429 is the one 4xx that arrives without a verdict: the customer-portal
    // endpoints throttle per address, and three launches in a row from one Mac
    // is enough to trip it. Locking Pro over that would be a lie.
    if status >= 500 || status == 429 || status == 408 {
        return Err(Rejection::Unreachable);
    }
    Ok((status, body))
}

/// A 4xx that is not the documented "no such key" one: the key exists and Polar is
/// refusing it (limit reached, blocked), or the request itself was malformed.
fn refusal(status: u16, body: &str) -> Rejection {
    if PolarKey::reports_limit(body) {
        return Rejection::LimitReached;
    }
    if body.to_ascii_lowercase().contains("\"detail\"") {
        // Polar said why, and it was about the key rather than the network.
        return match status {
            408 | 429 => Rejection::Unreachable,
            400..=499 => Rejection::Invalid,
            _ => Rejection::Unreachable,
        };
    }
    Rejection::Unreachable
}

/// Activates `key` for this Mac and returns the activation id, which the caller
/// keeps as the handle for the launch-time recheck and for releasing the slot.
///
/// A key Polar does not know is rejected as `Invalid`.
pub async fn activate(key: &str, label: &str) -> Result<String, Rejection> {
    let org = organization()?;
    let (status, body) = post(
        "activate",
        serde_json::json!({ "key": key, "organization_id": org, "label": label }),
    )
    .await?;
    match status {
        200..=299 => {
            let act: Activation = serde_json::from_str(&body).map_err(|_| Rejection::Unreachable)?;
            act.license_key.is_live()?;
            Ok(act.id)
        }
        // No such key under this organization: not ours, or refunded away.
        404 => Err(Rejection::Invalid),
        _ => Err(refusal(status, &body)),
    }
}

/// Reads a validation answer: whether `key` still entitles this Mac to Pro.
///
/// A check scoped to an activation answers with it, and `null` there means this
/// Mac's activation is gone — released from another machine, say — so Pro is not
/// ours to keep even though the key itself is alive.
fn validate_answer(body: &str) -> Result<bool, Rejection> {
    let raw: serde_json::Value = serde_json::from_str(body).map_err(|_| Rejection::Unreachable)?;
    let key: PolarKey = serde_json::from_value(raw.clone()).map_err(|_| Rejection::Unreachable)?;
    if raw.get("activation").is_some_and(|a| a.is_null()) {
        return Ok(false);
    }
    Ok(key.is_live().is_ok())
}

/// Asks Polar whether `key` still entitles this Mac to Pro, scoped to the
/// activation so a Mac released elsewhere stops being Pro here too. `Err` only
/// when Polar could not be asked.
pub async fn validate(key: &str, activation_id: &str) -> Result<bool, Rejection> {
    let org = organization()?;
    let (status, body) = post(
        "validate",
        serde_json::json!({ "key": key, "organization_id": org, "activation_id": activation_id }),
    )
    .await?;
    match status {
        200..=299 => validate_answer(&body),
        404 => Ok(false),
        _ => Err(refusal(status, &body)),
    }
}

/// Releases this Mac's activation so the key's slot is free again. A 404 means the
/// key or the activation is already gone: there is nothing left to release.
pub async fn deactivate(key: &str, activation_id: &str) -> Result<(), Rejection> {
    let org = organization()?;
    let (status, body) = post(
        "deactivate",
        serde_json::json!({ "key": key, "organization_id": org, "activation_id": activation_id }),
    )
    .await?;
    match status {
        200..=299 | 404 => Ok(()),
        _ => Err(refusal(status, &body)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(json: serde_json::Value) -> PolarKey {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn a_granted_key_without_expiry_is_live() {
        assert!(key(serde_json::json!({ "status": "granted", "usage": 1, "limit_activations": 5 })).is_live().is_ok());
    }

    #[test]
    fn the_documented_activation_answer_names_the_activation() {
        let act: Activation = serde_json::from_str(
            r#"{
                "id": "act-1",
                "license_key_id": "lk-1",
                "label": "CloakView on Mac",
                "license_key": {
                    "id": "lk-1",
                    "key": "ABCD-1234",
                    "display_key": "****-1234",
                    "status": "granted",
                    "limit_activations": 5,
                    "usage": 1,
                    "expires_at": null
                }
            }"#,
        )
        .unwrap();
        // The fields CloakView does not read (the limit, the usage) are tolerated.
        assert_eq!(act.id, "act-1");
        assert!(act.license_key.is_live().is_ok());
    }

    #[test]
    fn a_revoked_activation_on_the_key_locks_pro() {
        // What a validation scoped to a released activation looks like: the key is
        // alive, this Mac's activation is not.
        assert_eq!(validate_answer(r#"{ "status": "granted", "activation": null }"#), Ok(false));
        assert_eq!(validate_answer(r#"{ "status": "granted", "activation": { "id": "act-1" } }"#), Ok(true));
        // Without the field the key's own state decides (nothing claims otherwise).
        assert_eq!(validate_answer(r#"{ "status": "granted" }"#), Ok(true));
        assert_eq!(validate_answer(r#"{ "status": "disabled" }"#), Ok(false));
    }

    #[test]
    fn a_key_that_is_not_granted_is_refused() {
        assert_eq!(key(serde_json::json!({ "status": "disabled" })).is_live(), Err(Rejection::Disabled));
        assert_eq!(key(serde_json::json!({ "status": "expired" })).is_live(), Err(Rejection::Expired));
    }

    #[test]
    fn an_expiry_in_the_past_is_expired_and_a_future_one_is_not() {
        let past = key(serde_json::json!({ "status": "granted", "expires_at": "2020-01-01T00:00:00Z" }));
        assert_eq!(past.is_live(), Err(Rejection::Expired));
        let future = key(serde_json::json!({ "status": "granted", "expires_at": "2999-01-01T00:00:00Z" }));
        assert!(future.is_live().is_ok());
        // An unparseable date is not treated as an expiry.
        let odd = key(serde_json::json!({ "status": "granted", "expires_at": "soon" }));
        assert!(odd.is_live().is_ok());
    }

    #[test]
    fn a_limit_refusal_is_reported_as_the_limit() {
        let body = r#"{ "detail": "The activation limit has been reached for this license key." }"#;
        assert_eq!(refusal(422, body), Rejection::LimitReached);
        // Another refusal about the key is just an invalid key.
        assert_eq!(refusal(403, r#"{ "detail": "License key is blocked." }"#), Rejection::Invalid);
        // A body without a verdict is not a decision about the key.
        assert_eq!(refusal(400, "<html>oops</html>"), Rejection::Unreachable);
    }

    #[test]
    fn a_rate_limit_is_not_a_verdict_about_the_key() {
        // The customer-portal endpoints throttle per address: three launches in a
        // row from one Mac trips a 429. Reading that as a bad key would drop a
        // paying customer out of Pro, so it has to stay undecided.
        assert_eq!(refusal(429, r#"{ "detail": "Too many requests." }"#), Rejection::Unreachable);
        assert_eq!(refusal(408, r#"{ "detail": "Request timeout." }"#), Rejection::Unreachable);
    }

    #[test]
    fn an_unconfigured_build_reports_unreachable_rather_than_a_bad_key() {
        // The tree ships with no organization id; activation must not tell a buyer
        // their key is invalid when the build simply was not configured.
        if ORGANIZATION_ID.is_empty() {
            assert_eq!(organization(), Err(Rejection::Unreachable));
        } else {
            assert!(organization().is_ok());
        }
    }
}
