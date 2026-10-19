//! Arabic and Hebrew beyond the ISO tables: DIN 31635 (Arabic and
//! Persian), Buckwalter's ASCII, and the Hebrew Academy's 2006
//! rules for unpointed text.

use super::lookup;

/// DIN 31635:2011, the letters; the Persian four follow.
const DIN: &[(char, &str)] = &[
    ('ء', "ʾ"),
    ('آ', "ʾā"),
    ('أ', "ʾ"),
    ('ؤ', "ʾ"),
    ('إ', "ʾ"),
    ('ئ', "ʾ"),
    ('ا', "ā"),
    ('ٱ', "a"),
    ('ب', "b"),
    ('ة', "h"),
    ('ت', "t"),
    ('ث', "ṯ"),
    ('ج', "ǧ"),
    ('ح', "ḥ"),
    ('خ', "ḫ"),
    ('د', "d"),
    ('ذ', "ḏ"),
    ('ر', "r"),
    ('ز', "z"),
    ('س', "s"),
    ('ش', "š"),
    ('ص', "ṣ"),
    ('ض', "ḍ"),
    ('ط', "ṭ"),
    ('ظ', "ẓ"),
    ('ع', "ʿ"),
    ('غ', "ġ"),
    ('ف', "f"),
    ('ق', "q"),
    ('ك', "k"),
    ('ل', "l"),
    ('م', "m"),
    ('ن', "n"),
    ('ه', "h"),
    ('و', "w"),
    ('ى', "ā"),
    ('ي', "y"),
    ('پ', "p"),
    ('چ', "č"),
    ('ژ', "ž"),
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
    ('\u{670}', "ā"),
    ('ـ', ""),
    ('،', ","),
    ('؛', ";"),
    ('؟', "?"),
    ('٠', "0"),
    ('١', "1"),
    ('٢', "2"),
    ('٣', "3"),
    ('٤', "4"),
    ('٥', "5"),
    ('٦', "6"),
    ('٧', "7"),
    ('٨', "8"),
    ('٩', "9"),
];

/// A pointed or unpointed Arabic text through a letter table with
/// the common rules: shadda doubles its consonant, a short vowel
/// before its matching lengthening letter (fatḥa–alif, kasra–yāʾ,
/// ḍamma–wāw, the letter itself unvowelled) is the long vowel, an
/// alif opening a word with no hamza is its vowel, written a.
pub(super) fn arabic_with(text: &str, table: &[(char, &'static str)]) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let is_short = |c: char| matches!(c, '\u{64e}' | '\u{650}' | '\u{64f}');
    let is_sign = |c: char| matches!(c as u32, 0x064B..=0x0652 | 0x0670);
    let is_letter = |c: char| matches!(c as u32, 0x0621..=0x064A | 0x066E..=0x06D3);
    let mut last: Option<(&str, usize)> = None;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\u{651}' {
            if let Some((piece, at)) = last {
                out.insert_str(at, piece);
                last = Some((piece, at + piece.len()));
            }
            i += 1;
            continue;
        }
        if is_short(c) {
            // a long vowel when the lengthening letter follows unvowelled
            let next = chars.get(i + 1).copied();
            let after = chars.get(i + 2).copied();
            let lengthens = matches!(
                (c, next),
                ('\u{64e}', Some('ا')) | ('\u{650}', Some('ي' | 'ی')) | ('\u{64f}', Some('و'))
            ) && after.is_none_or(|a| !is_short(a) && a != '\u{651}');
            if lengthens {
                out.push(match c {
                    '\u{64e}' => 'ā',
                    '\u{650}' => 'ī',
                    _ => 'ū',
                });
                last = None;
                i += 2;
                continue;
            }
        }
        if c == 'ا' && (i == 0 || !is_letter(chars[i - 1]) && !is_sign(chars[i - 1])) {
            // a word opens with alif: the hamzat al-waṣl's vowel, a
            // unless a vowel sign says otherwise
            if !chars.get(i + 1).is_some_and(|n| is_short(*n)) {
                out.push('a');
            }
            last = None;
            i += 1;
            continue;
        }
        match lookup(table, c) {
            Some(m) => {
                out.push_str(m);
                if !is_sign(c) {
                    last = Some((m, out.len()));
                }
            }
            None => {
                out.push(c);
                last = None;
            }
        }
        i += 1;
    }
    out
}

pub(super) fn din31635(text: &str) -> String {
    arabic_with(text, DIN)
}

/// Buckwalter transliteration: one ASCII character per Arabic
/// letter or sign, reversible; the Persian letters as the
/// extended set has them.
const BUCKWALTER: &[(char, &str)] = &[
    ('ء', "'"),
    ('آ', "|"),
    ('أ', ">"),
    ('ؤ', "&"),
    ('إ', "<"),
    ('ئ', "}"),
    ('ا', "A"),
    ('ب', "b"),
    ('ة', "p"),
    ('ت', "t"),
    ('ث', "v"),
    ('ج', "j"),
    ('ح', "H"),
    ('خ', "x"),
    ('د', "d"),
    ('ذ', "*"),
    ('ر', "r"),
    ('ز', "z"),
    ('س', "s"),
    ('ش', "$"),
    ('ص', "S"),
    ('ض', "D"),
    ('ط', "T"),
    ('ظ', "Z"),
    ('ع', "E"),
    ('غ', "g"),
    ('ف', "f"),
    ('ق', "q"),
    ('ك', "k"),
    ('ل', "l"),
    ('م', "m"),
    ('ن', "n"),
    ('ه', "h"),
    ('و', "w"),
    ('ى', "Y"),
    ('ي', "y"),
    ('ٱ', "{"),
    ('پ', "P"),
    ('چ', "J"),
    ('ژ', "R"),
    ('گ', "G"),
    ('ک', "K"),
    ('ی', "Y"),
    ('ڤ', "V"),
    ('\u{64e}', "a"),
    ('\u{64f}', "u"),
    ('\u{650}', "i"),
    ('\u{64b}', "F"),
    ('\u{64c}', "N"),
    ('\u{64d}', "K"),
    ('\u{651}', "~"),
    ('\u{652}', "o"),
    ('\u{670}', "`"),
    ('ـ', "_"),
];

