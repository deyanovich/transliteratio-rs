//! Devanagari beyond ISO 15919: IAST (ṃ ṛ ṝ ḷ for the ISO ṁ r̥ r̥̄
//! l̥) and the Hunterian system (no diacritics, ch chh sh, ri).

use super::devanagari;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

fn respell(text: &str, pairs: &[(&str, &str)]) -> String {
    let mut s = text.to_string();
    for (from, to) in pairs {
        s = s.replace(from, to);
    }
    s
}

pub(super) fn iast(text: &str) -> String {
    let iso: String = devanagari(text).nfc().collect();
    respell(
        &iso,
        &[
            ("r̥̄", "ṝ"),
            ("r̥", "ṛ"),
            ("l̥̄", "ḹ"),
            ("l̥", "ḷ"),
            ("ṁ", "ṃ"),
            ("k͟h", "kh"),
        ],
    )
    .nfc()
    .collect()
}

pub(super) fn hunterian(text: &str) -> String {
    let iso: String = devanagari(text).nfc().collect();
    let spelt = respell(
        &iso,
        &[
            ("r̥̄", "ri"),
            ("r̥", "ri"),
            ("l̥̄", "li"),
            ("l̥", "li"),
            ("ch", "\u{1}"),
            ("c", "ch"),
            ("\u{1}", "chh"),
            ("ś", "sh"),
            ("ṣ", "sh"),
            ("ṁ", "n"),
            ("m̐", "n"),
            ("ḥ", "h"),
            ("k͟h", "kh"),
            ("ê", "e"),
            ("ô", "o"),
        ],
    );
    spelt
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .nfc()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn iast_and_hunterian() {
        assert_eq!(translit("संस्कृतम्", "iast").unwrap(), "saṃskṛtam");
        assert_eq!(translit("ऋषि कृष्ण", "iast").unwrap(), "ṛṣi kṛṣṇa");
        assert_eq!(translit("संस्कृतम्", "hunterian").unwrap(), "sanskritam");
        assert_eq!(
            translit("कृष्ण चन्द्र छत्री शिव", "hunterian").unwrap(),
            "krishna chandra chhatri shiva"
        );
        assert_eq!(translit("महाराष्ट्र", "hunterian").unwrap(), "maharashtra");
        // ISO 15919 is unchanged
        assert_eq!(translit("संस्कृतम्", "iso15919").unwrap(), "saṁskr̥tam");
    }
}
