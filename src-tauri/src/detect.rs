pub fn is_user_turn(text: &str, user_keywords: &[String], speaker_is_user: bool) -> bool {
    if speaker_is_user {
        return true;
    }
    let lower = text.to_lowercase();
    user_keywords.iter().any(|k| !k.is_empty() && lower.contains(&k.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_case_insensitive_and_empty_ignored() {
        let kw = vec!["opinas?".to_string(), "".to_string(), "CHRISTIAN".to_string()];
        assert!(is_user_turn("¿Tú qué OPINAS?", &kw, false));
        assert!(is_user_turn("christian, ¿puedes explicar?", &kw, false));
        assert!(!is_user_turn("buenos días a todos", &kw, false));
        // speaker channel always wins
        assert!(is_user_turn("cualquier cosa", &[], true));
    }
}
