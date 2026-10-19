//! The national systems of the Cyrillic and Georgian states:
//! Bulgarian (Streamlined System, 2009), Belarusian (2007),
//! Georgian (2002), Kazakh Latin (2021), Mongolian (MNS 5217:2012).

use super::{cased, lookup, lower, plain, push_cased};
use unicode_normalization::UnicodeNormalization;

/// The Streamlined System: ъ is a, й and ь are y, щ is sht; the
/// ending -ия is -ia.
const BULGARIAN: &[(char, &str)] = &[
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
    ('х', "h"),
    ('ц', "ts"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('щ', "sht"),
    ('ъ', "a"),
    ('ь', "y"),
    ('ю', "yu"),
    ('я', "ya"),
];

pub(super) fn bulgarian(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let l = lower(chars[i]);
        // the system's one exception, the country's name
        let name: String = chars[i..].iter().take(8).map(|c| lower(*c)).collect();
        if name == "българия" && chars.get(i + 8).is_none_or(|n| !n.is_alphabetic()) {
            push_cased(&mut out, &chars, i, 8, "bulgaria");
            i += 8;
            continue;
        }
        let word_end = |k: usize| chars.get(k).is_none_or(|n| !n.is_alphabetic());
        if l == 'и' && chars.get(i + 1).is_some_and(|n| lower(*n) == 'я') && word_end(i + 2) {
            push_cased(&mut out, &chars, i, 2, "ia");
            i += 2;
            continue;
        }
        match lookup(BULGARIAN, l) {
            Some(p) => push_cased(&mut out, &chars, i, 1, p),
            None => out.push(chars[i]),
        }
        i += 1;
    }
    out
}

/// The Belarusian national system (Council of Ministers, 2007):
/// е ё ю я are je jo ju ja at the start of a word, after a vowel,
/// an apostrophe or ў, and ie io iu ia after a consonant; the soft
/// sign is an acute on its consonant; the apostrophe is dropped.
const BELARUSIAN: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "h"),
    ('ґ', "g"),
    ('д', "d"),
    ('е', "ie"),
    ('ё', "io"),
    ('ж', "ž"),
    ('з', "z"),
    ('і', "i"),
    ('й', "j"),
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
    ('ў', "ŭ"),
    ('ф', "f"),
    ('х', "ch"),
    ('ц', "c"),
    ('ч', "č"),
    ('ш', "š"),
    ('ы', "y"),
    ('э', "e"),
    ('ю', "iu"),
    ('я', "ia"),
    ('\'', ""),
    ('’', ""),
    ('ʼ', ""),
];

pub(super) fn belarusian(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let vowelish = |c: char| "аеёіоуыэюяўй'’ʼ".contains(lower(c));
    for i in 0..chars.len() {
        let c = chars[i];
        let l = lower(c);
        if l == 'ь' {
            // the acute goes on the consonant just written
            if out.chars().last().is_some_and(|p| p.is_alphabetic()) {
                out.push('\u{301}');
            }
            continue;
        }
        let yotated = matches!(l, 'е' | 'ё' | 'ю' | 'я')
            && (i == 0 || !chars[i - 1].is_alphabetic() || vowelish(chars[i - 1]));
        let piece = match (l, yotated) {
            ('е', true) => Some("je"),
            ('ё', true) => Some("jo"),
            ('ю', true) => Some("ju"),
            ('я', true) => Some("ja"),
            _ => lookup(BELARUSIAN, l),
        };
        match piece {
            Some(p) => push_cased(&mut out, &chars, i, 1, p),
            None => out.push(c),
        }
    }
    out.nfc().collect()
}

