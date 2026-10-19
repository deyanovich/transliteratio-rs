//! Mandarin romanization over bopomofo and over pinyin readings
//! with tone numbers: Hanyu Pinyin (ISO 7098, tone marks) and
//! Wade–Giles (superscript tone numbers). Chinese characters have
//! no letter table: they need the readings library.

/// A syllable in pinyin pieces: the initial, the toneless rime,
/// the tone (1–4, 5 neutral, 0 unmarked).
struct Syllable {
    initial: &'static str,
    rime: String,
    tone: u8,
}

const INITIALS: &[(char, &str)] = &[
    ('ㄅ', "b"),
    ('ㄆ', "p"),
    ('ㄇ', "m"),
    ('ㄈ', "f"),
    ('ㄉ', "d"),
    ('ㄊ', "t"),
    ('ㄋ', "n"),
    ('ㄌ', "l"),
    ('ㄍ', "g"),
    ('ㄎ', "k"),
    ('ㄏ', "h"),
    ('ㄐ', "j"),
    ('ㄑ', "q"),
    ('ㄒ', "x"),
    ('ㄓ', "zh"),
    ('ㄔ', "ch"),
    ('ㄕ', "sh"),
    ('ㄖ', "r"),
    ('ㄗ', "z"),
    ('ㄘ', "c"),
    ('ㄙ', "s"),
];

const FINALS: &[(char, &str)] = &[
    ('ㄚ', "a"),
    ('ㄛ', "o"),
    ('ㄜ', "e"),
    ('ㄝ', "ê"),
    ('ㄞ', "ai"),
    ('ㄟ', "ei"),
    ('ㄠ', "ao"),
    ('ㄡ', "ou"),
    ('ㄢ', "an"),
    ('ㄣ', "en"),
    ('ㄤ', "ang"),
    ('ㄥ', "eng"),
    ('ㄦ', "er"),
    ('ㄭ', "i"),
];

fn tone_of(c: char) -> Option<u8> {
    Some(match c {
        'ˉ' => 1,
        'ˊ' => 2,
        'ˇ' => 3,
        'ˋ' => 4,
        '˙' => 5,
        _ => return None,
    })
}

