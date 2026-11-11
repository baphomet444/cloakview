/// Whether the current transcript line is the user's turn to speak.
///
/// - `speaker_is_user` (the audio channel itself) wins outright.
/// - Otherwise: a case-insensitive substring match against any of
///   `user_keywords`, or against `user_name` when one is configured —
///   the name is just one more keyword, per the spec ("keywords
///   configurables ('¿tú qué opinas?', nombre del usuario)").
///
/// Empty keywords and an absent or blank `user_name` never match.
pub fn is_user_turn(
    text: &str,
    user_keywords: &[String],
    user_name: Option<&str>,
    speaker_is_user: bool,
) -> bool {
    if speaker_is_user {
        return true;
    }
    let lower = text.to_lowercase();
    let keyword_hit = user_keywords.iter().any(|k| !k.is_empty() && lower.contains(&k.to_lowercase()));
    // The configured name is trimmed first: " Ana " should match "hola ana".
    let name_hit = user_name
        .map(str::trim)
        .is_some_and(|n| !n.is_empty() && lower.contains(&n.to_lowercase()));
    keyword_hit || name_hit
}

/// How a sentence that asks the listener for something tends to start, in
/// English and Spanish. Compared against the start of each sentence, lowercased.
const REQUEST_OPENERS: &[&str] = &[
    "what", "why", "how", "when", "where", "who", "which", "can you", "could you", "would you",
    "will you", "do you", "did you", "are you", "have you", "is there", "tell me", "explain",
    "describe", "walk me through", "talk me through", "give me", "show me", "let me know",
    "qué", "que opinas", "cómo", "como ves", "cuándo", "dónde", "por qué", "quién", "cuál",
    "puedes", "podrías", "podrias", "cuéntame", "cuentame", "explica", "explícame", "háblame",
    "hablame", "dime", "describe",
];

/// Whether what was said asks for a reply: a question, or a request such as
/// "tell me about…". This is the cue to suggest an answer without being asked.
///
/// A question mark anywhere is enough (the transcription punctuates). Without
/// one, a sentence has to start with a typical opener, as a whole word.
pub fn asks_for_reply(text: &str) -> bool {
    if text.contains('?') {
        return true;
    }
    let lower = text.to_lowercase();
    lower.split(['.', '!', ';', '\n']).any(|sentence| {
        let sentence = sentence.trim_start_matches(|c: char| !c.is_alphanumeric());
        REQUEST_OPENERS.iter().any(|opener| {
            sentence.strip_prefix(opener).is_some_and(|rest| !rest.starts_with(char::is_alphanumeric))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn questions_and_requests_ask_for_a_reply() {
        assert!(asks_for_reply("So, what is your experience with Rust?"));
        assert!(asks_for_reply("Thanks. Tell me about your last project."));
        assert!(asks_for_reply("Can you walk us through the numbers"));
        assert!(asks_for_reply("¿Cuánto tiempo llevas en ese puesto?"));
        assert!(asks_for_reply("Perfecto. Cuéntame cómo lo resolviste."));
    }

    #[test]
    fn statements_do_not_ask_for_a_reply() {
        assert!(!asks_for_reply("Good morning everyone, let's get started."));
        assert!(!asks_for_reply("Whatever works for the team is fine."));
        assert!(!asks_for_reply("However, the deadline moved to Friday."));
        assert!(!asks_for_reply("Buenos días a todos."));
        assert!(!asks_for_reply(""));
    }

    #[test]
    fn keyword_case_insensitive_and_empty_ignored() {
        let kw = vec!["opinas?".to_string(), "".to_string(), "CHRISTIAN".to_string()];
        assert!(is_user_turn("¿Tú qué OPINAS?", &kw, None, false));
        assert!(is_user_turn("christian, ¿puedes explicar?", &kw, None, false));
        assert!(!is_user_turn("buenos días a todos", &kw, None, false));
        // speaker channel always wins
        assert!(is_user_turn("cualquier cosa", &[], None, true));
    }

    #[test]
    fn user_name_counts_as_one_more_keyword() {
        assert!(is_user_turn("christian, ¿puedes explicar?", &[], Some("Christian"), false));
        assert!(is_user_turn("Gracias CHRISTIAN", &[], Some("christian"), false));
        // surrounding whitespace in the configured name is not significant
        assert!(is_user_turn("hola christian", &[], Some("  Christian "), false));
        assert!(!is_user_turn("buenos días a todos", &[], Some("Christian"), false));
    }

    #[test]
    fn absent_or_blank_user_name_does_not_break_or_match() {
        assert!(!is_user_turn("cualquier cosa", &[], None, false));
        assert!(!is_user_turn("cualquier cosa", &[], Some(""), false));
        assert!(!is_user_turn("cualquier cosa", &[], Some("   "), false));
        // keywords keep working alongside an absent name
        assert!(is_user_turn("¿tú qué opinas?", &["opinas".to_string()], None, false));
    }
}
