//! The provider registry: builtin profiles, per-provider key slots, and
//! resolution to a ready `dyn Provider`. Product rule: the user picks ONE
//! provider and pastes ONE key; everything (listening, answers, summaries,
//! translations, screen reads) goes to that provider.

use crate::error::{AppError, AppResult};
use crate::secrets;
use crate::state::{ModelOverride, Settings};
use cv_providers::{openai_compat::OpenAiCompat, profiles::ProviderProfile, SttMode};
use std::sync::Arc;

pub use cv_providers::profiles::builtin_profiles;

/// Launch gate (Nov 20, 2026): the dropdown lists ONLY providers verified
/// end-to-end with a real key. Phase 2 (batch STT for Groq/Together) is out
/// of scope for launch: those profiles stay compiled and unit-tested in the
/// crate, hidden here until a real-call test passes (the Nov 18 gate).
const LAUNCH_PROVIDER_IDS: &[&str] = &["openai"];

/// Every profile the user can pick — the launch set only (see
/// `LAUNCH_PROVIDER_IDS`). Each listed model name was verified against the
/// vendor's docs (see `ProviderProfile::models_doc_url`).
pub fn profiles() -> Vec<ProviderProfile> {
    builtin_profiles()
        .into_iter()
        .filter(|p| LAUNCH_PROVIDER_IDS.contains(&p.id.as_str()))
        .collect()
}

pub fn find_profile(id: &str) -> Option<ProviderProfile> {
    profiles().into_iter().find(|p| p.id == id)
}

/// The profile `id` names with `model_overrides` applied — what the app
/// actually calls, and what `list_profiles` shows. Unknown ids fall back to
/// the openai profile.
pub fn effective_profile(settings: &Settings, id: &str) -> ProviderProfile {
    let profile = find_profile(id).unwrap_or_else(|| find_profile("openai").expect("openai profile"));
    with_overrides(profile, settings.model_overrides.get(id))
}

/// The profile `Settings::active_provider` names; unknown values fall back to
/// "openai", the same rule other free-form settings fields use.
pub fn active_profile(settings: &Settings) -> ProviderProfile {
    effective_profile(settings, &settings.active_provider)
}

/// The profile with `model_overrides` applied. The override map is a safety
/// valve kept in the settings file (not the UI): a retired default model can
/// be replaced without waiting for a release.
fn with_overrides(mut profile: ProviderProfile, overrides: Option<&ModelOverride>) -> ProviderProfile {
    if let Some(overrides) = overrides {
        if let Some(stt) = overrides.stt.as_ref().filter(|m| !m.trim().is_empty()) {
            profile.stt_model = Some(stt.trim().to_string());
        }
        if let Some(llm) = overrides.llm.as_ref().filter(|m| !m.trim().is_empty()) {
            profile.llm_model = llm.trim().to_string();
        }
    }
    profile
}

fn resolve(profile: ProviderProfile, settings: &Settings) -> AppResult<Arc<dyn cv_providers::Provider>> {
    let overrides = settings.model_overrides.get(&profile.id);
    let profile = with_overrides(profile, overrides);
    let key = secrets::get_key(&profile.id)?;
    Ok(Arc::new(OpenAiCompat::new(profile, key)))
}/// The provider for `Settings::active_provider`, ready to answer and
/// (depending on its mode) transcribe.
pub fn resolve_active(settings: &Settings) -> AppResult<Arc<dyn cv_providers::Provider>> {
    resolve(active_profile(settings), settings)
}

/// The provider named by `id` (for the Test-connection button).
pub fn resolve_by_id(id: &str, settings: &Settings) -> AppResult<Arc<dyn cv_providers::Provider>> {
    let profile = find_profile(id).ok_or_else(|| AppError::UnknownProvider(id.into()))?;
    resolve(profile, settings)
}