/// The Georgian national system (2002): the aspirates plain, the
/// ejectives with an apostrophe.
const GEORGIAN_2002: &[(char, &str)] = &[
    ('ა', "a"),
    ('ბ', "b"),
    ('გ', "g"),
    ('დ', "d"),
    ('ე', "e"),
    ('ვ', "v"),
    ('ზ', "z"),
    ('თ', "t"),
    ('ი', "i"),
    ('კ', "k'"),
    ('ლ', "l"),
    ('მ', "m"),
    ('ნ', "n"),
    ('ო', "o"),
    ('პ', "p'"),
    ('ჟ', "zh"),
    ('რ', "r"),
    ('ს', "s"),
    ('ტ', "t'"),
    ('უ', "u"),
    ('ფ', "p"),
    ('ქ', "k"),
    ('ღ', "gh"),
    ('ყ', "q'"),
    ('შ', "sh"),
    ('ჩ', "ch"),
    ('ც', "ts"),
    ('ძ', "dz"),
    ('წ', "ts'"),
    ('ჭ', "ch'"),
    ('ხ', "kh"),
    ('ჯ', "j"),
    ('ჰ', "h"),
];

pub(super) fn georgian(text: &str) -> String {
    plain(text, |c| lookup(GEORGIAN_2002, c))
}

/// Kazakh in the Latin alphabet of 2021.
const KAZAKH: &[(char, &str)] = &[
    ('а', "a"),
    ('ә', "ä"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('ғ', "ğ"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "io"),
    ('ж', "j"),
    ('з', "z"),
    ('и', "i"),
    ('й', "i"),
    ('к', "k"),
    ('қ', "q"),
    ('л', "l"),
    ('м', "m"),
    ('н', "n"),
    ('ң', "ñ"),
    ('о', "o"),
    ('ө', "ö"),
    ('п', "p"),
    ('р', "r"),
    ('с', "s"),
    ('т', "t"),
    ('у', "u"),
    ('ұ', "ū"),
    ('ү', "ü"),
    ('ф', "f"),
    ('х', "h"),
    ('һ', "h"),
    ('ц', "ts"),
    ('ч', "ch"),
    ('ш', "ş"),
    ('щ', "şç"),
    ('ъ', ""),
    ('ы', "y"),
    ('і', "i"),
    ('ь', ""),
    ('э', "e"),
    ('ю', "iu"),
    ('я', "ia"),
];

pub(super) fn kazakh(text: &str) -> String {
    cased(text, |c| lookup(KAZAKH, c))
}

/// Mongolian, MNS 5217:2012.
const MONGOLIAN: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('е', "ye"),
    ('ё', "yo"),
    ('ж', "j"),
    ('з', "z"),
    ('и', "i"),
    ('й', "i"),
    ('к', "k"),
    ('л', "l"),
    ('м', "m"),
    ('н', "n"),
    ('о', "o"),
    ('ө', "ö"),
    ('п', "p"),
    ('р', "r"),
    ('с', "s"),
    ('т', "t"),
    ('у', "u"),
    ('ү', "ü"),
    ('ф', "f"),
    ('х', "kh"),
    ('ц', "ts"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('щ', "sh"),
    ('ъ', "i"),
    ('ы', "y"),
    ('ь', "i"),
    ('э', "e"),
    ('ю', "yu"),
    ('я', "ya"),
];

pub(super) fn mongolian(text: &str) -> String {
    cased(text, |c| lookup(MONGOLIAN, c))
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn national_systems() {
        assert_eq!(
            translit("България Шумен Търново", "bg").unwrap(),
            "Bulgaria Shumen Tarnovo"
        );
        assert_eq!(
            translit("Беларусь Магілёў Льв", "be").unwrap(),
            "Bielaruś Mahilioŭ Ĺv"
        );
        assert_eq!(translit("Ельск з'езд", "be").unwrap(), "Jeĺsk zjezd");
        assert_eq!(
            translit("თბილისი ქუთაისი წყალი", "ka").unwrap(),
            "tbilisi kutaisi ts'q'ali"
        );
        assert_eq!(
            translit("Қазақстан Шымкент Әлия", "kk").unwrap(),
            "Qazaqstan Şymkent Äliia"
        );
        assert_eq!(
            translit("Улаанбаатар Өвөрхангай Эрдэнэт", "mn").unwrap(),
            "Ulaanbaatar Övörkhangai Erdenet"
        );
    }
}
