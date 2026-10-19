//! The two families that span scripts: ALA-LC (the Library of
//! Congress tables, by script and language of the letters) and
//! BGN/PCGN (the boards' systems, by language). A run in a script
//! the family has no table for passes through.

use super::semitic::{arabic_with, hebrew_with};
use super::{ALA_LC, SR_LATN, Script, by_script, cased, elot, lookup, lower, plain, push_cased};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

// ---------------------------------------------------------------- Cyrillic

/// Which Cyrillic language the letters of a run show: the
/// distinctive letters decide, ъ before a consonant means
/// Bulgarian, Russian otherwise.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Slavic {
    Ru,
    Uk,
    Be,
    Bg,
    Sr,
    Mk,
    Kk,
}

fn slavic_of(run: &str) -> Slavic {
    let has = |set: &str| run.chars().any(|c| set.contains(lower(c)));
    if has("ѓќѕ") {
        Slavic::Mk
    } else if has("ђћљњџј") {
        Slavic::Sr
    } else if has("әғқңөұүһ") {
        Slavic::Kk
    } else if has("ў") {
        Slavic::Be
    } else if has("їєґ") {
        Slavic::Uk
    } else {
        let chars: Vec<char> = run.chars().collect();
        let vowel = |c: char| "аеёиоуыэюяіїє".contains(lower(c));
        let bulgarian = chars.iter().enumerate().any(|(i, c)| {
            lower(*c) == 'ъ'
                && chars
                    .get(i + 1)
                    .is_some_and(|n| n.is_alphabetic() && !vowel(*n))
        });
        if bulgarian { Slavic::Bg } else { Slavic::Ru }
    }
}

/// ALA-LC for the other Cyrillic languages, where they depart from
/// the Russian table.
const ALA_OTHER: &[(char, &str)] = &[
    ('і', "i"),
    ('ї', "ï"),
    ('є', "i\u{361}e"),
    ('ґ', "g"),
    ('ў', "ŭ"),
    ('ђ', "đ"),
    ('ј', "j"),
    ('љ', "lj"),
    ('њ', "nj"),
    ('ћ', "ć"),
    ('џ', "dž"),
    ('ѓ', "ǵ"),
    ('ќ', "ḱ"),
    ('ѕ', "dz"),
    ('ә', "ă"),
    ('ғ', "gh"),
    ('қ', "q"),
    ('ң', "ng"),
    ('ө', "ȯ"),
    ('ұ', "u"),
    ('ү', "ü"),
    ('һ', "h"),
];

fn ala_cyrillic(run: &str) -> String {
    let lang = slavic_of(run);
    cased(run, |c| match (lang, c) {
        (Slavic::Bg, 'ъ') => Some("ŭ"),
        (Slavic::Bg, 'щ') => Some("sht"),
        (Slavic::Uk, 'и') => Some("y"),
        (Slavic::Uk, 'г') => Some("h"),
        (Slavic::Be, 'г') => Some("h"),
        (Slavic::Sr | Slavic::Mk, 'х') => Some("h"),
        (Slavic::Sr | Slavic::Mk, 'ц') => Some("c"),
        (Slavic::Sr | Slavic::Mk, 'ч') => Some("č"),
        (Slavic::Sr | Slavic::Mk, 'ш') => Some("š"),
        (Slavic::Sr | Slavic::Mk, 'ж') => Some("ž"),
        (Slavic::Kk, 'у') => Some("ū"),
        (Slavic::Kk, 'и') => Some("ī"),
        _ => lookup(ALA_OTHER, c).or_else(|| lookup(ALA_LC, c)),
    })
}

