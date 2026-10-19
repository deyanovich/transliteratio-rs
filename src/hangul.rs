//! Korean from the jamo: Revised Romanization (2000) and
//! McCune–Reischauer (1939), each syllable block decomposed into
//! its initial, medial and final, with the liaison and the sound
//! changes between syllables the systems write.

const RR_INITIALS: &[&str] = &[
    "g", "kk", "n", "d", "tt", "r", "m", "b", "pp", "s", "ss", "", "j", "jj", "ch", "k", "t", "p",
    "h",
];
const RR_MEDIALS: &[&str] = &[
    "a", "ae", "ya", "yae", "eo", "e", "yeo", "ye", "o", "wa", "wae", "oe", "yo", "u", "wo", "we",
    "wi", "yu", "eu", "ui", "i",
];
const RR_FINALS: &[&str] = &[
    "", "k", "k", "k", "n", "n", "n", "t", "l", "k", "m", "l", "l", "l", "p", "l", "m", "p", "p",
    "t", "t", "ng", "t", "t", "k", "t", "p", "t",
];

const MR_INITIALS: &[&str] = &[
    "k", "kk", "n", "t", "tt", "r", "m", "p", "pp", "s", "ss", "", "ch", "tch", "ch'", "k'", "t'",
    "p'", "h",
];
const MR_MEDIALS: &[&str] = &[
    "a", "ae", "ya", "yae", "ŏ", "e", "yŏ", "ye", "o", "wa", "wae", "oe", "yo", "u", "wŏ", "we",
    "wi", "yu", "ŭ", "ŭi", "i",
];
const MR_FINALS: &[&str] = &[
    "", "k", "k", "k", "n", "n", "n", "t", "l", "k", "m", "p", "l", "l", "p", "l", "m", "p", "p",
    "t", "t", "ng", "t", "t", "k", "t", "p", "t",
];

/// A final's jamo as initials when a vowel follows (liaison):
/// the simple finals move whole, a cluster leaves its first part.
/// Indexed like the finals; (coda kept, initial moved).
const LIAISON: &[(usize, usize)] = &[
    // (coda final index, initial index) — 0 and 11 are "none"
    (0, 11),
    (0, 0),
    (0, 1),
    (1, 9),
    (0, 2),
    (4, 12),
    (4, 18),
    (0, 3),
    (0, 5),
    (8, 0),
    (8, 6),
    (8, 7),
    (8, 9),
    (8, 16),
    (8, 17),
    (8, 18),
    (0, 6),
    (0, 7),
    (17, 9),
    (0, 9),
    (0, 10),
    (21, 11),
    (0, 12),
    (0, 14),
    (0, 15),
    (0, 16),
    (0, 17),
    (0, 18),
];

struct Block {
    initial: usize,
    medial: usize,
    final_: usize,
}

fn block(c: char) -> Option<Block> {
    let code = c as u32;
    if !(0xAC00..=0xD7A3).contains(&code) {
        return None;
    }
    let idx = (code - 0xAC00) as usize;
    Some(Block {
        initial: idx / 588,
        medial: (idx % 588) / 28,
        final_: idx % 28,
    })
}

/// The sound changes between a final and the next initial that the
/// systems write: (final index, initial index) → (coda, onset).
fn assimilate(final_: usize, initial: usize, rr: bool) -> Option<(&'static str, &'static str)> {
    // finals: 1 ㄱ 4 ㄴ 7 ㄷ 8 ㄹ 16 ㅁ 17 ㅂ 19 ㅅ 21 ㅇ 27 ㅎ; initials: 2 ㄴ 5 ㄹ 6 ㅁ
    let n_l_m = |coda: &'static str| match initial {
        2 => Some((coda, "n")),
        5 => Some((coda, "n")),
        6 => Some((coda, "m")),
        _ => None,
    };
    match (final_, initial) {
        (1 | 2 | 3 | 9 | 24, 2 | 5 | 6) => n_l_m("ng"),
        (7 | 19 | 20 | 22 | 23 | 25 | 27, 2 | 5) => Some(("n", "n")),
        (7 | 19 | 20 | 22 | 23 | 25 | 27, 6) => Some(("n", "m")),
        (17 | 18 | 26, 2 | 5) => Some(("m", "n")),
        (17 | 18 | 26, 6) => Some(("m", "m")),
        (16 | 21, 5) => Some((if final_ == 16 { "m" } else { "ng" }, "n")),
        (4, 5) | (8, 2) | (8, 5) => Some(("l", "l")),
        (27, 0) if rr => Some(("", "k")),
        (27, 3) if rr => Some(("", "t")),
        (27, 12) if rr => Some(("", "ch")),
        (27, 0) => Some(("", "k'")),
        (27, 3) => Some(("", "t'")),
        (27, 12) => Some(("", "ch'")),
        _ => None,
    }
}

