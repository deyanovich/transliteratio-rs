//! The inverses of ISO 9 and GOST 7.79 system B: Latin text those
//! systems produced, back to Cyrillic. Longest match first, case
//! from the first letter of the match, every other character kept.

use super::{GOST_B, ISO9, lower};
use unicode_normalization::UnicodeNormalization;

/// The (latin, cyrillic) pairs of a table with the extra readings,
/// longest Latin first; a Latin spelling shared by two letters
/// answers to the first in the table.
fn pairs(table: &[(char, &'static str)], extra: &[(&'static str, char)]) -> Vec<(Vec<char>, char)> {
    let mut out: Vec<(Vec<char>, char)> = Vec::new();
    for (lat, cyr) in extra {
        out.push((lat.nfc().collect(), *cyr));
    }
    for (cyr, lat) in table {
        if lat.is_empty() {
            continue;
        }
        let key: Vec<char> = lat.nfc().collect();
        if !out.iter().any(|(k, _)| *k == key) {
            out.push((key, *cyr));
        }
    }
    out.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
    out
}

fn invert(text: &str, pairs: &[(Vec<char>, char)]) -> String {
    let text: String = text.nfc().collect();
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    'outer: while i < chars.len() {
        for (key, cyr) in pairs {
            let n = key.len();
            if i + n <= chars.len()
                && key
                    .iter()
                    .zip(&chars[i..i + n])
                    .all(|(k, c)| *k == lower(*c))
            {
                if chars[i].is_uppercase() {
                    out.extend(cyr.to_uppercase());
                } else {
                    out.push(*cyr);
                }
                i += n;
                continue 'outer;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// ISO 9 back to Cyrillic: the standard is one letter to one letter
/// across its languages, so the inverse is exact.
pub(super) fn iso9_cyrl(text: &str) -> String {
    invert(text, &pairs(ISO9, &[]))
}

/// GOST 7.79 B back to Cyrillic: the digraphs first (shh, zh, ch,
/// yo, yu, ya, ye, cz, e`, y`, ``), then the letters; a lone c is
/// ц (the system's c before е и й ы), a lone y is ы, ye is є.
pub(super) fn gost_cyrl(text: &str) -> String {
    invert(text, &pairs(GOST_B, &[("c", 'ц'), ("y", 'ы'), ("ye", 'є')]))
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn iso9_round_trips() {
        for word in [
            "Съешь же ещё этих мягких французских булок, да выпей чаю",
            "Київ Ґанок Їжак Євген",
            "Љубљана Ђорђе Џек Ћирилица",
            "Ѓорѕе Ќиро",
        ] {
            let latin = translit(word, "iso9").unwrap();
            assert_eq!(translit(&latin, "iso9-cyrl").unwrap(), word, "{latin}");
        }
    }

    #[test]
    fn gost_b_round_trips_russian() {
        for word in [
            "Щука",
            "ЩУКА",
            "Цыплёнок жуёт объедки",
            "Юлия, Яков и Эмма",
            "цирк Чехов",
        ] {
            let latin = translit(word, "gost").unwrap();
            assert_eq!(translit(&latin, "gost-cyrl").unwrap(), word, "{latin}");
        }
        assert_eq!(
            translit("Shhuka i czar`", "gost-cyrl").unwrap(),
            "Щука и царь"
        );
        assert_eq!(translit("Moskva 1812", "gost-cyrl").unwrap(), "Москва 1812");
    }
}
