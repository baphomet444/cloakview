//! OS-facing helpers behind `get_mic_status` and `open_external`. The pure
//! parts (status mapping, URL validation) are unit-tested; the calls that
//! touch the OS are thin wrappers around them.
//!
//! The app targets macOS only (see the plan's Global Constraints), so these
//! use macOS APIs directly.

use crate::error::{AppError, AppResult};
use objc2_av_foundation::{AVCaptureDevice, AVMediaTypeAudio};
use serde::Serialize;

/// Microphone permission as the frontend sees it. Serializes as a plain
/// camelCase string (`"notDetermined"`, `"authorized"`, ...), not a tagged
/// object — it carries no data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MicStatus {
    NotDetermined,
    Authorized,
    Denied,
    Restricted,
}

impl MicStatus {
    /// Maps a raw `AVAuthorizationStatus`: 0 notDetermined, 1 restricted,
    /// 2 denied, 3 authorized. A value Apple adds later maps to
    /// `NotDetermined` — never to `Authorized` — so an unknown status can't
    /// make the UI believe the mic is usable.
    fn from_raw(raw: isize) -> Self {
        match raw {
            1 => MicStatus::Restricted,
            2 => MicStatus::Denied,
            3 => MicStatus::Authorized,
            _ => MicStatus::NotDetermined,
        }
    }
}

/// Asks macOS for microphone access: the first time this shows the system prompt.
/// It does not wait for the answer; the caller reports the permission as missing and
/// the user presses Listen again. Does nothing once the choice has been made.
pub fn request_mic_access() {
    // SAFETY: `AVMediaTypeAudio` is an immutable constant exported by the AVFoundation framework.
    let Some(audio) = (unsafe { AVMediaTypeAudio }) else { return };
    let on_answer = block2::RcBlock::new(|_granted: objc2::runtime::Bool| {});
    // SAFETY: a plain class method taking the audio media type and a completion block,
    // which macOS may call from any thread; the block captures nothing.
    unsafe { AVCaptureDevice::requestAccessForMediaType_completionHandler(audio, &on_answer) };
}

/// Current microphone permission, read from the OS **without opening a
/// capture stream and without prompting**: `authorizationStatusForMediaType:`
/// only reports; prompting is a separate `requestAccess…` call this app
/// never makes from here.
pub fn mic_status() -> MicStatus {
    // SAFETY: `AVMediaTypeAudio` is an immutable constant exported by the
    // AVFoundation framework, which the crate links.
    let Some(audio) = (unsafe { AVMediaTypeAudio }) else {
        return MicStatus::NotDetermined;
    };
    // SAFETY: a plain class method taking the audio media type (one of the
    // two values it accepts); it neither prompts nor starts a session.
    let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(audio) };
    MicStatus::from_raw(status.0)
}

/// Accepts only absolute `http`/`https` URLs, returning the **normalized**
/// URL — the caller must open that value, so what is opened is exactly what
/// was validated (the parser lowercases the scheme and host and drops stray
/// tabs/newlines). Everything else — `file:`, `javascript:`, `mailto:`, app
/// URL schemes, relative or malformed input — is `AppError::InvalidUrl`.
pub fn validate_external_url(raw: &str) -> AppResult<url::Url> {
    let parsed = url::Url::parse(raw).map_err(|e| AppError::InvalidUrl(e.to_string()))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        other => Err(AppError::InvalidUrl(format!(
            "scheme {other:?} is not allowed (only http and https)"
        ))),
    }
}

/// Opens `raw` in the user's default browser through macOS `open(1)` after
/// `validate_external_url`. The URL is passed as a single argument (no shell
/// is involved), and since validation guarantees it starts with `http`, it
/// can't be read as an option. Waits for `open` itself (which returns as soon
/// as LaunchServices has the request), not for the browser.
pub async fn open_external(raw: &str) -> AppResult<()> {
    let url = validate_external_url(raw)?;
    let status = tokio::process::Command::new("open")
        .arg(url.as_str())
        .status()
        .await
        .map_err(|e| AppError::OpenFailed(e.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::OpenFailed(format!("`open` exited with {status}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mic_status_maps_av_authorization_status_values() {
        // AVAuthorizationStatus: 0 notDetermined, 1 restricted, 2 denied, 3 authorized.
        assert_eq!(MicStatus::from_raw(0), MicStatus::NotDetermined);
        assert_eq!(MicStatus::from_raw(1), MicStatus::Restricted);
        assert_eq!(MicStatus::from_raw(2), MicStatus::Denied);
        assert_eq!(MicStatus::from_raw(3), MicStatus::Authorized);
        // Unknown / future values are never treated as granted.
        assert_eq!(MicStatus::from_raw(99), MicStatus::NotDetermined);
        assert_eq!(MicStatus::from_raw(-1), MicStatus::NotDetermined);
    }

    #[test]
    fn mic_status_serializes_as_plain_camel_case_strings() {
        for (status, json) in [
            (MicStatus::NotDetermined, "notDetermined"),
            (MicStatus::Authorized, "authorized"),
            (MicStatus::Denied, "denied"),
            (MicStatus::Restricted, "restricted"),
        ] {
            assert_eq!(serde_json::to_value(status).unwrap(), serde_json::json!(json));
        }
    }

    /// Calls the real AVFoundation query. It must link, return a valid
    /// variant, and — the point of "sin abrir stream" — neither prompt nor
    /// hang (a prompt would block this test until someone clicked it).
    #[test]
    fn mic_status_reads_the_real_os_state_without_prompting() {
        let status = mic_status();
        assert!(matches!(
            status,
            MicStatus::NotDetermined | MicStatus::Authorized | MicStatus::Denied | MicStatus::Restricted
        ));
    }

    #[test]
    fn accepts_http_and_https_and_returns_the_normalized_url() {
        assert_eq!(
            validate_external_url("https://example.com/a?b=1#c").unwrap().as_str(),
            "https://example.com/a?b=1#c"
        );
        assert!(validate_external_url("http://localhost:3000").is_ok());
        // Scheme and host are case-insensitive; what comes back is what gets opened.
        assert_eq!(
            validate_external_url("HTTPS://Example.COM").unwrap().as_str(),
            "https://example.com/"
        );
        // Surrounding whitespace is ignored.
        assert!(validate_external_url("  https://example.com  ").is_ok());
    }

    #[test]
    fn rejects_every_other_scheme_and_anything_that_is_not_an_absolute_url() {
        for bad in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "mailto:a@b.co",
            "ftp://example.com",
            "data:text/html,<script>1</script>",
            "x-apple.systempreferences:com.apple.preference.security",
            "tel:+123",
            "ssh://host",
            "",
            "   ",
            "not a url",
            "example.com",
            "//example.com",
            "https:",
            "http://",
        ] {
            assert!(
                matches!(validate_external_url(bad), Err(AppError::InvalidUrl(_))),
                "{bad:?} must be rejected as InvalidUrl"
            );
        }
    }

    /// `open_external` must refuse before spawning anything: a rejected URL
    /// has to come back as `InvalidUrl`, not as an `open` launch failure.
    #[tokio::test]
    async fn open_external_rejects_a_bad_scheme_before_launching_anything() {
        let err = open_external("file:///etc/passwd").await.unwrap_err();
        assert!(matches!(err, AppError::InvalidUrl(_)), "got {err:?}");
    }
}