fn romanize(text: &str, rr: bool) -> String {
    let (initials, medials, finals) = if rr {
        (RR_INITIALS, RR_MEDIALS, RR_FINALS)
    } else {
        (MR_INITIALS, MR_MEDIALS, MR_FINALS)
    };
    let chars: Vec<char> = text.chars().collect();
    let blocks: Vec<Option<Block>> = chars.iter().map(|c| block(*c)).collect();
    let mut out = String::with_capacity(text.len() * 2);
    for i in 0..chars.len() {
        let Some(b) = &blocks[i] else {
            out.push(chars[i]);
            continue;
        };
        let prev = if i > 0 { blocks[i - 1].as_ref() } else { None };
        let next = blocks.get(i + 1).and_then(|n| n.as_ref());
        // the onset: the written initial, or the previous final
        // moved over by liaison, or the assimilated pair's onset
        let mut onset = initials[b.initial].to_string();
        let mut coda = finals[b.final_].to_string();
        if let Some(p) = prev {
            if b.initial == 11 && p.final_ != 0 {
                let (_, moved) = LIAISON[p.final_];
                onset = initials[moved].to_string();
                if !rr && matches!(moved, 0 | 3 | 7 | 12) {
                    onset = voiced(moved);
                }
            } else if let Some((_, on)) = assimilate(p.final_, b.initial, rr) {
                onset = on.to_string();
            } else if !rr
                && matches!(b.initial, 0 | 3 | 7 | 12)
                && (p.final_ == 0 || matches!(p.final_, 4 | 8 | 16 | 21))
            {
                // McCune–Reischauer voices ㄱ ㄷ ㅂ ㅈ between voiced sounds
                onset = voiced(b.initial);
            } else if !rr && b.initial == 5 && p.final_ == 8 {
                onset = "l".into();
            }
            if !rr && p.final_ == 4 && onset == "g" {
                out.push('\'');
            }
            // ng before a plain vowel: yŏng'ŏ (not before y- or w-)
            if !rr
                && p.final_ == 21
                && b.initial == 11
                && !matches!(b.medial, 2 | 3 | 6 | 7 | 9 | 10 | 12 | 14 | 15 | 16 | 17)
            {
                out.push('\'');
            }
        }
        if let Some(n) = next {
            if n.initial == 11 && b.final_ != 0 {
                let (kept, _) = LIAISON[b.final_];
                coda = finals[kept].to_string();
            } else if let Some((co, _)) = assimilate(b.final_, n.initial, rr) {
                coda = co.to_string();
            }
        }
        out.push_str(&onset);
        out.push_str(medials[b.medial]);
        out.push_str(&coda);
    }
    out
}

fn voiced(initial: usize) -> String {
    match initial {
        0 => "g",
        3 => "d",
        7 => "b",
        12 => "j",
        _ => MR_INITIALS[initial],
    }
    .to_string()
}

pub(super) fn revised(text: &str) -> String {
    romanize(text, true)
}

pub(super) fn mccune_reischauer(text: &str) -> String {
    romanize(text, false)
}

#[cfg(test)]
mod tests {
    use crate::transliterate as translit;

    #[test]
    fn revised_romanization() {
        assert_eq!(translit("한글", "rr").unwrap(), "hangeul");
        assert_eq!(
            translit("서울 부산 대구", "rr").unwrap(),
            "seoul busan daegu"
        );
        assert_eq!(
            translit("종로 신라 독립 한라산", "rr").unwrap(),
            "jongno silla dongnip hallasan"
        );
        assert_eq!(
            translit("음악 좋다 백마", "rr").unwrap(),
            "eumak jota baengma"
        );
        assert_eq!(translit("김치 떡볶이", "rr").unwrap(), "gimchi tteokbokki");
    }

    #[test]
    fn mccune_reischauer() {
        assert_eq!(translit("한글", "mr").unwrap(), "han'gŭl");
        assert_eq!(
            translit("서울 부산 대구", "mr").unwrap(),
            "sŏul pusan taegu"
        );
        assert_eq!(
            translit("종로 신라 영어", "mr").unwrap(),
            "chongno silla yŏng'ŏ"
        );
        assert_eq!(translit("평양 김치", "mr").unwrap(), "p'yŏngyang kimch'i");
    }
}
