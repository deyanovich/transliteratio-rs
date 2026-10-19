//! Modified Hepburn over the kana: shi chi tsu fu ji, the long
//! vowels with a macron (ei and ii written out), n' before a vowel
//! or y, the sokuon doubling its consonant (tch before ch).

use super::{KANA, lookup};

/// Where Hepburn departs from the kunrei table.
const HEPBURN: &[(char, &str)] = &[
    ('し', "shi"),
    ('ち', "chi"),
    ('つ', "tsu"),
    ('ふ', "fu"),
    ('じ', "ji"),
    ('ぢ', "ji"),
    ('づ', "zu"),
    ('を', "o"),
    ('ゔ', "vu"),
];

fn syllable(c: char) -> Option<&'static str> {
    lookup(HEPBURN, c).or_else(|| lookup(KANA, c))
}

fn macron(out: &mut String) {
    let bar = match out.pop() {
        Some('a') => 'ā',
        Some('i') => 'ī',
        Some('u') => 'ū',
        Some('e') => 'ē',
        Some('o') => 'ō',
        Some(other) => {
            out.push(other);
            return;
        }
        None => return,
    };
    out.push(bar);
}

pub(super) fn hepburn(text: &str) -> String {
    let chars: Vec<char> = text
        .chars()
        .map(|c| match c as u32 {
            0x30A1..=0x30F6 => char::from_u32(c as u32 - 0x60).unwrap_or(c),
            _ => c,
        })
        .collect();
    let mut out = String::with_capacity(text.len());
    let mut sokuon = false;
    let mut nasal = false;
    for (i, &c) in chars.iter().enumerate() {
        let next = chars.get(i + 1).copied();
        match c {
            'っ' => {
                sokuon = true;
                continue;
            }
            'ん' => {
                out.push('n');
                nasal = true;
                continue;
            }
            'ゃ' | 'ゅ' | 'ょ' => {
                if out.ends_with('i') {
                    out.pop();
                }
                // sh, ch, j take the vowel bare: sha, chu, jo
                let y = out.ends_with("sh") || out.ends_with("ch") || out.ends_with('j');
                out.push_str(match (c, y) {
                    ('ゃ', true) => "a",
                    ('ゅ', true) => "u",
                    (_, true) => "o",
                    ('ゃ', false) => "ya",
                    ('ゅ', false) => "yu",
                    _ => "yo",
                });
                continue;
            }
            'ぁ' | 'ぃ' | 'ぅ' | 'ぇ' | 'ぉ' => {
                // a small vowel replaces the syllable's own: ファ fa, ティ ti
                if out.chars().last().is_some_and(|l| "aiueo".contains(l)) {
                    out.pop();
                }
                out.push(match c {
                    'ぁ' => 'a',
                    'ぃ' => 'i',
                    'ぅ' => 'u',
                    'ぇ' => 'e',
                    _ => 'o',
                });
                continue;
            }
            'ー' => {
                macron(&mut out);
                continue;
            }
            _ => {}
        }
        match syllable(c) {
            Some(m) => {
                if nasal && m.starts_with(['a', 'i', 'u', 'e', 'o', 'y']) {
                    out.push('\'');
                }
                nasal = false;
                if sokuon {
                    if m.starts_with("ch") {
                        out.push('t');
                    } else if let Some(first) = m.chars().next() {
                        out.push(first);
                    }
                    sokuon = false;
                }
                let long = m.len() == 1
                    && matches!(
                        (out.chars().last(), m),
                        (Some('o'), "u" | "o")
                            | (Some('u'), "u")
                            | (Some('e'), "e")
                            | (Some('a'), "a")
                    );
                if long {
                    macron(&mut out);
                } else {
                    out.push_str(m);
                }
                let _ = next;
            }
            None => {
                sokuon = false;
                nasal = false;
                out.push(c);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn modified_hepburn() {
        assert_eq!(translit("しんぶん", "hepburn").unwrap(), "shinbun");
        assert_eq!(
            translit("とうきょう おおさか", "hepburn").unwrap(),
            "tōkyō ōsaka"
        );
        assert_eq!(
            translit("まっちゃ きって", "hepburn").unwrap(),
            "matcha kitte"
        );
        assert_eq!(
            translit("しんいち ほんや", "hepburn").unwrap(),
            "shin'ichi hon'ya"
        );
        assert_eq!(
            translit("ちゃ じゅう しょう", "hepburn").unwrap(),
            "cha jū shō"
        );
        assert_eq!(translit("えいが いいえ", "hepburn").unwrap(), "eiga iie");
        assert_eq!(
            translit("コーヒー ファン ティー", "hepburn").unwrap(),
            "kōhī fan tī"
        );
        assert_eq!(translit("ふじさん", "hepburn").unwrap(), "fujisan");
        // kunrei is unchanged
        assert_eq!(translit("しんぶん", "iso3602").unwrap(), "sinbun");
    }
}