/// BGN/PCGN for Ukrainian (1965), Belarusian (1979), Bulgarian
/// (1952), Macedonian (1981), Kazakh (1979); Serbian is the Latin
/// alphabet; Russian is the 1947 system.
const BGN_UK: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "h"),
    ('ґ', "g"),
    ('д', "d"),
    ('е', "e"),
    ('є', "ye"),
    ('ж', "zh"),
    ('з', "z"),
    ('и', "y"),
    ('і', "i"),
    ('ї', "yi"),
    ('й', "y"),
    ('к', "k"),
    ('л', "l"),
    ('м', "m"),
    ('н', "n"),
    ('о', "o"),
    ('п', "p"),
    ('р', "r"),
    ('с', "s"),
    ('т', "t"),
    ('у', "u"),
    ('ф', "f"),
    ('х', "kh"),
    ('ц', "ts"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('щ', "shch"),
    ('ь', "ʼ"),
    ('ю', "yu"),
    ('я', "ya"),
    ('\'', "ʼ"),
    ('’', "ʼ"),
];

const BGN_BE: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "h"),
    ('ґ', "g"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "yo"),
    ('ж', "zh"),
    ('з', "z"),
    ('і', "i"),
    ('й', "y"),
    ('к', "k"),
    ('л', "l"),
    ('м', "m"),
    ('н', "n"),
    ('о', "o"),
    ('п', "p"),
    ('р', "r"),
    ('с', "s"),
    ('т', "t"),
    ('у', "u"),
    ('ў', "w"),
    ('ф', "f"),
    ('х', "kh"),
    ('ц', "ts"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('ы', "y"),
    ('ь', "ʼ"),
    ('э', "e"),
    ('ю', "yu"),
    ('я', "ya"),
    ('\'', "ʼ"),
    ('’', "ʼ"),
];

const BGN_BG: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('е', "e"),
    ('ж', "zh"),
    ('з', "z"),
    ('и', "i"),
    ('й', "y"),
    ('к', "k"),
    ('л', "l"),
    ('м', "m"),
    ('н', "n"),
    ('о', "o"),
    ('п', "p"),
    ('р', "r"),
    ('с', "s"),
    ('т', "t"),
    ('у', "u"),
    ('ф', "f"),
    ('х', "kh"),
    ('ц', "ts"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('щ', "sht"),
    ('ъ', "ŭ"),
    ('ь', "ʼ"),
    ('ю', "yu"),
    ('я', "ya"),
    ('ѫ', "ŭ"),
    ('ѣ', "e"),
];

const BGN_MK: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('ѓ', "đ"),
    ('е', "e"),
    ('ж', "ž"),
    ('з', "z"),
    ('ѕ', "dz"),
    ('и', "i"),
    ('ј', "j"),
    ('к', "k"),
    ('л', "l"),
    ('љ', "lj"),
    ('м', "m"),
    ('н', "n"),
    ('њ', "nj"),
    ('о', "o"),
    ('п', "p"),
    ('р', "r"),
    ('с', "s"),
    ('т', "t"),
    ('ќ', "ḱ"),
    ('у', "u"),
    ('ф', "f"),
    ('х', "h"),
    ('ц', "c"),
    ('ч', "č"),
    ('џ', "dž"),
    ('ш', "š"),
];

