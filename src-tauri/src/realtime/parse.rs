use crate::realtime::RealtimeEvent;

/// Parses one realtime WS frame into an event, or `None` for frame types
/// this app doesn't act on. Pure — no I/O, unit-tested directly.
pub fn parse_frame(json: &str) -> Option<RealtimeEvent> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let t = v.get("type")?.as_str()?;
    match t {
        "session.started" => Some(RealtimeEvent::SessionStarted),
        "conversation.item.input_audio_transcription.completed" => {
            // The API puts the text in `transcript`. The nested `item.content`
            // form is what this parser first assumed; it stays as a fallback.
            let text = match v.get("transcript").and_then(|t| t.as_str()) {
                Some(transcript) => transcript.to_string(),
                None => v
                    .get("item")?
                    .get("content")?
                    .as_array()?
                    .iter()
                    .filter_map(|c| c.get("text")?.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            };
            if text.trim().is_empty() {
                None
            } else {
                Some(RealtimeEvent::Transcript {
                    speaker: "remote".into(), // speaker attribution is heuristic in v1
                    text: text.trim().into(),
                    is_user_turn: false,
                })
            }
        }
        // The provider's error text goes to the UI (`error` event) and the
        // diagnostics log: scrub it like every other boundary crossing.
        "error" => Some(RealtimeEvent::Error {
            message: cv_providers::redact::scrub_secrets(
                v.get("error")?.get("message")?.as_str()?,
            ),
        }),
        _ => None,
    }
}

/// How sure the model was of a completed transcript: the mean log-probability
/// of its tokens (0 is certain; more negative is less sure) and how many tokens
/// there were. `None` when the frame carries no `logprobs` (the session did not
/// ask for them, or the model does not report them). Pure.
pub fn confidence(json: &str) -> Option<(f64, usize)> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    if v.get("type")?.as_str()? != "conversation.item.input_audio_transcription.completed" {
        return None;
    }
    let probs: Vec<f64> = v.get("logprobs")?.as_array()?.iter().filter_map(|t| t.get("logprob")?.as_f64()).collect();
    if probs.is_empty() {
        return None;
    }
    Some((probs.iter().sum::<f64>() / probs.len() as f64, probs.len()))
}

/// Below this mean log-probability a line is a caption the model invented from
/// music, sound effects or noise, not something anybody said. Measured on a
/// video-like track: invented lines came back around -3.0, clear speech near
/// 0.0, and speech garbled by a music bed around -1.3 (kept).
const INVENTED_BELOW: f64 = -2.0;
/// Short lines need to be surer: invented captions are usually a few tokens.
const SHORT_TOKENS: usize = 6;
const SHORT_INVENTED_BELOW: f64 = -1.2;

/// What these models classically "hear" in music, applause or silence. Compared
/// whole, after lowercasing and dropping punctuation, so a real sentence that
/// merely contains "thank you" is never touched.
const INVENTED_PHRASES: &[&str] = &[
    "thank you",
    "thank you very much",
    "thanks for watching",
    "thank you for watching",
    "thank you so much for watching",
    "please subscribe",
    "like and subscribe",
    "bye",
    "you",
    "gracias",
    "gracias por ver",
    "gracias por ver el video",
    "suscríbete",
    "subtítulos realizados por la comunidad de amara org",
    "amara org",
];