pub(super) fn buckwalter(text: &str) -> String {
    // the shadda before the vowel it carries, as Buckwalter writes
    // it, whatever order normalization left the signs in
    let chars: Vec<char> = text.chars().collect();
    let mut reordered = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if matches!(c as u32, 0x064B..=0x0650) && chars.get(i + 1) == Some(&'\u{651}') {
            reordered.push('\u{651}');
            reordered.push(c);
            i += 2;
            continue;
        }
        reordered.push(c);
        i += 1;
    }
    super::plain(&reordered, |c| lookup(BUCKWALTER, c))
}

/// The Hebrew Academy's 2006 rules for unpointed text (the road
/// sign system): ב כ פ are b k p at the start of a word or with a
/// dagesh and v kh f elsewhere, ח is ch, צ is ts, ש is sh (s with
/// a sin dot), א and ע are silent, a final ה is silent; points,
/// where written, give their vowels.
const HEBREW_2006: &[(char, &str)] = &[
    ('א', ""),
    ('ב', "v"),
    ('ג', "g"),
    ('ד', "d"),
    ('ה', "h"),
    ('ו', "v"),
    ('ז', "z"),
    ('ח', "ch"),
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
    ('צ', "ts"),
    ('ץ', "ts"),
    ('ק', "k"),
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

pub(super) fn hebrew_2006(text: &str) -> String {
    hebrew_with(text, HEBREW_2006, "s", true)
}

/// A Hebrew text through a letter table with the shared rules: ב כ פ
/// take the table's b k p reading at the start of a word or with a
/// dagesh (the table holds the soft reading), the sin dot gives
/// `sin`, a final ה is silent when `drop_he`, the points give
/// their vowels.
pub(super) fn hebrew_with(
    text: &str,
    table: &[(char, &'static str)],
    sin: &str,
    drop_he: bool,
) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let is_point = |c: char| matches!(c as u32, 0x05B0..=0x05C7);
    let is_letter = |c: char| matches!(c as u32, 0x05D0..=0x05EA);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if !is_letter(c) {
            match lookup(table, c) {
                Some(m) => out.push_str(m),
                None => out.push(c),
            }
            i += 1;
            continue;
        }
        // the letter's points, as NFC leaves them
        let mut j = i + 1;
        let mut dagesh = false;
        let mut sin_dot = false;
        let mut points: Vec<char> = Vec::new();
        while j < chars.len() && is_point(chars[j]) {
            match chars[j] {
                '\u{5bc}' => dagesh = true,
                '\u{5c2}' => sin_dot = true,
                p => points.push(p),
            }
            j += 1;
        }
        let initial = i == 0 || !is_letter(chars[i - 1]) && !is_point(chars[i - 1]);
        let final_ = !chars.get(j).is_some_and(|n| is_letter(*n));
        let piece = match c {
            'ב' if initial || dagesh => "b",
            'כ' if initial || dagesh => "k",
            'פ' if initial || dagesh => "p",
            'ש' if sin_dot => sin,
            'ה' if drop_he && final_ && !dagesh && points.is_empty() => "",
            'ו' if points.iter().any(|p| matches!(p, '\u{5b9}' | '\u{5ba}')) => {
                points.retain(|p| !matches!(p, '\u{5b9}' | '\u{5ba}'));
                "o"
            }
            'ו' if dagesh && points.is_empty() => "u",
            _ => lookup(table, c).unwrap_or(""),
        };
        out.push_str(piece);
        for p in points {
            if let Some(m) = lookup(table, p) {
                out.push_str(m);
            }
        }
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn din_31635_pointed_and_unpointed() {
        assert_eq!(translit("كِتَابٌ", "din31635").unwrap(), "kitābun");
        assert_eq!(translit("مُحَمَّد", "din31635").unwrap(), "muḥammad");
        assert_eq!(translit("القاهرة", "din31635").unwrap(), "alqāhrh");
        assert_eq!(translit("شكرا", "din31635").unwrap(), "škrā");
        assert_eq!(translit("پژوهش", "din31635").unwrap(), "pžwhš");
    }

    #[test]
    fn buckwalter_is_ascii_and_one_to_one() {
        assert_eq!(translit("مُحَمَّد", "buckwalter").unwrap(), "muHam~ad");
        assert_eq!(translit("الشَّمْسُ", "buckwalter").unwrap(), "Al$~amosu");
        assert!(
            translit("ذهب إلى المدرسة", "buckwalter")
                .unwrap()
                .is_ascii()
        );
    }

    #[test]
    fn hebrew_road_signs() {
        assert_eq!(translit("חיפה", "heb").unwrap(), "chyf");
        assert_eq!(translit("בית לחם", "heb").unwrap(), "byt lchm");
        assert_eq!(translit("כפר סבא", "heb").unwrap(), "kfr sv");
        assert_eq!(translit("שָׁלוֹם", "heb").unwrap(), "shalom");
        assert_eq!(translit("פֶּתַח תִּקְוָה", "heb").unwrap(), "petach tikva");
    }
}