fn find(table: &[(char, &'static str)], c: char) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == c).map(|(_, v)| *v)
}

/// The pinyin rime of a bopomofo medial + final behind an initial
/// (or none): the spelling rules of the scheme.
fn rime(initial: &str, medial: Option<char>, final_: Option<&str>) -> String {
    let bare = initial.is_empty();
    let jqx = matches!(initial, "j" | "q" | "x");
    let f = final_.unwrap_or("");
    match medial {
        None => {
            if f.is_empty() {
                // zhi chi shi ri zi ci si
                return "i".into();
            }
            f.to_string()
        }
        Some('ㄧ') => {
            let r = match f {
                "" => "i",
                "a" => "ia",
                "o" => "io",
                "e" | "ê" => "ie",
                "ai" => "iai",
                "ao" => "iao",
                "ou" => "iu",
                "an" => "ian",
                "en" => "in",
                "ang" => "iang",
                "eng" => "ing",
                other => other,
            };
            if bare {
                match r {
                    "i" => "yi".into(),
                    "iu" => "you".into(),
                    "in" => "yin".into(),
                    "ing" => "ying".into(),
                    other => format!("y{}", &other[1..]),
                }
            } else {
                r.into()
            }
        }
        Some('ㄨ') => {
            let r = match f {
                "" => "u",
                "a" => "ua",
                "o" => "uo",
                "ai" => "uai",
                "ei" => "ui",
                "an" => "uan",
                "en" => "un",
                "ang" => "uang",
                "eng" => {
                    if bare {
                        "ueng"
                    } else {
                        "ong"
                    }
                }
                other => other,
            };
            if bare {
                match r {
                    "u" => "wu".into(),
                    "ui" => "wei".into(),
                    "un" => "wen".into(),
                    other => format!("w{}", &other[1..]),
                }
            } else {
                r.into()
            }
        }
        Some('ㄩ') => {
            let r = match f {
                "" => "ü",
                "e" | "ê" => "üe",
                "an" => "üan",
                "en" => "ün",
                "eng" => "iong",
                other => other,
            };
            if bare {
                match r {
                    "iong" => "yong".into(),
                    other => format!("yu{}", &other['ü'.len_utf8()..]),
                }
            } else if jqx {
                r.replace('ü', "u")
            } else {
                r.into()
            }
        }
        _ => f.to_string(),
    }
}

/// The tone mark on its vowel: a or e first, the o of ou, else the
/// last vowel.
fn mark_tone(rime: &str, tone: u8) -> String {
    let mark = match tone {
        1 => '\u{304}',
        2 => '\u{301}',
        3 => '\u{30c}',
        4 => '\u{300}',
        _ => return rime.to_string(),
    };
    let chars: Vec<char> = rime.chars().collect();
    let vowel = |c: char| "aeiouüAEIOUÜ".contains(c);
    let at = chars
        .iter()
        .position(|c| matches!(c, 'a' | 'e' | 'A' | 'E'))
        .or_else(|| {
            chars
                .windows(2)
                .position(|w| (w[0] == 'o' || w[0] == 'O') && w[1] == 'u')
        })
        .or_else(|| chars.iter().rposition(|c| vowel(*c)));
    let Some(at) = at else {
        return rime.to_string();
    };
    use unicode_normalization::UnicodeNormalization;
    let mut out = String::new();
    for (i, c) in chars.iter().enumerate() {
        out.push(*c);
        if i == at {
            out.push(mark);
        }
    }
    out.nfc().collect()
}

fn superscript(tone: u8) -> &'static str {
    match tone {
        1 => "¹",
        2 => "²",
        3 => "³",
        4 => "⁴",
        _ => "",
    }
}

/// Wade–Giles from the pinyin pieces.
fn wade_giles(s: &Syllable) -> String {
    let (i, r) = (s.initial, s.rime.as_str());
    let initial = match i {
        "b" => "p",
        "p" => "p'",
        "d" => "t",
        "t" => "t'",
        "g" => "k",
        "k" => "k'",
        "j" => "ch",
        "q" => "ch'",
        "x" => "hs",
        "zh" => "ch",
        "ch" => "ch'",
        "r" => "j",
        "z" => "ts",
        "c" => "ts'",
        other => other,
    };
    // the apical vowel after zh ch sh r z c s
    if r == "i" {
        return match i {
            "zh" => "chih".into(),
            "ch" => "ch'ih".into(),
            "sh" => "shih".into(),
            "r" => "jih".into(),
            "z" => "tzŭ".into(),
            "c" => "tz'ŭ".into(),
            "s" => "ssŭ".into(),
            _ => format!("{initial}i"),
        };
    }
    let jqx = matches!(i, "j" | "q" | "x");
    let gkh = matches!(i, "g" | "k" | "h");
    let rime = match r {
        "e" if i.is_empty() => "ê",
        "e" if gkh => "o",
        "e" => "ê",
        "en" => "ên",
        "eng" => "êng",
        "er" => "êrh",
        "ie" => "ieh",
        "ian" => "ien",
        "iong" => "iung",
        "ong" => "ung",
        "ui" if gkh => "uei",
        "u" if jqx => "ü",
        "ue" if jqx => "üeh",
        "uan" if jqx => "üan",
        "un" if jqx => "ün",
        "üe" => "üeh",
        "yi" => "i",
        "ye" => "yeh",
        "you" => "yu",
        "yan" => "yen",
        "yong" => "yung",
        "wen" => "wên",
        "weng" => "wêng",
        "yu" => "yü",
        "yue" => "yüeh",
        "yuan" => "yüan",
        "yun" => "yün",
        other => other,
    };
    format!("{initial}{rime}")
}

/// Bopomofo text: syllables, each to its pieces; everything else
/// passes through.
fn bopomofo(text: &str, render: &dyn Fn(&Syllable) -> String) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        // the first tone is unmarked in bopomofo
        let mut tone = 1u8;
        let mut j = i;
        // a leading neutral-tone dot
        if chars.get(j) == Some(&'˙') {
            tone = 5;
            j += 1;
        }
        let initial = chars.get(j).and_then(|c| find(INITIALS, *c));
        if initial.is_some() {
            j += 1;
        }
        let medial = chars
            .get(j)
            .filter(|c| matches!(c, 'ㄧ' | 'ㄨ' | 'ㄩ'))
            .copied();
        if medial.is_some() {
            j += 1;
        }
        let final_ = chars.get(j).and_then(|c| find(FINALS, *c));
        if final_.is_some() {
            j += 1;
        }
        if initial.is_none() && medial.is_none() && final_.is_none() {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        if let Some(t) = chars.get(j).and_then(|c| tone_of(*c)) {
            tone = t;
            j += 1;
        }
        let initial = initial.unwrap_or("");
        let final_ = if final_ == Some("i") { None } else { final_ };
        let s = Syllable {
            initial,
            rime: rime(initial, medial, final_),
            tone,
        };
        out.push_str(&render(&s));
        i = j;
    }
    out
}