/// Validates that listening can run with the active provider and returns its
/// key. Phase 1: listening is the OpenAI Realtime path only; a batch-only
/// provider is refused here until Phase 2 wires the segment pipeline.
pub fn active_streaming_key(settings: &Settings) -> AppResult<String> {
    streaming_key_for(active_profile(settings))
}

/// The mode check behind `active_streaming_key`, keyed off one concrete
/// profile so the batch-refusal branch stays unit-testable while batch
/// providers are hidden from the dropdown.
fn streaming_key_for(profile: ProviderProfile) -> AppResult<String> {
    let mode = profile.stt_model.as_ref().map(|_| profile.stt_mode);
    match mode {
        Some(SttMode::Streaming) => secrets::get_key(&profile.id),
        Some(SttMode::Batch) => Err(AppError::SttUnsupported(profile.display_name)),
        None => Err(AppError::SttUnsupported(profile.display_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_profile_falls_back_to_openai_for_unknown_ids() {
        let mut s = Settings::default();
        assert_eq!(active_profile(&s).id, "openai");
        s.active_provider = "nonsense".into();
        assert_eq!(active_profile(&s).id, "openai");
    }

    #[test]
    fn overrides_replace_models_and_blank_overrides_do_not() {
        let p = builtin_profiles().into_iter().find(|p| p.id == "groq").unwrap();
        let applied = with_overrides(p.clone(), Some(&ModelOverride {
            stt: Some(" whisper-large-v3 ".into()),
            llm: None,
        }));
        assert_eq!(applied.stt_model.as_deref(), Some("whisper-large-v3"));
        assert_eq!(applied.llm_model, p.llm_model);

        let blank = with_overrides(p.clone(), Some(&ModelOverride { stt: Some("  ".into()), llm: Some("".into()) }));
        assert_eq!(blank.stt_model, p.stt_model);
        assert_eq!(blank.llm_model, p.llm_model);

        let none = with_overrides(p.clone(), None);
        assert_eq!(none.stt_model, p.stt_model);
        assert_eq!(none.llm_model, p.llm_model);
    }

    #[test]
    fn unknown_provider_id_is_a_typed_error() {
        let s = Settings::default();
        assert!(matches!(resolve_by_id("nope", &s), Err(AppError::UnknownProvider(_))));
    }

    #[test]
    fn streaming_key_refuses_batch_only_providers() {
        let _guard = crate::secrets::TEST_LOCK.lock().unwrap();
        keyring::set_default_credential_builder(keyring::mock::default_credential_builder());
        let s = Settings { active_provider: "openai".into(), ..Default::default() };
        // No key saved: still the streaming/openai decision, then key error.
        let _ = secrets::clear_key("openai");
        assert!(matches!(active_streaming_key(&s), Err(AppError::ProviderKeyMissing(_))));

        // The batch-refusal branch, against the crate's ungated profiles:
        // what a hand-edited settings file naming Groq would hit once the
        // dropdown ungates it (Phase 2). While the launch gate hides Groq,
        // `active_provider = "groq"` falls back to openai instead (see the
        // launch-gate test below).
        let groq = builtin_profiles().into_iter().find(|p| p.id == "groq").unwrap();
        assert!(matches!(streaming_key_for(groq), Err(AppError::SttUnsupported(_))));
    }

    #[test]
    fn launch_gate_lists_only_openai_and_gated_ids_fall_back() {
        let ids: Vec<String> = profiles().into_iter().map(|p| p.id).collect();
        assert_eq!(ids, vec!["openai"], "launch dropdown must show only verified providers");
        assert!(find_profile("groq").is_none());
        assert!(find_profile("together").is_none());

        // A settings file naming a gated provider behaves like an unknown
        // one: the openai fallback, never a half-configured other provider.
        let s = Settings { active_provider: "groq".into(), ..Default::default() };
        assert_eq!(active_profile(&s).id, "openai");
        assert!(matches!(resolve_by_id("groq", &s), Err(AppError::UnknownProvider(_))));
    }
}
