//! Provider failures mapped to fixed categories with our own text, plus a
//! pattern scrubber as a second defense.
//!
//! Why: some vendors echo a masked fragment of the API key inside 401 error
//! bodies. A raw provider body must never reach the UI (the IPC payload) or
//! the diagnostics log. First defense: classify the failure and render our
//! own message — the body is inspected for classification but never copied.
//! Second defense: scrub anything that still matches a common key pattern
//! before a string crosses either boundary.

/// The fixed failure categories. The message text is ours, never the
/// provider's body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// 401/403: the key was refused.
    InvalidKey,
    /// 429 or an explicit quota error.
    RateLimited,
    /// 404 or a body naming a missing/retired model.
    ModelNotFound,
    /// Could not reach the vendor at all.
    Network,
    /// Everything else, reported with the HTTP status only.
    Other,
}

impl FailureKind {
    /// Our own message for the category, with the HTTP status when there is
    /// one. Never includes the response body.
    pub fn message(self, status: Option<u16>) -> String {
        let status = status.map(|s| format!(" (HTTP {s})")).unwrap_or_default();
        match self {
            FailureKind::InvalidKey => format!("the API key was rejected{status}"),
            FailureKind::RateLimited => format!("rate limited or out of quota{status}"),
            FailureKind::ModelNotFound => format!("the model was not found or has been retired{status}"),
            FailureKind::Network => "could not reach the provider (network error)".to_string(),
            FailureKind::Other => format!("the provider request failed{status}"),
        }
    }
}

/// Classifies an HTTP failure: the status decides, the body only refines the
/// model-not-found case (a retired model often comes back as 400/4xx with a
/// body saying so). `body` is read here and never returned.
pub fn classify(status: u16, body: &str) -> FailureKind {
    match status {
        401 | 403 => FailureKind::InvalidKey,
        429 => FailureKind::RateLimited,
        404 => FailureKind::ModelNotFound,
        _ => {
            let lower = body.to_ascii_lowercase();
            let names_a_model_problem = lower.contains("model_not_found")
                || (lower.contains("model")
                    && (lower.contains("not found")
                        || lower.contains("does not exist")
                        || lower.contains("has been retired")
                        || lower.contains("decommissioned")));
            if names_a_model_problem {
                FailureKind::ModelNotFound
            } else {
                FailureKind::Other
            }
        }
    }
}