const BGN_KK: &[(char, &str)] = &[
    ('а', "a"),
    ('ә', "ä"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('ғ', "gh"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "yo"),
    ('ж', "zh"),
    ('з', "z"),
    ('и', "ī"),
    ('й', "y"),
    ('к', "k"),
    ('қ', "q"),
    ('л', "l"),
    ('м', "m"),
    ('н', "n"),
    ('ң', "ng"),
    ('о', "o"),
    ('ө', "ö"),
    ('п', "p"),
    ('р', "r"),
    ('с', "s"),
    ('т', "t"),
    ('у', "ū"),
    ('ұ', "u"),
    ('ү', "ü"),
    ('ф', "f"),
    ('х', "kh"),
    ('һ', "h"),
    ('ц', "ts"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('щ', "shch"),
    ('ъ', "ʺ"),
    ('ы', "y"),
    ('і', "i"),
    ('ь', "ʼ"),
    ('э', "e"),
    ('ю', "yu"),
    ('я', "ya"),
];

/// The 1979 Belarusian system's one context rule: е is ye at the
/// start of a word and after a vowel, ь or the apostrophe.
fn bgn_belarusian(run: &str) -> String {
    let chars: Vec<char> = run.chars().collect();
    let mut out = String::with_capacity(run.len());
    let vowelish = |c: char| "аеёіоуыэюяўь'’".contains(lower(c));
    for i in 0..chars.len() {
        let l = lower(chars[i]);
        let yot = l == 'е' && (i == 0 || !chars[i - 1].is_alphabetic() || vowelish(chars[i - 1]));
        let piece = if yot { Some("ye") } else { lookup(BGN_BE, l) };
        match piece {
            Some(p) => push_cased(&mut out, &chars, i, 1, p),
            None => out.push(chars[i]),
        }
    }
    out
}

fn bgn_cyrillic(run: &str) -> String {
    match slavic_of(run) {
        Slavic::Ru => super::bgn(run),
        Slavic::Uk => cased(run, |c| lookup(BGN_UK, c)),
        Slavic::Be => bgn_belarusian(run),
        Slavic::Bg => cased(run, |c| lookup(BGN_BG, c)),
        Slavic::Sr => cased(run, |c| lookup(SR_LATN, c)),
        Slavic::Mk => cased(run, |c| lookup(BGN_MK, c)),
        Slavic::Kk => cased(run, |c| lookup(BGN_KK, c)),
    }
}

// ---------------------------------------------------------------- Greek

/// ALA-LC Greek: ē ō for η ω, y for υ (u in the diphthongs), ph th
/// ch for φ θ χ, ng nk nx nch for γ before γ κ ξ χ, h for the
/// rough breathing, rh for ῥ; accents dropped.
const ALA_GREEK: &[(char, &str)] = &[
    ('α', "a"),
    ('β', "b"),
    ('γ', "g"),
    ('δ', "d"),
    ('ε', "e"),
    ('ζ', "z"),
    ('η', "ē"),
    ('θ', "th"),
    ('ι', "i"),
    ('κ', "k"),
    ('λ', "l"),
    ('μ', "m"),
    ('ν', "n"),
    ('ξ', "x"),
    ('ο', "o"),
    ('π', "p"),
    ('ρ', "r"),
    ('σ', "s"),
    ('ς', "s"),
    ('τ', "t"),
    ('υ', "y"),
    ('φ', "ph"),
    ('χ', "ch"),
    ('ψ', "ps"),
    ('ω', "ō"),
    ('ϝ', "w"),
];

fn ala_greek(run: &str) -> String {
    struct Base {
        c: char,
        rough: bool,
        marks: String,
    }
    let mut bases: Vec<Base> = Vec::new();
    for c in run.nfd() {
        if is_combining_mark(c)
            && let Some(b) = bases.last_mut()
        {
            if c == '\u{314}' {
                b.rough = true;
            }
            b.marks.push(c);
            continue;
        }
        bases.push(Base {
            c,
            rough: false,
            marks: String::new(),
        });
    }
    let chars: Vec<char> = bases.iter().map(|b| b.c).collect();
    let mut out = String::with_capacity(run.len());
    let mut i = 0;
    while i < bases.len() {
        let b = &bases[i];
        let l = lower(b.c);
        let Some(letter) = lookup(ALA_GREEK, l) else {
            out.push(b.c);
            out.push_str(&b.marks);
            i += 1;
            continue;
        };
        let nl = bases.get(i + 1).map(|n| lower(n.c));
        let next_rough = bases.get(i + 1).is_some_and(|n| n.rough);
        let (piece, span): (String, usize) = if l == 'γ'
            && matches!(nl, Some('γ' | 'κ' | 'ξ' | 'χ'))
        {
            (
                match nl {
                    Some('γ') => "ng",
                    Some('κ') => "nk",
                    Some('ξ') => "nx",
                    _ => "nch",
                }
                .into(),
                2,
            )
        } else if matches!(l, 'α' | 'ε' | 'η' | 'ο') && nl == Some('υ') && !next_rough {
            (format!("{letter}u"), 2)
        } else if l == 'ρ' && b.rough {
            ("rh".into(), 1)
        } else if b.rough || (matches!(l, 'α' | 'ε' | 'η' | 'ο') && nl == Some('υ') && next_rough)
        {
            // the rough breathing is an h before its vowel (or diphthong)
            let rest = if nl == Some('υ') && next_rough {
                format!("{letter}u")
            } else {
                letter.into()
            };
            let span = if nl == Some('υ') && next_rough { 2 } else { 1 };
            let mut h = String::from("h");
            h.push_str(&rest);
            // case: the capital moves to the h
            if b.c.is_uppercase() {
                let mut s = String::from("H");
                s.push_str(&rest);
                out.push_str(&s);
                i += span;
                continue;
            }
            (h, span)
        } else {
            (letter.into(), 1)
        };
        push_cased(&mut out, &chars, i, span, &piece);
        i += span;
    }
    out.nfc().collect()
}

// ---------------------------------------------------------------- Semitic

const ALA_ARABIC: &[(char, &str)] = &[
    ('ء', "ʼ"),
    ('آ', "ā"),
    ('أ', "ʼ"),
    ('ؤ', "ʼ"),
    ('إ', "ʼ"),
    ('ئ', "ʼ"),
    ('ا', "ā"),
    ('ٱ', "a"),
    ('ب', "b"),
    ('ة', "h"),
    ('ت', "t"),
    ('ث', "th"),
    ('ج', "j"),
    ('ح', "ḥ"),
    ('خ', "kh"),
    ('د', "d"),
    ('ذ', "dh"),
    ('ر', "r"),
    ('ز', "z"),
    ('س', "s"),
    ('ش', "sh"),
    ('ص', "ṣ"),
    ('ض', "ḍ"),
    ('ط', "ṭ"),
    ('ظ', "ẕ"),
    ('ع', "ʻ"),
    ('غ', "gh"),
    ('ف', "f"),
    ('ق', "q"),
    ('ك', "k"),
    ('ل', "l"),
    ('م', "m"),
    ('ن', "n"),
    ('ه', "h"),
    ('و', "w"),
    ('ى', "á"),
    ('ي', "y"),
    ('پ', "p"),
    ('چ', "ch"),
    ('ژ', "zh"),
    ('گ', "g"),
    ('ک', "k"),
    ('ی', "y"),
    ('ڤ', "v"),
    ('\u{64e}', "a"),
    ('\u{650}', "i"),
    ('\u{64f}', "u"),
    ('\u{652}', ""),
    ('\u{64b}', "an"),
    ('\u{64d}', "in"),
    ('\u{64c}', "un"),
    ('\u{670}', "á"),
    ('ـ', ""),
];

const BGN_ARABIC: &[(char, &str)] = &[
    ('ء', "ʼ"),
    ('آ', "ā"),
    ('أ', "ʼ"),
    ('ؤ', "ʼ"),
    ('إ', "ʼ"),
    ('ئ', "ʼ"),
    ('ا', "ā"),
    ('ٱ', "a"),
    ('ب', "b"),
    ('ة', "h"),
    ('ت', "t"),
    ('ث', "th"),
    ('ج', "j"),
    ('ح', "ḩ"),
    ('خ', "kh"),
    ('د', "d"),
    ('ذ', "dh"),
    ('ر', "r"),
    ('ز', "z"),
    ('س', "s"),
    ('ش', "sh"),
    ('ص', "ş"),
    ('ض', "ḑ"),
    ('ط', "ţ"),
    ('ظ', "z̧"),
    ('ع', "ʻ"),
    ('غ', "gh"),
    ('ف', "f"),
    ('ق', "q"),
    ('ك', "k"),
    ('ل', "l"),
    ('م', "m"),
    ('ن', "n"),
    ('ه', "h"),
    ('و', "w"),
    ('ى', "á"),
    ('ي', "y"),
    ('پ', "p"),
    ('چ', "ch"),
    ('ژ', "zh"),
    ('گ', "g"),
    ('ک', "k"),
    ('ی', "y"),
    ('ڤ', "v"),
    ('\u{64e}', "a"),
    ('\u{650}', "i"),
    ('\u{64f}', "u"),
    ('\u{652}', ""),
    ('\u{64b}', "an"),
    ('\u{64d}', "in"),
    ('\u{64c}', "un"),
    ('\u{670}', "á"),
    ('ـ', ""),
];

const ALA_HEBREW: &[(char, &str)] = &[
    ('א', "ʼ"),
    ('ב', "v"),
    ('ג', "g"),
    ('ד', "d"),
    ('ה', "h"),
    ('ו', "v"),
    ('ז', "z"),
    ('ח', "ḥ"),
    ('ט', "ṭ"),
    ('י', "y"),
    ('כ', "kh"),
    ('ך', "kh"),
    ('ל', "l"),
    ('מ', "m"),
    ('ם', "m"),
    ('נ', "n"),
    ('ן', "n"),
    ('ס', "s"),
    ('ע', "ʻ"),
    ('פ', "f"),
    ('ף', "f"),
    ('צ', "ts"),
    ('ץ', "ts"),
    ('ק', "ḳ"),
    ('ר', "r"),
    ('ש', "sh"),
    ('ת', "t"),
    ('\u{5b8}', "a"),
    ('\u{5b7}', "a"),
    ('\u{5b5}', "e"),
    ('\u{5b6}', "e"),
    ('\u{5b4}', "i"),
    ('\u{5b9}', "o"),
    ('\u{5ba}', "o"),
    ('\u{5bb}', "u"),
    ('\u{5b0}', "e"),
    ('\u{5b1}', "e"),
    ('\u{5b2}', "a"),
    ('\u{5b3}', "o"),
    ('\u{5bc}', ""),
    ('\u{5c1}', ""),
    ('\u{5c2}', ""),
    ('\u{5be}', "-"),
    ('\u{5c3}', ":"),
    ('\u{5f3}', "'"),
    ('\u{5f4}', "\""),
];

const BGN_HEBREW: &[(char, &str)] = &[
    ('א', ""),
    ('ב', "v"),
    ('ג', "g"),
    ('ד', "d"),
    ('ה', "h"),
    ('ו', "v"),
    ('ז', "z"),
    ('ח', "ẖ"),
    ('ט', "t"),
    ('י', "y"),
    ('כ', "kh"),
    ('ך', "kh"),
    ('ל', "l"),
    ('מ', "m"),
    ('ם', "m"),
    ('נ', "n"),
    ('ן', "n"),
    ('ס', "s"),
    ('ע', ""),
    ('פ', "f"),
    ('ף', "f"),
    ('צ', "ẕ"),
    ('ץ', "ẕ"),
    ('ק', "q"),
    ('ר', "r"),
    ('ש', "sh"),
    ('ת', "t"),
    ('\u{5b8}', "a"),
    ('\u{5b7}', "a"),
    ('\u{5b5}', "e"),
    ('\u{5b6}', "e"),
    ('\u{5b4}', "i"),
    ('\u{5b9}', "o"),
    ('\u{5ba}', "o"),
    ('\u{5bb}', "u"),
    ('\u{5b0}', ""),
    ('\u{5b1}', "e"),
    ('\u{5b2}', "a"),
    ('\u{5b3}', "o"),
    ('\u{5bc}', ""),
    ('\u{5c1}', ""),
    ('\u{5c2}', ""),
    ('\u{5be}', "-"),
    ('\u{5c3}', ":"),
    ('\u{5f3}', "'"),
    ('\u{5f4}', "\""),
];

// ---------------------------------------------------------------- Caucasus

const ALA_ARMENIAN: &[(char, &str)] = &[
    ('ա', "a"),
    ('բ', "b"),
    ('գ', "g"),
    ('դ', "d"),
    ('ե', "e"),
    ('զ', "z"),
    ('է', "ē"),
    ('ը', "ě"),
    ('թ', "tʻ"),
    ('ժ', "zh"),
    ('ի', "i"),
    ('լ', "l"),
    ('խ', "kh"),
    ('ծ', "ts"),
    ('կ', "k"),
    ('հ', "h"),
    ('ձ', "dz"),
    ('ղ', "gh"),
    ('ճ', "ch"),
    ('մ', "m"),
    ('յ', "y"),
    ('ն', "n"),
    ('շ', "sh"),
    ('ո', "o"),
    ('չ', "chʻ"),
    ('պ', "p"),
    ('ջ', "j"),
    ('ռ', "ṛ"),
    ('ս', "s"),
    ('վ', "v"),
    ('տ', "t"),
    ('ր', "r"),
    ('ց', "tsʻ"),
    ('ւ', "w"),
    ('փ', "pʻ"),
    ('ք', "kʻ"),
    ('օ', "ō"),
    ('ֆ', "f"),
    ('և', "ew"),
];

const BGN_ARMENIAN: &[(char, &str)] = &[
    ('ա', "a"),
    ('բ', "b"),
    ('գ', "g"),
    ('դ', "d"),
    ('ե', "e"),
    ('զ', "z"),
    ('է', "e"),
    ('ը', "y"),
    ('թ', "tʼ"),
    ('ժ', "zh"),
    ('ի', "i"),
    ('լ', "l"),
    ('խ', "kh"),
    ('ծ', "ts"),
    ('կ', "k"),
    ('հ', "h"),
    ('ձ', "dz"),
    ('ղ', "gh"),
    ('ճ', "ch"),
    ('մ', "m"),
    ('յ', "y"),
    ('ն', "n"),
    ('շ', "sh"),
    ('ո', "o"),
    ('չ', "chʼ"),
    ('պ', "p"),
    ('ջ', "j"),
    ('ռ', "r"),
    ('ս', "s"),
    ('վ', "v"),
    ('տ', "t"),
    ('ր', "r"),
    ('ց', "tsʼ"),
    ('ւ', "v"),
    ('փ', "pʼ"),
    ('ք', "kʼ"),
    ('օ', "o"),
    ('ֆ', "f"),
    ('և', "yev"),
];

/// BGN/PCGN Armenian: ե and ո are ye and vo at the start of a word.
fn bgn_armenian(run: &str) -> String {
    let chars: Vec<char> = run.chars().collect();
    let mut out = String::with_capacity(run.len());
    for i in 0..chars.len() {
        let l = lower(chars[i]);
        let initial = i == 0 || !chars[i - 1].is_alphabetic();
        let piece = match (l, initial) {
            ('ե', true) => Some("ye"),
            ('ո', true) => Some("vo"),
            ('և', true) => Some("yev"),
            ('և', false) => Some("ev"),
            _ => lookup(BGN_ARMENIAN, l),
        };
        match piece {
            Some(p) => push_cased(&mut out, &chars, i, 1, p),
            None => out.push(chars[i]),
        }
    }
    out
}

const ALA_GEORGIAN: &[(char, &str)] = &[
    ('ა', "a"),
    ('ბ', "b"),
    ('გ', "g"),
    ('დ', "d"),
    ('ე', "e"),
    ('ვ', "v"),
    ('ზ', "z"),
    ('თ', "t"),
    ('ი', "i"),
    ('კ', "ḳ"),
    ('ლ', "l"),
    ('მ', "m"),
    ('ნ', "n"),
    ('ო', "o"),
    ('პ', "p̣"),
    ('ჟ', "zh"),
    ('რ', "r"),
    ('ს', "s"),
    ('ტ', "ṭ"),
    ('უ', "u"),
    ('ფ', "p"),
    ('ქ', "k"),
    ('ღ', "gh"),
    ('ყ', "q̇"),
    ('შ', "sh"),
    ('ჩ', "ch"),
    ('ც', "ts"),
    ('ძ', "dz"),
    ('წ', "tṣ"),
    ('ჭ', "cḥ"),
    ('ხ', "kh"),
    ('ჯ', "j"),
    ('ჰ', "h"),
];

const BGN_GEORGIAN: &[(char, &str)] = &[
    ('ა', "a"),
    ('ბ', "b"),
    ('გ', "g"),
    ('დ', "d"),
    ('ე', "e"),
    ('ვ', "v"),
    ('ზ', "z"),
    ('თ', "tʼ"),
    ('ი', "i"),
    ('კ', "k"),
    ('ლ', "l"),
    ('მ', "m"),
    ('ნ', "n"),
    ('ო', "o"),
    ('პ', "p"),
    ('ჟ', "zh"),
    ('რ', "r"),
    ('ს', "s"),
    ('ტ', "t"),
    ('უ', "u"),
    ('ფ', "pʼ"),
    ('ქ', "kʼ"),
    ('ღ', "gh"),
    ('ყ', "q"),
    ('შ', "sh"),
    ('ჩ', "chʼ"),
    ('ც', "tsʼ"),
    ('ძ', "dz"),
    ('წ', "ts"),
    ('ჭ', "ch"),
    ('ხ', "kh"),
    ('ჯ', "j"),
    ('ჰ', "h"),
];

// ---------------------------------------------------------------- Thai

/// ALA-LC Thai, letter by letter: the prefixed vowels (เ แ โ ใ ไ)
/// follow their consonant, the tone marks go.
const ALA_THAI: &[(char, &str)] = &[
    ('ก', "k"),
    ('ข', "kh"),
    ('ฃ', "kh"),
    ('ค', "kh"),
    ('ฅ', "kh"),
    ('ฆ', "kh"),
    ('ง', "ng"),
    ('จ', "čh"),
    ('ฉ', "ch"),
    ('ช', "ch"),
    ('ซ', "s"),
    ('ฌ', "ch"),
    ('ญ', "y"),
    ('ฎ', "d"),
    ('ฏ', "t"),
    ('ฐ', "th"),
    ('ฑ', "th"),
    ('ฒ', "th"),
    ('ณ', "n"),
    ('ด', "d"),
    ('ต', "t"),
    ('ถ', "th"),
    ('ท', "th"),
    ('ธ', "th"),
    ('น', "n"),
    ('บ', "b"),
    ('ป', "p"),
    ('ผ', "ph"),
    ('ฝ', "f"),
    ('พ', "ph"),
    ('ฟ', "f"),
    ('ภ', "ph"),
    ('ม', "m"),
    ('ย', "y"),
    ('ร', "r"),
    ('ฤ', "rư"),
    ('ล', "l"),
    ('ฦ', "lư"),
    ('ว', "w"),
    ('ศ', "s"),
    ('ษ', "s"),
    ('ส', "s"),
    ('ห', "h"),
    ('ฬ', "l"),
    ('อ', "ʻ"),
    ('ฮ', "h"),
    ('ะ', "a"),
    ('\u{e31}', "a"),
    ('า', "ā"),
    ('ำ', "am"),
    ('\u{e34}', "i"),
    ('\u{e35}', "ī"),
    ('\u{e36}', "ư"),
    ('\u{e37}', "ư̄"),
    ('\u{e38}', "u"),
    ('\u{e39}', "ū"),
    ('เ', "ē"),
    ('แ', "æ"),
    ('โ', "ō"),
    ('ใ', "ai"),
    ('ไ', "ai"),
    ('ๅ', ""),
    ('\u{e47}', ""),
    ('\u{e48}', ""),
    ('\u{e49}', ""),
    ('\u{e4a}', ""),
    ('\u{e4b}', ""),
    ('\u{e4c}', ""),
    ('\u{e4d}', "ṁ"),
    ('ๆ', ""),
    ('๐', "0"),
    ('๑', "1"),
    ('๒', "2"),
    ('๓', "3"),
    ('๔', "4"),
    ('๕', "5"),
    ('๖', "6"),
    ('๗', "7"),
    ('๘', "8"),
    ('๙', "9"),
];

fn ala_thai(run: &str) -> String {
    let chars: Vec<char> = run.chars().collect();
    let mut out = String::with_capacity(run.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if matches!(c, 'เ' | 'แ' | 'โ' | 'ใ' | 'ไ')
            && let Some(cons) = chars.get(i + 1).and_then(|n| lookup(ALA_THAI, *n))
        {
            out.push_str(cons);
            out.push_str(lookup(ALA_THAI, c).unwrap_or(""));
            i += 2;
            continue;
        }
        match lookup(ALA_THAI, c) {
            Some(m) => out.push_str(m),
            None => out.push(c),
        }
        i += 1;
    }
    out
}

// ---------------------------------------------------------------- families

pub(super) fn ala(text: &str) -> String {
    by_script(text, |run, script| match script {
        Script::Cyrillic => ala_cyrillic(run),
        Script::Greek => ala_greek(run),
        Script::Arabic => arabic_with(run, ALA_ARABIC),
        Script::Hebrew => hebrew_with(run, ALA_HEBREW, "ś", false),
        Script::Armenian => cased(run, |c| lookup(ALA_ARMENIAN, c)),
        Script::Georgian => plain(run, |c| lookup(ALA_GEORGIAN, c)),
        Script::Devanagari => super::indic::iast(run),
        Script::Thai => ala_thai(run),
        Script::Kana => super::kana::hepburn(run),
        Script::Hangul => super::hangul::mccune_reischauer(run),
        Script::Other => run.to_string(),
    })
}

pub(super) fn bgn_family(text: &str) -> String {
    by_script(text, |run, script| match script {
        Script::Cyrillic => bgn_cyrillic(run),
        Script::Greek => elot(run),
        Script::Arabic => arabic_with(run, BGN_ARABIC),
        Script::Hebrew => hebrew_with(run, BGN_HEBREW, "s", false),
        Script::Armenian => bgn_armenian(run),
        Script::Georgian => plain(run, |c| lookup(BGN_GEORGIAN, c)),
        _ => run.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn ala_lc_by_script() {
        assert_eq!(translit("Москва", "ala").unwrap(), "Moskva");
        assert_eq!(translit("Київ Їжак", "ala").unwrap(), "Kyïv Ïzhak");
        assert_eq!(translit("Търново Щука", "ala").unwrap(), "Tŭrnovo Shtuka");
        assert_eq!(translit("Ђорђе Чачак", "ala").unwrap(), "Đorđe Čačak");
        assert_eq!(
            translit("Ἀχιλλεύς ὁ Ῥόδος ἄγγελος εὐχή", "ala").unwrap(),
            "Achilleus ho Rhodos angelos euchē"
        );
        assert_eq!(translit("Ὅμηρος", "ala").unwrap(), "Homēros");
        assert_eq!(translit("مُحَمَّد", "ala").unwrap(), "muḥammad");
        assert_eq!(translit("שָׁלוֹם", "ala").unwrap(), "shalom");
        assert_eq!(translit("Հայաստան", "ala").unwrap(), "Hayastan");
        assert_eq!(translit("თბილისი", "ala").unwrap(), "tbilisi");
        assert_eq!(translit("कृष्ण", "ala").unwrap(), "kṛṣṇa");
        assert_eq!(translit("เชียงใหม่", "ala").unwrap(), "chēīynghaim");
        assert_eq!(translit("とうきょう", "ala").unwrap(), "tōkyō");
        assert_eq!(translit("한글", "ala").unwrap(), "han'gŭl");
        // mixed scripts, each its own table; Latin passes through
        assert_eq!(
            translit("Москва and Ἀθῆναι", "ala").unwrap(),
            "Moskva and Athēnai"
        );
    }

    #[test]
    fn bgn_pcgn_by_language() {
        assert_eq!(translit("Ёлка Щука", "bgn").unwrap(), "Yëlka Shchuka");
        assert_eq!(
            translit("Київ Запоріжжя", "bgn").unwrap(),
            "Kyyiv Zaporizhzhya"
        );
        assert_eq!(
            translit("Магілёў Ельск", "bgn").unwrap(),
            "Mahilyow Yel’sk".replace('’', "ʼ")
        );
        assert_eq!(translit("Търново Щука", "bgn").unwrap(), "Tŭrnovo Shtuka");
        assert_eq!(translit("Скопје Ѕвезда", "bgn").unwrap(), "Skopje Dzvezda");
        assert_eq!(translit("Қазақстан", "bgn").unwrap(), "Qazaqstan");
        assert_eq!(translit("Ευαγγελία", "bgn").unwrap(), "Evangelia");
        assert_eq!(translit("مُحَمَّد", "bgn").unwrap(), "muḩammad");
        assert_eq!(translit("Երևան Ոսկան", "bgn").unwrap(), "Yerevan Voskan");
        assert_eq!(translit("თბილისი", "bgn").unwrap(), "tʼbilisi");
        // a script the boards' tables do not cover passes through
        assert_eq!(translit("とうきょう", "bgn").unwrap(), "とうきょう");
    }
}