/// A pinyin reading with a tone number (ma3, lü4, nv3, Zhong1):
/// its pieces, or None when the letters are not a syllable.
fn reading(word: &str) -> Option<Syllable> {
    let last = word.chars().last()?;
    let (letters, digit) = word.split_at(word.len() - last.len_utf8());
    let tone: u8 = digit.parse().ok().filter(|t| (1..=5).contains(t))?;
    let lower = letters.to_lowercase().replace('v', "ü");
    let initial = [
        "zh", "ch", "sh", "b", "p", "m", "f", "d", "t", "n", "l", "g", "k", "h", "j", "q", "x",
        "r", "z", "c", "s",
    ]
    .into_iter()
    .find(|p| lower.starts_with(p))
    .unwrap_or("");
    let rime = lower[initial.len()..].to_string();
    if rime.is_empty() || !rime.chars().all(|c| "aeiouüngr".contains(c)) {
        return None;
    }
    Some(Syllable {
        initial,
        rime,
        tone,
    })
}

/// Pinyin readings with tone numbers, word by word; a word without
/// a digit or outside the syllabary passes through.
fn readings(text: &str, render: &dyn Fn(&Syllable, bool) -> String) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if word.is_empty() {
            return;
        }
        let capital = word.chars().next().is_some_and(char::is_uppercase);
        match reading(word) {
            Some(s) => out.push_str(&render(&s, capital)),
            None => out.push_str(word),
        }
        word.clear();
    };
    for c in text.chars() {
        if c.is_alphabetic() && (c as u32) < 0x3000 {
            // a syllable ended by a digit may run into the next
            // syllable's letters: Zhong1guo2
            if !word.is_empty() && word.ends_with(|d: char| d.is_ascii_digit()) {
                flush(&mut word, &mut out);
            }
            word.push(c);
        } else if c.is_ascii_digit()
            && !word.is_empty()
            && !word.ends_with(|d: char| d.is_ascii_digit())
        {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

fn capitalize(s: &str, capital: bool) -> String {
    if !capital {
        return s.to_string();
    }
    let mut it = s.chars();
    match it.next() {
        Some(first) => first.to_uppercase().chain(it).collect(),
        None => String::new(),
    }
}

pub(super) fn pinyin(text: &str) -> String {
    let step = bopomofo(text, &|s| {
        let spelled = format!("{}{}", s.initial, s.rime);
        mark_tone(&spelled, s.tone)
    });
    readings(&step, &|s, capital| {
        let spelled = format!("{}{}", s.initial, s.rime);
        capitalize(&mark_tone(&spelled, s.tone), capital)
    })
}

pub(super) fn wade(text: &str) -> String {
    let step = bopomofo(text, &|s| {
        format!("{}{}", wade_giles(s), superscript(s.tone))
    });
    readings(&step, &|s, capital| {
        format!(
            "{}{}",
            capitalize(&wade_giles(s), capital),
            superscript(s.tone)
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn pinyin_over_bopomofo_and_readings() {
        assert_eq!(translit("ㄓㄨㄥ ㄍㄨㄛˊ", "pinyin").unwrap(), "zhōng guó");
        assert_eq!(translit("ㄅㄟˇㄐㄧㄥ", "pinyin").unwrap(), "běijīng");
        assert_eq!(
            translit("ㄋㄩˇ ㄒㄩˇ ㄧㄡˇ ㄨㄟˋ ㄩㄢˋ", "pinyin").unwrap(),
            "nǚ xǔ yǒu wèi yuàn"
        );
        assert_eq!(
            translit("ㄕˊ ㄖˋ ㄗˋ ˙ㄇㄚ", "pinyin").unwrap(),
            "shí rì zì ma"
        );
        assert_eq!(
            translit("Zhong1guo2 ren2 nv3 lu:4", "pinyin").unwrap(),
            "Zhōngguó rén nǚ lu:4"
        );
        assert_eq!(translit("ni3 hao3 ma5", "pinyin").unwrap(), "nǐ hǎo ma");
        assert_eq!(translit("Beijing 1949", "pinyin").unwrap(), "Beijing 1949");
    }

    #[test]
    fn wade_giles() {
        assert_eq!(translit("ㄓㄨㄥ ㄍㄨㄛˊ", "wade").unwrap(), "chung¹ kuo²");
        assert_eq!(translit("ㄅㄟˇㄐㄧㄥ", "wade").unwrap(), "pei³ching¹");
        assert_eq!(translit("Mao2 Ze2dong1", "wade").unwrap(), "Mao² Tsê²tung¹");
        assert_eq!(
            translit("zhi1 chi2 shi3 ri4 zi1 ci2 si3", "wade").unwrap(),
            "chih¹ ch'ih² shih³ jih⁴ tzŭ¹ tz'ŭ² ssŭ³"
        );
        assert_eq!(
            translit("xu1 ju2 que4 gui4 he2 er3", "wade").unwrap(),
            "hsü¹ chü² ch'üeh⁴ kuei⁴ ho² êrh³"
        );
    }
}
