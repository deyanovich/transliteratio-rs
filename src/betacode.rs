//! TLG Beta Code: Greek in ASCII, the letters as capitals, a `*`
//! before a capital, the marks as `)` `(` `/` `\` `=` `+` `|` after
//! the letter (between the `*` and a capital letter, the iota
//! subscript after it).

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

const LETTERS: &[(char, &str)] = &[
    ('α', "A"),
    ('β', "B"),
    ('γ', "G"),
    ('δ', "D"),
    ('ε', "E"),
    ('ζ', "Z"),
    ('η', "H"),
    ('θ', "Q"),
    ('ι', "I"),
    ('κ', "K"),
    ('λ', "L"),
    ('μ', "M"),
    ('ν', "N"),
    ('ξ', "C"),
    ('ο', "O"),
    ('π', "P"),
    ('ρ', "R"),
    ('σ', "S"),
    ('ς', "S"),
    ('τ', "T"),
    ('υ', "U"),
    ('φ', "F"),
    ('χ', "X"),
    ('ψ', "Y"),
    ('ω', "W"),
    ('ϝ', "V"),
    ('ϛ', "#2"),
    ('ϙ', "#3"),
    ('ϡ', "#5"),
    ('·', ":"),
    (';', ";"),
    ('\u{37e}', ";"),
    ('\u{387}', ":"),
    ('\u{2019}', "'"),
    ('\u{1fbd}', "'"),
];

fn mark(c: char) -> Option<&'static str> {
    Some(match c {
        '\u{313}' => ")",
        '\u{314}' => "(",
        '\u{301}' => "/",
        '\u{300}' => "\\",
        '\u{342}' => "=",
        '\u{308}' => "+",
        '\u{345}' => "|",
        '\u{304}' => "%26",
        '\u{306}' => "%27",
        _ => return None,
    })
}

pub(super) fn betacode(text: &str) -> String {
    let chars: Vec<char> = text.nfd().collect();
    let mut out = String::with_capacity(text.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let l = c.to_lowercase().next().unwrap_or(c);
        let Some(letter) = LETTERS.iter().find(|(k, _)| *k == l).map(|(_, v)| *v) else {
            out.push(c);
            i += 1;
            continue;
        };
        // the marks on this letter, in Beta Code's order: breathing,
        // accent, diaeresis, then the subscript
        let mut marks: Vec<char> = Vec::new();
        let mut j = i + 1;
        while j < chars.len() && is_combining_mark(chars[j]) {
            marks.push(chars[j]);
            j += 1;
        }
        let order = |m: &char| match m {
            '\u{313}' | '\u{314}' => 0,
            '\u{301}' | '\u{300}' | '\u{342}' => 1,
            '\u{308}' => 2,
            '\u{304}' | '\u{306}' => 3,
            _ => 4,
        };
        marks.sort_by_key(order);
        let (above, subscript): (Vec<char>, Vec<char>) =
            marks.iter().partition(|m| **m != '\u{345}');
        let unknown: String = marks.iter().filter(|m| mark(**m).is_none()).collect();
        if c.is_uppercase() {
            out.push('*');
            out.extend(above.iter().filter_map(|m| mark(*m)));
            out.push_str(letter);
        } else {
            out.push_str(letter);
            out.extend(above.iter().filter_map(|m| mark(*m)));
        }
        out.extend(subscript.iter().filter_map(|m| mark(*m)));
        out.push_str(&unknown);
        i = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn homer_in_beta_code() {
        assert_eq!(
            translit("μῆνιν ἄειδε θεὰ Πηληϊάδεω Ἀχιλῆος", "betacode").unwrap(),
            "MH=NIN A)/EIDE QEA\\ *PHLHI+A/DEW *)AXILH=OS"
        );
        assert_eq!(
            translit("ᾤχετο ᾍδης", "betacode").unwrap(),
            "W)/|XETO *(/A|DHS"
        );
        assert_eq!(
            translit("ὁ λόγος· τί;", "betacode").unwrap(),
            "O( LO/GOS: TI/;"
        );
        // Latin text and digits pass through
        assert_eq!(translit("Il. 1.1", "betacode").unwrap(), "Il. 1.1");
    }
}
