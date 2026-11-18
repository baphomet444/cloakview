//! The language the app opens in on a clean install.
//!
//! The UI ships two catalogs (`src/lib/locales`), so the only question is whether
//! Spanish wins. The first *supported* language in the user's list decides, the way
//! every localized app behaves: a Mac set to German with Spanish second opens in
//! Spanish rather than dropping to English.

/// `"es"` when Spanish comes before English in the preferred list, `"en"` otherwise.
///
/// Pure, so the mapping is testable without asking the system: `preferred` is what
/// macOS reports, e.g. `["es-419", "en-US"]`.
pub fn pick(preferred: &[String]) -> &'static str {
    for tag in preferred {
        // "es", "es-419", "es_ES", "ES-es" all mean Spanish; "en-GB" means English.
        let code = tag.split(['-', '_']).next().unwrap_or("");
        if code.eq_ignore_ascii_case("es") {
            return "es";
        }
        if code.eq_ignore_ascii_case("en") {
            return "en";
        }
    }
    "en"
}

/// macOS's preferred languages, most-preferred first.
///
/// `pick` is the only thing that matters to callers; this exists so the macOS API
/// call stays in one place.
#[cfg(target_os = "macos")]
pub fn system() -> Vec<String> {
    use objc2_foundation::NSLocale;
    NSLocale::preferredLanguages()
        .iter()
        .map(|language| language.to_string())
        .collect()
}

/// Off macOS there is nothing to ask, so a clean install opens in English.
#[cfg(not(target_os = "macos"))]
pub fn system() -> Vec<String> {
    Vec::new()
}

/// The settings value for a clean install: the system language, or English.
pub fn default_language() -> String {
    pick(&system()).to_string()
}

#[cfg(test)]
mod tests {
    use super::{default_language, pick};

    fn tags(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn spanish_first_is_spanish() {
        assert_eq!(pick(&tags(&["es-419", "en-US"])), "es");
        assert_eq!(pick(&tags(&["es"])), "es");
        assert_eq!(pick(&tags(&["ES-es", "en-US"])), "es");
        assert_eq!(pick(&tags(&["es_ES"])), "es");
    }

    #[test]
    fn anything_else_is_english() {
        assert_eq!(pick(&tags(&["en-US"])), "en");
        assert_eq!(pick(&[]), "en");
        assert_eq!(pick(&tags(&[""])), "en");
        assert_eq!(pick(&tags(&["de-DE", "fr-FR"])), "en");
    }

    #[test]
    fn the_first_supported_language_wins() {
        // English before Spanish: an English-first Mac stays English.
        assert_eq!(pick(&tags(&["en-US", "es-419"])), "en");
        // A German Mac with Spanish second gets Spanish, not a drop to English.
        assert_eq!(pick(&tags(&["de-DE", "es-ES"])), "es");
    }

    #[test]
    fn the_default_is_one_of_the_two_catalogs() {
        let language = default_language();
        assert!(language == "en" || language == "es", "got {language}");
    }
}
