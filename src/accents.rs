//! Accents and marks: `deaccent(LANG)` removes the combining marks
//! a language counts as accents and keeps the ones that make
//! letters (ñ, ç, ё, â); `strip` removes every nonspacing mark.
//! Both work on the NFD form and recompose, so a text that
//! arrived precomposed leaves precomposed.

#[cfg(feature = "deaccent")]
use crate::Error;
#[cfg(feature = "strip")]
use unicode_normalization::UnicodeNormalization;
#[cfg(feature = "strip")]
use unicode_properties::UnicodeGeneralCategory;
#[cfg(feature = "strip")]
use unicode_properties::general_category::GeneralCategory;

#[cfg(feature = "deaccent")]
mod tables {
    use super::Error;
    use unicode_normalization::UnicodeNormalization;

    const ACUTE: char = '\u{301}';
    const GRAVE: char = '\u{300}';
    const CIRCUMFLEX: char = '\u{302}';
    const TILDE: char = '\u{303}';
    const MACRON: char = '\u{304}';
    const BREVE: char = '\u{306}';
    const DIAERESIS: char = '\u{308}';
    const HOOK: char = '\u{309}';
    const PSILI: char = '\u{313}';
    const DASIA: char = '\u{314}';
    const DOT_BELOW: char = '\u{323}';
    const CEDILLA: char = '\u{327}';
    const PERISPOMENI: char = '\u{342}';

    /// The languages with an accent table, by tag.
    pub const LANGUAGES: &[&str] = &[
        "ru", "uk", "be", "bg", "grc", "el", "fr", "es", "pt", "ca", "it", "ro", "vi", "de", "en",
        "sv", "fi", "da", "no", "sr", "hr", "bs", "sl", "cs", "sk", "pl", "lt", "lv", "et", "hu",
        "tr",
    ];

    /// The combining marks the language counts as accents; `None` for a
    /// language without a table.
    fn accents_of(lang: &str) -> Option<&'static [char]> {
        Some(match lang {
            // the stress mark; ё й ї ў are letters
            "ru" | "uk" | "be" | "bg" => &[ACUTE],
            // the three accents, the breathings, the quantities; the
            // iota subscript and the dialytika stay
            "grc" => &[
                ACUTE,
                GRAVE,
                CIRCUMFLEX,
                PERISPOMENI,
                PSILI,
                DASIA,
                MACRON,
                BREVE,
            ],
            // the tonos; the dialytika stays
            "el" => &[ACUTE],
            "fr" => &[ACUTE, GRAVE, CIRCUMFLEX, DIAERESIS, CEDILLA],
            // ñ stays
            "es" => &[ACUTE, DIAERESIS],
            "pt" => &[ACUTE, GRAVE, CIRCUMFLEX, TILDE, DIAERESIS, CEDILLA],
            // ç and l·l stay
            "ca" => &[ACUTE, GRAVE, CIRCUMFLEX, TILDE, DIAERESIS],
            // ș ț stay (comma below)
            "it" | "ro" => &[ACUTE, GRAVE, CIRCUMFLEX, BREVE],
            // the tone marks; â ă ê ô ơ ư đ are letters
            "vi" => &[ACUTE, GRAVE, HOOK, TILDE, DOT_BELOW],
            // every mark is part of a letter
            "de" | "en" | "sv" | "fi" | "da" | "no" | "nb" | "nn" | "sr" | "hr" | "bs" | "sl"
            | "cs" | "sk" | "pl" | "lt" | "lv" | "et" | "hu" | "tr" => &[],
            _ => return None,
        })
    }

    /// The language of a locale tag: `ru-RU` → `ru`, `grc` → `grc`.
    pub fn language_of(tag: &str) -> String {
        tag.split(['-', '_'])
            .next()
            .unwrap_or(tag)
            .to_ascii_lowercase()
    }

    /// The Cargo feature that carries a language's accent table; the
    /// identity languages need none.
    fn feature_of(lang: &str) -> Option<&'static str> {
        Some(match lang {
            "ru" | "uk" | "be" | "bg" => "deaccent-cyrillic",
            "grc" | "el" => "deaccent-greek",
            "fr" | "es" | "pt" | "ca" | "it" | "ro" => "deaccent-romance",
            "vi" => "deaccent-vietnamese",
            _ => return None,
        })
    }

    /// `text` without the accents of `lang` (a language tag; `ru-RU` is
    /// `ru`); a language without a table is [`Error::UnknownLanguage`],
    /// one whose feature this build left off [`Error::FeatureOff`].
    pub fn deaccent(text: &str, lang: &str) -> Result<String, Error> {
        let lang = language_of(lang);
        let Some(marks) = accents_of(&lang) else {
            return Err(Error::UnknownLanguage(lang));
        };
        if let Some(feature) = feature_of(&lang)
            && !crate::enabled(feature)
        {
            return Err(Error::FeatureOff {
                table: lang,
                feature,
            });
        }
        if marks.is_empty() {
            return Ok(text.nfc().collect());
        }
        Ok(text.nfd().filter(|c| !marks.contains(c)).nfc().collect())
    }
}