fn normalized(text: &str) -> String {
    let kept: String = text.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
    kept.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether a transcribed line is most likely invented from non-speech audio
/// (a video's music, sound effects, a noisy room) rather than words someone said:
/// the model was unsure of it, or it is one of the classic phrases these models
/// produce from noise. Such lines are dropped before they reach the transcript,
/// so they never become a "question" to answer either.
pub fn is_probably_invented(text: &str, confidence: Option<(f64, usize)>) -> bool {
    if let Some((mean, tokens)) = confidence {
        if mean < INVENTED_BELOW || (tokens <= SHORT_TOKENS && mean < SHORT_INVENTED_BELOW) {
            return true;
        }
    }
    let line = normalized(text);
    line.is_empty() || INVENTED_PHRASES.contains(&line.as_str())
}

#[cfg(test)]
mod invented_tests {
    use super::is_probably_invented;

    #[test]
    fn captions_invented_from_music_are_dropped() {
        // The two lines a video-like track produced, with their measured confidence.
        assert!(is_probably_invented("and he died there.", Some((-2.95, 5))));
        assert!(is_probably_invented("of the town.", Some((-3.03, 4))));
        // A short line the model was unsure of.
        assert!(is_probably_invented("I see you.", Some((-1.5, 4))));
    }

    #[test]
    fn real_speech_is_kept_even_when_the_model_struggled() {
        assert!(!is_probably_invented("Thanks for joining us today. So, tell me about yourself.", Some((-0.02, 24))));
        // Speech over a music bed: garbled, but real.
        assert!(!is_probably_invented("Do you work at this company?", Some((-1.26, 7))));
        assert!(!is_probably_invented("Yes.", Some((-0.1, 2))));
        // Without confidence (a model that reports none), only the phrase list applies.
        assert!(!is_probably_invented("Why this company?", None));
    }

    #[test]
    fn the_classic_noise_phrases_are_dropped_whole_only() {
        assert!(is_probably_invented("Thank you.", None));
        assert!(is_probably_invented("Thanks for watching!", Some((-0.3, 4))));
        assert!(is_probably_invented("¡Gracias por ver el video!", None));
        assert!(is_probably_invented("...", None));
        // A real sentence that contains one of them is untouched.
        assert!(!is_probably_invented("Thank you, and what is your notice period?", None));
    }
}

#[cfg(test)]
mod confidence_tests {
    use super::confidence;

    #[test]
    fn the_mean_logprob_of_a_completed_transcript() {
        let frame = r#"{"type":"conversation.item.input_audio_transcription.completed","transcript":"Hi there",
            "logprobs":[{"token":"Hi","logprob":-0.1,"bytes":[72,105]},{"token":" there","logprob":-0.3,"bytes":[]}]}"#;
        let (mean, n) = confidence(frame).unwrap();
        assert!((mean - -0.2).abs() < 1e-9);
        assert_eq!(n, 2);
        // No logprobs asked for, or another frame type: nothing to say.
        assert!(confidence(r#"{"type":"conversation.item.input_audio_transcription.completed","transcript":"x"}"#).is_none());
        assert!(confidence(r#"{"type":"error","logprobs":[{"logprob":-1.0}]}"#).is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_completed_transcript() {
        let frame = r#"{"type":"conversation.item.input_audio_transcription.completed",
            "item":{"content":[{"type":"input_text","text":"Hola, ¿me escuchan?"}]}}"#;
        match parse_frame(frame) {
            Some(RealtimeEvent::Transcript { text, .. }) => assert_eq!(text, "Hola, ¿me escuchan?"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parses_the_transcript_field_of_a_real_completed_event() {
        let frame = r#"{"type":"conversation.item.input_audio_transcription.completed",
            "item_id":"item_003","content_index":0,"transcript":" Hello, how are you? "}"#;
        match parse_frame(frame) {
            Some(RealtimeEvent::Transcript { text, .. }) => assert_eq!(text, "Hello, how are you?"),
            other => panic!("unexpected {other:?}"),
        }
        let empty = r#"{"type":"conversation.item.input_audio_transcription.completed","transcript":"  "}"#;
        assert!(parse_frame(empty).is_none());
    }

    #[test]
    fn malformed_frames_yield_none_or_error_event() {
        assert!(parse_frame("not json").is_none());
        assert!(parse_frame("{}").is_none());
        let err = r#"{"type":"error","error":{"message":"rate_limit"}}"#;
        assert!(matches!(parse_frame(err), Some(RealtimeEvent::Error { .. })));
    }

    /// `TurnBoundary` isn't a frame type the realtime API sends, and this
    /// deliberately never produces one — that's fine, not a gap: turn
    /// detection is a heuristic over each `Transcript`'s *text*
    /// (`detect::is_user_turn`, called from `listen::pump`), not something
    /// the server signals. This test just locks in that known fact so a
    /// future change here doesn't silently surprise whoever's touching
    /// `pump` (its module doc explains the real trigger).
    #[test]
    fn never_produces_turn_boundary_but_that_is_fine() {
        let frames = [
            r#"{"type":"session.started"}"#,
            r#"{"type":"conversation.item.input_audio_transcription.completed",
                "item":{"content":[{"type":"input_text","text":"hola"}]}}"#,
            r#"{"type":"error","error":{"message":"boom"}}"#,
            r#"{"type":"something.unhandled"}"#,
            "not json",
            "{}",
        ];
        for frame in frames {
            assert!(
                !matches!(parse_frame(frame), Some(RealtimeEvent::TurnBoundary)),
                "parse_frame produced TurnBoundary for {frame}; update listen.rs's module doc \
                 if this is now the real turn-detected trigger"
            );
        }
    }
}
