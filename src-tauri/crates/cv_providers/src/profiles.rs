use serde::{Deserialize, Serialize};

use crate::SttMode;

/// Which request/response dialect one vendor speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiFlavor {
    /// OpenAI's Responses API (`/v1/responses`): `instructions` field,
    /// `input_text`/`input_image` parts, `output_text` SSE deltas.
    Responses,
    /// The OpenAI-compatible Chat Completions API (`/v1/chat/completions`),
    /// also spoken by Groq, Together, DeepSeek, OpenRouter, ...
    Chat,
}

/// Everything needed to talk to one vendor with ONE key.
#[derive(Debug, Clone, Serialize)]
pub struct ProviderProfile {
    pub id: String,           // "openai", "groq", "together", ...
    pub display_name: String,
    pub base_url: String,     // OpenAI-compatible root, e.g. https://api.openai.com/v1
    pub flavor: ApiFlavor,
    /// How the app transcribes with this provider. `Streaming` keeps the
    /// realtime WebSocket path (OpenAI only, today); `Batch` routes VAD-cut
    /// segments through `transcribe`. `stt_model` being None (no STT at
    /// all) is a profile the UI must refuse to activate.
    pub stt_mode: SttMode,
    pub stt_model: Option<String>, // None => provider cannot transcribe
    pub llm_model: String,
    /// Which STT models the provider offers for the streaming listen path.
    /// Empty for providers that cannot stream (Batch) or have no STT at all.
    /// The first entry is the default.
    pub stt_model_options: &'static [&'static str],
    /// Documented source for the model names above, for the launch audit.
    pub models_doc_url: &'static str,
}

/// The launch set. Every model name was checked against the vendor's
/// official docs on 2026-11-14 (sources in `models_doc_url`); the crate's
/// callers cite these in the launch report. Defaults only: an app-side
/// `model_overrides` setting can replace either model without a release.
pub fn builtin_profiles() -> Vec<ProviderProfile> {
    vec![
        ProviderProfile {
            id: "openai".into(),
            display_name: "OpenAI".into(),
            base_url: "https://api.openai.com/v1".into(),
            flavor: ApiFlavor::Responses,
            stt_mode: SttMode::Streaming,
            // Batch model: the app streams STT through the Realtime API with
            // the same model (gpt-4o-mini-transcribe), which the shipped
            // 0.2.1 build already uses. The batch name is what transcribe()
            // and the connection test use.
            stt_model: Some("gpt-4o-mini-transcribe".into()),
            llm_model: "gpt-4o-mini".into(),
            stt_model_options: &[
                "gpt-4o-mini-transcribe",
                "gpt-4o-transcribe",
            ],
            models_doc_url: "https://platform.openai.com/docs/models".into(),
        },
        ProviderProfile {
            id: "groq".into(),
            display_name: "Groq".into(),
            base_url: "https://api.groq.com/openai/v1".into(),
            flavor: ApiFlavor::Chat,
            stt_mode: SttMode::Batch,
            stt_model: Some("whisper-large-v3-turbo".into()),
            // llama-3.3-70b-versatile was retired 2026-08-16; gpt-oss-120b is
            // in the current production table.
            llm_model: "openai/gpt-oss-120b".into(),
            stt_model_options: &[],
            models_doc_url: "https://console.groq.com/docs/models".into(),
        },
        ProviderProfile {
            id: "together".into(),
            display_name: "Together AI".into(),
            base_url: "https://api.together.xyz/v1".into(),
            flavor: ApiFlavor::Chat,
            stt_mode: SttMode::Batch,
            stt_model: Some("openai/whisper-large-v3".into()),
            // Llama-3.3-70B left the serverless catalog; gpt-oss-120b is in
            // the current chat table.
            llm_model: "openai/gpt-oss-120b".into(),
            stt_model_options: &[],
            models_doc_url: "https://docs.together.ai/docs/serverless-models".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        let profiles = builtin_profiles();
        let mut ids: Vec<&str> = profiles.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), profiles.len(), "duplicate profile ids");
    }

    #[test]
    fn every_launch_profile_transcribes_and_speaks_chat_or_responses() {
        for p in builtin_profiles() {
            assert!(p.stt_model.is_some(), "{} lost its STT model", p.id);
            assert!(!p.llm_model.is_empty());
            assert!(!p.models_doc_url.is_empty(), "{} lacks its doc citation", p.id);
        }
    }

    #[test]
    fn openai_profile_lists_only_official_realtime_transcription_models() {
        let openai = builtin_profiles()
            .into_iter()
            .find(|p| p.id == "openai")
            .expect("openai profile exists");
        assert_eq!(
            openai.stt_model_options,
            &["gpt-4o-mini-transcribe", "gpt-4o-transcribe"]
        );
        assert_eq!(openai.stt_model_options[0], "gpt-4o-mini-transcribe");
    }

    #[test]
    fn non_streaming_providers_have_empty_stt_model_options() {
        for p in builtin_profiles() {
            if p.stt_mode == SttMode::Batch {
                assert!(p.stt_model_options.is_empty(), "{} is batch and should not offer streaming STT model choices", p.id);
            }
        }
    }
}