#[cfg(feature = "deaccent")]
pub use tables::{LANGUAGES, deaccent, language_of};

/// `text` without any nonspacing mark.
#[cfg(feature = "strip")]
pub fn strip(text: &str) -> String {
    text.nfd()
        .filter(|c| c.general_category() != GeneralCategory::NonspacingMark)
        .nfc()
        .collect()
}

#[cfg(all(test, feature = "deaccent", feature = "strip"))]
mod tests {
    use super::*;

    #[test]
    fn stress_goes_the_letters_stay() {
        assert_eq!(deaccent("ёлка подъё\u{301}м", "ru").unwrap(), "ёлка подъём");
        assert_eq!(deaccent("за\u{301}мок", "ru-RU").unwrap(), "замок");
        assert_eq!(deaccent("Ки́їв", "uk").unwrap(), "Київ");
    }

    #[test]
    fn greek_accents_and_breathings() {
        assert_eq!(deaccent("Ἀχιλλεύς", "grc").unwrap(), "Αχιλλευς");
        // the iota subscript and the dialytika stay
        assert_eq!(deaccent("τῷ ἀϋπνίᾳ", "grc").unwrap(), "τῳ αϋπνιᾳ");
        assert_eq!(
            deaccent("Οδυσσέας προϊόν", "el").unwrap(),
            "Οδυσσεας προϊον"
        );
    }

    #[test]
    fn romance_tables_keep_their_letters() {
        assert_eq!(
            deaccent("élève œuvre garçon", "fr").unwrap(),
            "eleve œuvre garcon"
        );
        assert_eq!(
            deaccent("niño pingüino canción", "es").unwrap(),
            "niño pinguino cancion"
        );
        assert_eq!(deaccent("ação São", "pt").unwrap(), "acao Sao");
        assert_eq!(deaccent("Barça col·legi", "ca").unwrap(), "Barça col·legi");
        assert_eq!(deaccent("perché", "it").unwrap(), "perche");
        assert_eq!(deaccent("București ață", "ro").unwrap(), "București ața");
        assert_eq!(deaccent("Việt Nam đẹp", "vi").unwrap(), "Viêt Nam đep");
    }

    #[test]
    fn identity_languages_and_unknown() {
        assert_eq!(deaccent("Müller Łódź", "de").unwrap(), "Müller Łódź");
        assert_eq!(deaccent("Łódź", "pl").unwrap(), "Łódź");
        assert_eq!(
            deaccent("x", "klingon"),
            Err(Error::UnknownLanguage("klingon".into()))
        );
    }

    #[test]
    fn strip_takes_every_mark_and_no_letter() {
        assert_eq!(
            strip("Ἀχιλλεύς élève Łódź ёлка"),
            "Αχιλλευς eleve Łodz елка"
        );
        assert_eq!(strip("œuvre straße"), "œuvre straße");
        // a spacing vowel sign is a letter's part, not a mark to strip
        assert_eq!(strip("की"), "की");
    }
}