/// Replaces anything that looks like an API-key fragment with `[redacted]`.
///
/// Two rules:
///
/// 1. Known key prefixes — `sk-...` (OpenAI/Together, incl. `sk-proj-`,
///    `sk-ant-`), `gsk_...`/`gsk-...` (Groq): the prefix followed by at
///    least 8 letters/digits/`_`/`-` with at least one digit, **and** a
///    word boundary before the prefix (so `desk-1234` or `task-list` are
///    untouched). Real keys are base62-ish and always contain digits.
/// 2. Long opaque tokens — any run of ≥ 32 letters/digits/`_`/`-` with both
///    a letter and a digit catches key-shaped strings even without a known
///    prefix, while leaving model names (`whisper-large-v3-turbo`,
///    `gpt-4o-mini-transcribe`) and URLs readable — they never reach 32.
///
/// The output of this function only ever appears in error surfaces, so a
/// rare false positive mangles an error string, never user data.
pub fn scrub_secrets(input: &str) -> String {
    const PREFIXES: &[&str] = &["sk-", "gsk_", "gsk-"];
    const MIN_PREFIX_RUN: usize = 8;
    const MIN_OPAQUE_RUN: usize = 32;

    let is_run_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let rest = &input[i..];
        // Rule 1: a known prefix on a word boundary.
        let on_boundary = i == 0 || !is_run_byte(bytes[i - 1]);
        if on_boundary {
            let prefix = PREFIXES.iter().find(|p| rest.starts_with(**p));
            if let Some(prefix) = prefix {
                let after = &rest[prefix.len()..];
                let run_len = after.bytes().take_while(|&b| is_run_byte(b)).count();
                let run_has_digit = after.bytes().take(run_len).any(|b| b.is_ascii_digit());
                if run_len >= MIN_PREFIX_RUN && run_has_digit {
                    out.push_str("[redacted]");
                    i += prefix.len() + run_len;
                    continue;
                }
            }
        }
        // Rule 2: a long opaque token starting here.
        let run_len = rest.bytes().take_while(|&b| is_run_byte(b)).count();
        if run_len >= MIN_OPAQUE_RUN {
            let run = &rest[..run_len];
            let has_digit = run.bytes().any(|b| b.is_ascii_digit());
            let has_letter = run.bytes().any(|b| b.is_ascii_alphabetic());
            if has_digit && has_letter {
                out.push_str("[redacted]");
                i += run_len;
                continue;
            }
        }
        let ch = rest.chars().next().expect("i < len");
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_by_status_first() {
        assert_eq!(classify(401, ""), FailureKind::InvalidKey);
        assert_eq!(classify(403, ""), FailureKind::InvalidKey);
        assert_eq!(classify(429, ""), FailureKind::RateLimited);
        assert_eq!(classify(404, ""), FailureKind::ModelNotFound);
        assert_eq!(classify(500, ""), FailureKind::Other);
        assert_eq!(classify(400, ""), FailureKind::Other);
    }

    #[test]
    fn body_names_a_retired_model_even_without_404() {
        assert_eq!(
            classify(400, r#"{"error":{"message":"The model `whisper-x` has been retired"}}"#),
            FailureKind::ModelNotFound
        );
        assert_eq!(
            classify(422, r#"{"error":{"code":"model_not_found"}}"#),
            FailureKind::ModelNotFound
        );
        // A body that does not name a model problem stays "other".
        assert_eq!(classify(500, r#"{"error":{"message":"internal"}}"#), FailureKind::Other);
    }

    #[test]
    fn messages_are_ours_and_carry_the_status_never_a_body() {
        assert_eq!(FailureKind::InvalidKey.message(Some(401)), "the API key was rejected (HTTP 401)");
        assert_eq!(FailureKind::RateLimited.message(Some(429)), "rate limited or out of quota (HTTP 429)");
        assert!(FailureKind::ModelNotFound.message(Some(404)).contains("404"));
        assert_eq!(FailureKind::Network.message(None), "could not reach the provider (network error)");
        assert_eq!(FailureKind::Other.message(None), "the provider request failed");
    }

    #[test]
    fn scrubs_common_key_shapes() {
        assert_eq!(scrub_secrets("key sk-AbCdEf1234GhIjKl5678xyz end"), "key [redacted] end");
        assert_eq!(scrub_secrets("key=sk-AbCdEf1234GhIjKl5678xyz."), "key=[redacted].");
        assert_eq!(scrub_secrets("sk-proj-AAAAbbbbCCCC1111"), "[redacted]");
        assert_eq!(scrub_secrets("sk-ant-api03-AAAAbbbbCCCC1111"), "[redacted]");
        assert_eq!(scrub_secrets("gsk_AAAAbbbbCCCC1111dddd"), "[redacted]");
        assert_eq!(scrub_secrets("gsk-AAAAbbbbCCCC1111"), "[redacted]");
        // A key in a URL query is still a key.
        assert_eq!(
            scrub_secrets("https://proxy.test/?api_key=sk-AbCdEf1234GhIjKl5678&x=1"),
            "https://proxy.test/?api_key=[redacted]&x=1"
        );
        // A long opaque token with no known prefix is still a key shape.
        let opaque = "AAAAbbbbCCCC1111ddddEEEEffff2222GGGG";
        assert_eq!(scrub_secrets(&format!("token {opaque} done")), "token [redacted] done");
    }

    /// The other direction, from the review: model names and URLs in error
    /// strings must stay readable, and prefix-lookalikes inside ordinary
    /// words must not be eaten.
    #[test]
    fn keeps_model_names_urls_and_ordinary_words_readable() {
        let realistic = [
            // Websocket/endpoint errors from the realtime path.
            "wss://api.openai.com/v1/realtime?model=gpt-4o-mini-transcribe",
            "HTTP error: 401 Unauthorized for https://api.openai.com/v1/realtime",
            "wss://api.groq.com/openai/v1/realtime",
            // Batch STT endpoints and model names.
            "POST https://api.groq.com/openai/v1/audio/transcriptions (whisper-large-v3-turbo): 413",
            "model openai/gpt-oss-120b does not exist",
            "The model `gpt-4o-mini-realtime` has been retired",
            "unknown model gpt-4o-mini-transcribe",
            // Prefix-lookalikes inside words: boundary rule keeps them.
            "moved to desk-1234",
            "task-list and risk-levels5 ok",
            "logskey_12345678 unchanged",
            // Model-name length is far below the opaque-token threshold.
            "whisper-large-v3-turbo",
        ];
        for s in realistic {
            assert_eq!(scrub_secrets(s), s, "over-redacted: {s}");
        }
    }

    #[test]
    fn scrub_leaves_plain_prose_and_short_runs_alone() {
        assert_eq!(scrub_secrets("ask-the-user"), "ask-the-user");
        assert_eq!(scrub_secrets("no key here"), "no key here");
        // A prefix with too short a run is not a key.
        assert_eq!(scrub_secrets("error: sk-xyz"), "error: sk-xyz");
    }

    #[test]
    fn scrub_handles_non_ascii_and_empty() {
        assert_eq!(scrub_secrets(""), "");
        assert_eq!(scrub_secrets("opiniones sk-AbCdEf1234GhIjKl5678 ¿tú?"), "opiniones [redacted] ¿tú?");
    }
}
