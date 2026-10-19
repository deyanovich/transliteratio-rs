//! Transliteration, string to string: the ISO letter tables for the
//! major non-Latin scripts, the national and library systems in use
//! beside them (GOST 7.79, BGN/PCGN, ALA-LC, the passport systems,
//! ELOT 743, DIN 31635, Hepburn, pinyin and Wade–Giles, the Korean
//! systems, TLG Beta Code, …), the Latin-to-ASCII folds, the
//! inverses of the reversible tables, and beside them the accent
//! tables (`deaccent` by language, `strip` for every mark) and the
//! historical spelling tables (`modernize`).
//!
//! Every table is letter-wise with the few context rules its standard
//! has; a character the table does not name passes through, so a text
//! that mixes scripts transliterates its own scripts and keeps the
//! rest. The tables that need a lexicon rather than letters (Chinese
//! characters, kanji, Thai) are not here: asking a letter table over
//! such text is a typed error, [`Error::NeedsLexicon`].
//!
//! ```
//! use transliteratio::{transliterate, deaccent, strip};
//! assert_eq!(transliterate("Щука", "iso9").unwrap(), "Ŝuka");
//! assert_eq!(transliterate("Щука", "gost").unwrap(), "Shhuka");
//! assert_eq!(transliterate("Ευαγγελία", "elot").unwrap(), "Evangelia");
//! assert_eq!(deaccent("élève garçon", "fr").unwrap(), "eleve garcon");
//! assert_eq!(strip("Ἀχιλλεύς"), "Αχιλλευς");
//! ```

mod accents;
mod betacode;
mod families;
mod hangul;
mod indic;
mod inverse;
mod kana;
mod national;
mod semitic;
mod sinitic;

#[cfg(feature = "strip")]
pub use accents::strip;
#[cfg(feature = "deaccent")]
pub use accents::{LANGUAGES, deaccent, language_of};

use unicode_normalization::UnicodeNormalization;

/// The schemes by name; the empty name is the script's own ISO
/// scheme, chosen per code point. The Russian systems answer to
/// their Russian names too (гост, научная, бгн, паспорт).
pub const SCHEMES: &[&str] = &[
    "iso",
    "iso9",
    "iso843",
    "iso9985",
    "iso9984",
    "iso259",
    "iso233",
    "iso15919",
    "iso11940",
    "iso3602",
    "gost",
    "scientific",
    "bgn",
    "passport",
    "ala",
    "slug",
    "uk",
    "sr-latn",
    "sr-cyrl",
    "elot",
    "de",
    "mrz",
    "ascii",
    "iso9-cyrl",
    "gost-cyrl",
    "betacode",
    "din31635",
    "buckwalter",
    "heb",
    "hepburn",
    "pinyin",
    "wade",
    "rr",
    "mr",
    "iast",
    "hunterian",
    "bg",
    "be",
    "ka",
    "kk",
    "mn",
    "rtgs",
];

/// What can go wrong: a table or language nobody defined, or a
/// letter table asked over text that needs a lexicon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// No table of that name; [`TABLES`] lists the codes.
    UnknownTable(String),
    /// The table is letter-wise and the text holds characters that
    /// only a lexicon can read: Chinese characters under `pinyin` and
    /// `wade`, kanji under `hepburn` and `iso3602`, Thai under `rtgs`.
    NeedsLexicon {
        /// The table asked for.
        table: String,
        /// What the text holds: "Chinese characters", "kanji", "Thai".
        text: &'static str,
    },
    /// No accent table for that language; [`LANGUAGES`] lists them.
    UnknownLanguage(String),
    /// No spelling table of that name; [`SPELLINGS`] lists them.
    UnknownSpelling(String),
    /// The table exists but this build left its Cargo feature off.
    FeatureOff {
        /// The table asked for.
        table: String,
        /// The feature that carries it.
        feature: &'static str,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownTable(t) => write!(f, "unknown transliteration table '{t}'"),
            Error::NeedsLexicon { table, text } => {
                write!(
                    f,
                    "'{table}' over {text} needs a lexicon, not a letter table"
                )
            }
            Error::UnknownLanguage(l) => write!(f, "no accent table for language '{l}'"),
            Error::UnknownSpelling(t) => write!(
                f,
                "unknown spelling table '{t}': the tables are ru-1918 (also 'дореформенная', 'старая') and long-s"
            ),
            Error::FeatureOff { table, feature } => {
                write!(
                    f,
                    "table '{table}' is not in this build (feature '{feature}')"
                )
            }
        }
    }
}

impl std::error::Error for Error {}

/// A table's metadata: its code, the standard it follows, the
/// script it reads, and whether an inverse table exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Table {
    /// The name the table answers to.
    pub code: &'static str,
    /// The system, as its standard names it.
    pub name: &'static str,
    /// The script (or family of scripts) the table reads.
    pub script: &'static str,
    /// The code of the table that reads the output back, if any.
    pub inverse: Option<&'static str>,
    /// Whether the table needs a lexicon over some of its script
    /// (true for `rtgs`, which has no letter-wise part).
    pub lexicon_only: bool,
}

/// Every table, in the order the names are listed.
pub const TABLES: &[Table] = &[
    Table {
        code: "iso",
        name: "each script's ISO table",
        script: "any",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso9",
        name: "ISO 9:1995 (GOST 7.79 system A)",
        script: "Cyrillic",
        inverse: Some("iso9-cyrl"),
        lexicon_only: false,
    },
    Table {
        code: "iso843",
        name: "ISO 843:1997 type 1",
        script: "Greek",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso9985",
        name: "ISO 9985:1996",
        script: "Armenian",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso9984",
        name: "ISO 9984:1996",
        script: "Georgian",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso259",
        name: "ISO 259:1984",
        script: "Hebrew",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso233",
        name: "ISO 233:1984",
        script: "Arabic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso15919",
        name: "ISO 15919:2001",
        script: "Devanagari",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso11940",
        name: "ISO 11940:1998",
        script: "Thai",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso3602",
        name: "ISO 3602:1989 (Kunrei-shiki)",
        script: "kana",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "gost",
        name: "GOST 7.79-2000 system B",
        script: "Cyrillic",
        inverse: Some("gost-cyrl"),
        lexicon_only: false,
    },
    Table {
        code: "scientific",
        name: "scholarly (DIN 1460, ISO/R 9:1968)",
        script: "Cyrillic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "bgn",
        name: "BGN/PCGN, by language",
        script: "Cyrillic, Greek, Arabic, Hebrew, Armenian, Georgian",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "passport",
        name: "MVD order of 2020, ICAO Doc 9303",
        script: "Cyrillic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "ala",
        name: "ALA-LC, by script and language",
        script: "Cyrillic, Greek, Arabic, Hebrew, Armenian, Georgian, Devanagari, Thai, kana, hangul",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "slug",
        name: "BGN/PCGN folded to ASCII",
        script: "Cyrillic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "uk",
        name: "Ukrainian national system (2010)",
        script: "Cyrillic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "sr-latn",
        name: "Serbian Cyrillic to Latin",
        script: "Cyrillic",
        inverse: Some("sr-cyrl"),
        lexicon_only: false,
    },
    Table {
        code: "sr-cyrl",
        name: "Serbian Latin to Cyrillic",
        script: "Latin",
        inverse: Some("sr-latn"),
        lexicon_only: false,
    },
    Table {
        code: "elot",
        name: "ELOT 743 (ISO 843 type 2)",
        script: "Greek",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "de",
        name: "DIN 5007-2",
        script: "Latin",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "mrz",
        name: "ICAO Doc 9303 part 3, machine-readable zone",
        script: "Latin",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "ascii",
        name: "neutral fold to ASCII",
        script: "Latin",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iso9-cyrl",
        name: "ISO 9 read back",
        script: "Latin",
        inverse: Some("iso9"),
        lexicon_only: false,
    },
    Table {
        code: "gost-cyrl",
        name: "GOST 7.79 B read back",
        script: "Latin",
        inverse: Some("gost"),
        lexicon_only: false,
    },
    Table {
        code: "betacode",
        name: "TLG Beta Code",
        script: "Greek",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "din31635",
        name: "DIN 31635:2011",
        script: "Arabic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "buckwalter",
        name: "Buckwalter transliteration",
        script: "Arabic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "heb",
        name: "Academy of the Hebrew Language, 2006",
        script: "Hebrew",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "hepburn",
        name: "modified Hepburn",
        script: "kana",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "pinyin",
        name: "Hanyu Pinyin (ISO 7098)",
        script: "bopomofo, pinyin readings",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "wade",
        name: "Wade–Giles",
        script: "bopomofo, pinyin readings",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "rr",
        name: "Revised Romanization of Korean (2000)",
        script: "hangul",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "mr",
        name: "McCune–Reischauer (1939)",
        script: "hangul",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "iast",
        name: "IAST",
        script: "Devanagari",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "hunterian",
        name: "Hunterian",
        script: "Devanagari",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "bg",
        name: "Streamlined System (2009)",
        script: "Cyrillic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "be",
        name: "Belarusian national system (2007)",
        script: "Cyrillic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "ka",
        name: "Georgian national system (2002)",
        script: "Georgian",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "kk",
        name: "Kazakh Latin alphabet (2021)",
        script: "Cyrillic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "mn",
        name: "MNS 5217:2012",
        script: "Cyrillic",
        inverse: None,
        lexicon_only: false,
    },
    Table {
        code: "rtgs",
        name: "Royal Thai General System (1999)",
        script: "Thai",
        inverse: None,
        lexicon_only: true,
    },
];

fn has_ideograph(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c as u32, 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF | 0x20000..=0x3134F)
    })
}

fn has_thai(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c as u32, 0x0E01..=0x0E3A | 0x0E40..=0x0E4E))
}

/// `text` under the table named `table` (`""` or `"iso"` for each
/// script's own ISO table).
pub fn transliterate(text: &str, table: &str) -> Result<String, Error> {
    let scheme = table;
    let scheme = match scheme.to_ascii_lowercase().as_str() {
        "" | "iso" => "iso",
        "iso9" | "iso-9" | "iso 9" | "gost-a" | "гост-а" | "гост а" => "iso9",
        "gost" | "gost-b" | "gost7.79" | "gost-7.79" | "гост" | "гост-б" | "гост б" => {
            "gost"
        }
        "passport" | "паспорт" | "мвд" | "icao" | "мид" => "passport",
        "iso843" | "iso-843" => "iso843",
        "iso9985" | "iso-9985" => "iso9985",
        "iso9984" | "iso-9984" => "iso9984",
        "iso259" | "iso-259" => "iso259",
        "iso233" | "iso-233" => "iso233",
        "iso15919" | "iso-15919" => "iso15919",
        "iso11940" | "iso-11940" => "iso11940",
        "iso3602" | "iso-3602" | "kunrei" => "iso3602",
        "scientific" | "scholarly" | "научная" | "науч" | "научн" => "scientific",
        "bgn" | "bgn-pcgn" | "pcgn" | "bgn/pcgn" | "бгн" => "bgn",
        "ala" | "ala-lc" | "lc" => "ala",
        "slug" => "slug",
        "uk" | "ukr" | "укр" => "uk",
        "sr-latn" | "sr_latn" | "srlatn" => "sr-latn",
        "sr-cyrl" | "sr_cyrl" | "srcyrl" => "sr-cyrl",
        "elot" | "elot743" | "elot-743" => "elot",
        "de" | "din5007" | "din-5007" => "de",
        "mrz" | "icao9303" | "icao-9303" => "mrz",
        "ascii" => "ascii",
        "iso9-cyrl" | "iso9_cyrl" | "iso9cyrl" => "iso9-cyrl",
        "gost-cyrl" | "gost_cyrl" | "gostcyrl" => "gost-cyrl",
        "betacode" | "beta" | "tlg" => "betacode",
        "din31635" | "din-31635" | "din" => "din31635",
        "buckwalter" | "bw" => "buckwalter",
        "heb" | "academy" => "heb",
        "hepburn" => "hepburn",
        "pinyin" | "hanyu" => "pinyin",
        "wade" | "wade-giles" | "wg" => "wade",
        "rr" | "revised" => "rr",
        "mr" | "mccune" | "mccune-reischauer" => "mr",
        "iast" => "iast",
        "hunterian" => "hunterian",
        "bg" | "bul" => "bg",
        "be" | "bel" => "be",
        "ka" | "kat" | "geo" => "ka",
        "kk" | "kaz" => "kk",
        "mn" | "mon" => "mn",
        "rtgs" => "rtgs",
        other => return Err(Error::UnknownTable(other.to_string())),
    };
    // the lexicon tables: a text that needs the readings library is
    // refused whole, so a reader never takes a passed-through
    // character for a table gap
    let needs_library = match scheme {
        "hepburn" | "iso3602" if has_ideograph(text) => Some("kanji"),
        "pinyin" | "wade" if has_ideograph(text) => Some("Chinese characters"),
        "rtgs" if has_thai(text) => Some("Thai"),
        _ => None,
    };
    if let Some(what) = needs_library {
        return Err(Error::NeedsLexicon {
            table: scheme.to_string(),
            text: what,
        });
    }
    let text: String = text.nfc().collect();
    macro_rules! on {
        ($feature:literal, $f:expr) => {
            gated(cfg!(feature = $feature), $feature, scheme, &text, $f)
        };
    }
    match scheme {
        "iso" => Ok(auto(&text)),
        "iso9" => on!("iso9", &|t| cased(t, |c| lookup(ISO9, c))),
        "scientific" => on!("scientific", &|t| cased(t, |c| lookup(SCIENTIFIC, c))),
        "bgn" => on!("bgn", &families::bgn_family),
        "gost" => on!("gost", &gost_b),
        "passport" => on!("passport", &|t| cased(t, |c| lookup(PASSPORT, c))),
        "iso843" => on!("iso843", &greek),
        "iso9985" => on!("iso9985", &|t| cased(t, |c| lookup(ARMENIAN, c))),
        "iso9984" => on!("iso9984", &|t| plain(t, |c| lookup(GEORGIAN, c))),
        "iso259" => on!("iso259", &hebrew),
        "iso233" => on!("iso233", &arabic),
        "iso15919" => on!("iso15919", &devanagari),
        "iso11940" => on!("iso11940", &|t| plain(t, |c| lookup(THAI, c))),
        "iso3602" => on!("iso3602", &kana),
        "ala" => on!("ala", &families::ala),
        "slug" => on!("slug", &slug),
        "uk" => on!("uk", &ukrainian),
        "sr-latn" => on!("sr", &|t| cased(t, |c| lookup(SR_LATN, c))),
        "sr-cyrl" => on!("sr", &sr_cyrl),
        "elot" => on!("elot", &elot),
        "de" => on!("de", &|t| cased(t, |c| lookup(DIN_5007, c))),
        "mrz" => on!("mrz", &mrz),
        "ascii" => on!("ascii", &ascii),
        "iso9-cyrl" => on!("iso9-cyrl", &inverse::iso9_cyrl),
        "gost-cyrl" => on!("gost-cyrl", &inverse::gost_cyrl),
        "betacode" => on!("betacode", &betacode::betacode),
        "din31635" => on!("din31635", &semitic::din31635),
        "buckwalter" => on!("buckwalter", &semitic::buckwalter),
        "heb" => on!("heb", &semitic::hebrew_2006),
        "hepburn" => on!("hepburn", &kana::hepburn),
        "pinyin" => on!("pinyin", &sinitic::pinyin),
        "wade" => on!("wade", &sinitic::wade),
        "rr" => on!("rr", &hangul::revised),
        "mr" => on!("mr", &hangul::mccune_reischauer),
        "iast" => on!("iast", &indic::iast),
        "hunterian" => on!("hunterian", &indic::hunterian),
        "bg" => on!("bg", &national::bulgarian),
        "be" => on!("be", &national::belarusian),
        "ka" => on!("ka", &national::georgian),
        "kk" => on!("kk", &national::kazakh),
        "mn" => on!("mn", &national::mongolian),
        "rtgs" => Ok(text.clone()),
        _ => unreachable!(),
    }
}

/// Whether the Cargo feature of a table is on in this build.
pub fn enabled(feature: &str) -> bool {
    FEATURES.iter().any(|(name, on)| *name == feature && *on)
}

/// Every table feature with its state in this build.
pub const FEATURES: &[(&str, bool)] = &[
    ("iso9", cfg!(feature = "iso9")),
    ("iso843", cfg!(feature = "iso843")),
    ("iso9985", cfg!(feature = "iso9985")),
    ("iso9984", cfg!(feature = "iso9984")),
    ("iso259", cfg!(feature = "iso259")),
    ("iso233", cfg!(feature = "iso233")),
    ("iso15919", cfg!(feature = "iso15919")),
    ("iso11940", cfg!(feature = "iso11940")),
    ("iso3602", cfg!(feature = "iso3602")),
    ("gost", cfg!(feature = "gost")),
    ("scientific", cfg!(feature = "scientific")),
    ("bgn", cfg!(feature = "bgn")),
    ("bgn-ru", cfg!(feature = "bgn-ru")),
    ("bgn-uk", cfg!(feature = "bgn-uk")),
    ("bgn-be", cfg!(feature = "bgn-be")),
    ("bgn-bg", cfg!(feature = "bgn-bg")),
    ("bgn-mk", cfg!(feature = "bgn-mk")),
    ("bgn-kk", cfg!(feature = "bgn-kk")),
    ("bgn-arabic", cfg!(feature = "bgn-arabic")),
    ("bgn-hebrew", cfg!(feature = "bgn-hebrew")),
    ("bgn-armenian", cfg!(feature = "bgn-armenian")),
    ("bgn-georgian", cfg!(feature = "bgn-georgian")),
    ("passport", cfg!(feature = "passport")),
    ("ala", cfg!(feature = "ala")),
    ("ala-ru", cfg!(feature = "ala-ru")),
    ("ala-greek", cfg!(feature = "ala-greek")),
    ("ala-arabic", cfg!(feature = "ala-arabic")),
    ("ala-hebrew", cfg!(feature = "ala-hebrew")),
    ("ala-armenian", cfg!(feature = "ala-armenian")),
    ("ala-georgian", cfg!(feature = "ala-georgian")),
    ("ala-thai", cfg!(feature = "ala-thai")),
    ("slug", cfg!(feature = "slug")),
    ("uk", cfg!(feature = "uk")),
    ("sr", cfg!(feature = "sr")),
    ("elot", cfg!(feature = "elot")),
    ("de", cfg!(feature = "de")),
    ("mrz", cfg!(feature = "mrz")),
    ("ascii", cfg!(feature = "ascii")),
    ("iso9-cyrl", cfg!(feature = "iso9-cyrl")),
    ("gost-cyrl", cfg!(feature = "gost-cyrl")),
    ("betacode", cfg!(feature = "betacode")),
    ("din31635", cfg!(feature = "din31635")),
    ("buckwalter", cfg!(feature = "buckwalter")),
    ("heb", cfg!(feature = "heb")),
    ("hepburn", cfg!(feature = "hepburn")),
    ("pinyin", cfg!(feature = "pinyin")),
    ("wade", cfg!(feature = "wade")),
    ("rr", cfg!(feature = "rr")),
    ("mr", cfg!(feature = "mr")),
    ("iast", cfg!(feature = "iast")),
    ("hunterian", cfg!(feature = "hunterian")),
    ("bg", cfg!(feature = "bg")),
    ("be", cfg!(feature = "be")),
    ("ka", cfg!(feature = "ka")),
    ("kk", cfg!(feature = "kk")),
    ("mn", cfg!(feature = "mn")),
    ("spelling", cfg!(feature = "spelling")),
    ("deaccent", cfg!(feature = "deaccent")),
    ("deaccent-cyrillic", cfg!(feature = "deaccent-cyrillic")),
    ("deaccent-greek", cfg!(feature = "deaccent-greek")),
    ("deaccent-romance", cfg!(feature = "deaccent-romance")),
    ("deaccent-vietnamese", cfg!(feature = "deaccent-vietnamese")),
    ("strip", cfg!(feature = "strip")),
];

/// Run a table when its feature is on, else the typed refusal that
/// names the feature.
fn gated(
    on: bool,
    feature: &'static str,
    table: &str,
    text: &str,
    f: &dyn Fn(&str) -> String,
) -> Result<String, Error> {
    if on {
        Ok(f(text))
    } else {
        Err(Error::FeatureOff {
            table: table.to_string(),
            feature,
        })
    }
}

fn lookup(table: &[(char, &'static str)], c: char) -> Option<&'static str> {
    table.iter().find(|(k, _)| *k == c).map(|(_, v)| *v)
}

/// Apply `map` to the lowercase of each letter, restoring case: an
/// uppercase letter titlecases its output, and an uppercase letter
/// beside another uppercase letter uppercases it whole (ЩУКА → ŜUKA,
/// Щука → Ŝuka).
fn cased(text: &str, map: impl Fn(char) -> Option<&'static str>) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        let lower: char = c.to_lowercase().next().unwrap_or(c);
        match map(lower) {
            Some(m) if c.is_uppercase() => {
                let neighbour_upper = chars.get(i + 1).is_some_and(|n| n.is_uppercase())
                    || (i > 0 && chars[i - 1].is_uppercase());
                if neighbour_upper {
                    out.extend(m.chars().flat_map(char::to_uppercase));
                } else {
                    let mut it = m.chars();
                    if let Some(first) = it.next() {
                        out.extend(first.to_uppercase());
                    }
                    out.push_str(it.as_str());
                }
            }
            Some(m) => out.push_str(m),
            None => out.push(c),
        }
    }
    out
}

/// Caseless scripts: map or pass through.
fn plain(text: &str, map: impl Fn(char) -> Option<&'static str>) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match map(c) {
            Some(m) => out.push_str(m),
            None => out.push(c),
        }
    }
    out
}

/// Each code point under its own script's ISO scheme.
fn auto(text: &str) -> String {
    by_script(text, |run, script| {
        let (feature, f): (&str, &dyn Fn(&str) -> String) = match script {
            Script::Cyrillic => ("iso9", &|r| cased(r, |c| lookup(ISO9, c))),
            Script::Greek => ("iso843", &greek),
            Script::Armenian => ("iso9985", &|r| cased(r, |c| lookup(ARMENIAN, c))),
            Script::Georgian => ("iso9984", &|r| plain(r, |c| lookup(GEORGIAN, c))),
            Script::Hebrew => ("iso259", &hebrew),
            Script::Arabic => ("iso233", &arabic),
            Script::Devanagari => ("iso15919", &devanagari),
            Script::Thai => ("iso11940", &|r| plain(r, |c| lookup(THAI, c))),
            Script::Kana => ("iso3602", &kana),
            Script::Hangul | Script::Other => return run.to_string(),
        };
        // a script whose table this build left out passes through
        if enabled(feature) {
            f(run)
        } else {
            run.to_string()
        }
    })
}

/// The text in runs of one script each (marks and spaces stay with
/// the run they follow), every run through `f`.
fn by_script(text: &str, f: impl Fn(&str, Script) -> String) -> String {
    let mut out = String::with_capacity(text.len());
    let mut run = String::new();
    let mut run_script = script_of(' ');
    let flush = |run: &mut String, script: Script, out: &mut String| {
        if run.is_empty() {
            return;
        }
        out.push_str(&f(run, script));
        run.clear();
    };
    for c in text.chars() {
        let s = script_of(c);
        // combining marks and spaces stay with the run they follow
        let s = if s == Script::Other && (is_mark(c) || c.is_whitespace()) {
            run_script
        } else {
            s
        };
        if s != run_script {
            flush(&mut run, run_script, &mut out);
            run_script = s;
        }
        run.push(c);
    }
    flush(&mut run, run_script, &mut out);
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Script {
    Cyrillic,
    Greek,
    Armenian,
    Georgian,
    Hebrew,
    Arabic,
    Devanagari,
    Thai,
    Kana,
    Hangul,
    Other,
}

fn script_of(c: char) -> Script {
    match c as u32 {
        0x0400..=0x052F | 0x1C80..=0x1C8F | 0xA640..=0xA69F => Script::Cyrillic,
        0x0370..=0x03FF | 0x1F00..=0x1FFF => Script::Greek,
        0x0530..=0x058F | 0xFB13..=0xFB17 => Script::Armenian,
        0x10A0..=0x10FF | 0x2D00..=0x2D2F => Script::Georgian,
        0x0590..=0x05FF | 0xFB1D..=0xFB4F => Script::Hebrew,
        0x0600..=0x06FF | 0x0750..=0x077F | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF => Script::Arabic,
        0x0900..=0x097F | 0xA8E0..=0xA8FF => Script::Devanagari,
        0x0E00..=0x0E7F => Script::Thai,
        0x3040..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9F => Script::Kana,
        0xAC00..=0xD7A3 | 0x1100..=0x11FF | 0x3130..=0x318F => Script::Hangul,
        _ => Script::Other,
    }
}

fn is_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}

// ---------------------------------------------------------------- Cyrillic

/// ISO 9:1995 (= GOST 7.79 system A): one Latin letter per Cyrillic
/// letter, with diacritics, for every language the standard covers.
const ISO9: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "ë"),
    ('ж', "ž"),
    ('з', "z"),
    ('и', "i"),
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
    ('ф', "f"),
    ('х', "h"),
    ('ц', "c"),
    ('ч', "č"),
    ('ш', "š"),
    ('щ', "ŝ"),
    ('ъ', "ʺ"),
    ('ы', "y"),
    ('ь', "ʹ"),
    ('э', "è"),
    ('ю', "û"),
    ('я', "â"),
    // Ukrainian, Belarusian, Rusyn
    ('ґ', "g\u{300}"),
    ('є', "ê"),
    ('і', "ì"),
    ('ї', "ï"),
    ('ў', "ŭ"),
    // Serbian, Macedonian
    ('ђ', "đ"),
    ('ј', "ǰ"),
    ('љ', "l\u{302}"),
    ('њ', "n\u{302}"),
    ('ћ', "ć"),
    ('џ', "d\u{302}"),
    ('ѓ', "ǵ"),
    ('ќ', "ḱ"),
    ('ѕ', "ẑ"),
    // Bulgarian and pre-reform Russian
    ('ѫ', "ǎ"),
    ('ѣ', "ě"),
    ('ѳ', "f\u{300}"),
    ('ѵ', "y\u{300}"),
    // Kazakh, Kyrgyz, Tatar, Bashkir, Mongolian, Tajik, and the like
    ('ә', "a\u{30b}"),
    ('ғ', "ġ"),
    ('қ', "ķ"),
    ('ң', "ņ"),
    ('ө', "ô"),
    ('ұ', "u\u{307}"),
    ('ү', "ù"),
    ('һ', "ḥ"),
    ('ӣ', "ī"),
    ('ӯ', "ū"),
    ('ҳ', "ḩ"),
    ('ҷ', "ç"),
    ('ҝ', "k\u{337}"),
    ('ӑ', "ă"),
    ('ӓ', "ä"),
    ('ӗ', "ĕ"),
    ('ӧ', "ö"),
    ('ӱ', "ü"),
    ('ҕ', "ğ"),
    ('ҙ', "ẑ"),
    ('ҫ', "ş"),
    ('ҷ', "ç"),
    ('ҹ', "ç\u{306}"),
    ('ҭ', "ţ"),
    ('ҳ', "ḩ"),
    ('ӏ', "‡"),
    ('ѐ', "è"),
    ('ѝ', "ì"),
    ('ӂ', "z\u{306}"),
    ('ӌ', "c\u{326}"),
    ('ҁ', "c\u{326}"),
];

/// The scientific (scholarly) system for Russian, «научная
/// транслитерация», the one the linguistic literature uses: ISO 9
/// with x, šč, ju, ja.
const SCIENTIFIC: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "ë"),
    ('ж', "ž"),
    ('з', "z"),
    ('и', "i"),
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
    ('ф', "f"),
    ('х', "x"),
    ('ц', "c"),
    ('ч', "č"),
    ('ш', "š"),
    ('щ', "šč"),
    ('ъ', "ʺ"),
    ('ы', "y"),
    ('ь', "ʹ"),
    ('э', "è"),
    ('ю', "ju"),
    ('я', "ja"),
    ('ѣ', "ě"),
    ('ѳ', "f"),
    ('ѵ', "i"),
    ('і', "i"),
    ('є', "je"),
    ('ї', "ji"),
    ('ґ', "g"),
    ('ў', "ŭ"),
];

/// ГОСТ 7.79-2000 system Б: the ASCII system, what «транслит по
/// ГОСТу» means in practice (system А is ISO 9). ц is cz, or c
/// before е, и, й, ы; ъ and ь are grave accents.
const GOST_B: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "yo"),
    ('ж', "zh"),
    ('з', "z"),
    ('и', "i"),
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
    ('ф', "f"),
    ('х', "x"),
    ('ц', "cz"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('щ', "shh"),
    ('ъ', "``"),
    ('ы', "y`"),
    ('ь', "`"),
    ('э', "e`"),
    ('ю', "yu"),
    ('я', "ya"),
    ('ѣ', "ye"),
    ('ѳ', "fh"),
    ('ѵ', "yh"),
    ('ѫ', "o`"),
    ('ґ', "g`"),
    ('є', "ye"),
    ('і', "i"),
    ('ї', "yi"),
    ('ў', "u`"),
];

fn gost_b(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() * 2);
    for (i, &c) in chars.iter().enumerate() {
        let lower = c.to_lowercase().next().unwrap_or(c);
        let piece = if lower == 'ц'
            && chars
                .get(i + 1)
                .is_some_and(|n| "еийы".contains(n.to_lowercase().next().unwrap_or(*n)))
        {
            Some("c")
        } else {
            lookup(GOST_B, lower)
        };
        match piece {
            Some(m) if c.is_uppercase() => {
                let neighbour_upper = chars.get(i + 1).is_some_and(|n| n.is_uppercase())
                    || (i > 0 && chars[i - 1].is_uppercase());
                if neighbour_upper {
                    out.extend(m.chars().flat_map(char::to_uppercase));
                } else {
                    let mut it = m.chars();
                    if let Some(first) = it.next() {
                        out.extend(first.to_uppercase());
                    }
                    out.push_str(it.as_str());
                }
            }
            Some(m) => out.push_str(m),
            None => out.push(c),
        }
    }
    out
}

/// The passport system (МВД 2014, ICAO Doc 9303): ASCII, no
/// diacritics, ъ is ie, ь is dropped.
const PASSPORT: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "e"),
    ('ж', "zh"),
    ('з', "z"),
    ('и', "i"),
    ('й', "i"),
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
    ('ъ', "ie"),
    ('ы', "y"),
    ('ь', ""),
    ('э', "e"),
    ('ю', "iu"),
    ('я', "ia"),
];

/// BGN/PCGN 1947 for Russian: the Anglo-American practical system.
const BGN: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "ë"),
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
    ('щ', "shch"),
    ('ъ', "ʺ"),
    ('ы', "y"),
    ('ь', "ʹ"),
    ('э', "e"),
    ('ю', "yu"),
    ('я', "ya"),
    ('ѣ', "ye"),
    ('ѳ', "f"),
    ('ѵ', "i"),
    ('і', "i"),
];

/// BGN/PCGN with its one contextual rule: е and ё take a y at the
/// start of a word and after a vowel, й, ъ or ь.
fn bgn(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let vowelish = |c: char| "аеёиоуыэюяйъь".contains(c.to_lowercase().next().unwrap_or(c));
    for (i, &c) in chars.iter().enumerate() {
        let lower = c.to_lowercase().next().unwrap_or(c);
        let yotated = matches!(lower, 'е' | 'ё')
            && (i == 0 || !chars[i - 1].is_alphabetic() || vowelish(chars[i - 1]));
        let piece = match (lower, yotated) {
            ('е', true) => Some("ye"),
            ('ё', true) => Some("yë"),
            _ => lookup(BGN, lower),
        };
        match piece {
            Some(m) if c.is_uppercase() => {
                let neighbour_upper = chars.get(i + 1).is_some_and(|n| n.is_uppercase())
                    || (i > 0 && chars[i - 1].is_uppercase());
                if neighbour_upper {
                    out.extend(m.chars().flat_map(char::to_uppercase));
                } else {
                    let mut it = m.chars();
                    if let Some(first) = it.next() {
                        out.extend(first.to_uppercase());
                    }
                    out.push_str(it.as_str());
                }
            }
            Some(m) => out.push_str(m),
            None => out.push(c),
        }
    }
    out
}

/// Push `piece` for the letter at `i` with its case: an uppercase
/// letter titlecases its output, and uppercases it whole when a
/// cased neighbour (the letter after `span`, else the one before)
/// is uppercase too.
fn push_cased(out: &mut String, chars: &[char], i: usize, span: usize, piece: &str) {
    let c = chars[i];
    if !c.is_uppercase() {
        out.push_str(piece);
        return;
    }
    let after = chars.get(i + span).filter(|n| n.is_alphabetic());
    let before = if i > 0 {
        chars.get(i - 1).filter(|n| n.is_alphabetic())
    } else {
        None
    };
    let whole = match (after, before) {
        (Some(n), _) => n.is_uppercase(),
        (None, Some(p)) => p.is_uppercase(),
        (None, None) => false,
    };
    if whole {
        out.extend(piece.chars().flat_map(char::to_uppercase));
    } else {
        let mut it = piece.chars();
        if let Some(first) = it.next() {
            out.extend(first.to_uppercase());
        }
        out.push_str(it.as_str());
    }
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// ALA-LC romanization for Russian: the Library of Congress
/// tables, with the ties over the two-letter renderings.
const ALA_LC: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('е', "e"),
    ('ё', "ë"),
    ('ж', "zh"),
    ('з', "z"),
    ('и', "i"),
    ('й', "ĭ"),
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
    ('ц', "t\u{361}s"),
    ('ч', "ch"),
    ('ш', "sh"),
    ('щ', "shch"),
    ('ъ', "ʺ"),
    ('ы', "y"),
    ('ь', "ʹ"),
    ('э', "ė"),
    ('ю', "i\u{361}u"),
    ('я', "i\u{361}a"),
    ('ѣ', "i\u{361}e"),
    ('ѳ', "ḟ"),
    ('ѵ', "ẏ"),
    ('і', "ī"),
];

/// The ASCII slug: BGN/PCGN with the primes dropped and the
/// diaereses off.
fn slug(text: &str) -> String {
    let romanized = bgn(text);
    romanized
        .nfd()
        .filter(|c| !matches!(*c, 'ʹ' | 'ʺ') && !unicode_normalization::char::is_combining_mark(*c))
        .nfc()
        .collect()
}

/// The Ukrainian national system (Cabinet resolution 55 of 2010):
/// the yotated letters take a y at the start of a word and an i
/// within, зг is zgh, the apostrophe and the soft sign are dropped.
const UKRAINIAN: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "h"),
    ('ґ', "g"),
    ('д', "d"),
    ('е', "e"),
    ('є', "ie"),
    ('ж', "zh"),
    ('з', "z"),
    ('и', "y"),
    ('і', "i"),
    ('ї', "i"),
    ('й', "i"),
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
    ('ь', ""),
    ('ю', "iu"),
    ('я', "ia"),
    ('\'', ""),
    ('’', ""),
    ('ʼ', ""),
];

fn ukrainian(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let l = lower(c);
        // the apostrophe is dropped, and the letter after it is
        // within its word
        let initial =
            i == 0 || !(chars[i - 1].is_alphabetic() || matches!(chars[i - 1], '\'' | '’' | 'ʼ'));
        let next = chars.get(i + 1).map(|n| lower(*n));
        let (piece, span): (Option<&str>, usize) = match (l, next) {
            ('з', Some('г')) => (Some("zgh"), 2),
            ('є', _) if initial => (Some("ye"), 1),
            ('ї', _) if initial => (Some("yi"), 1),
            ('й', _) if initial => (Some("y"), 1),
            ('ю', _) if initial => (Some("yu"), 1),
            ('я', _) if initial => (Some("ya"), 1),
            _ => (lookup(UKRAINIAN, l), 1),
        };
        match piece {
            Some(p) => push_cased(&mut out, &chars, i, span, p),
            None => out.push(c),
        }
        i += span;
    }
    out
}

/// Serbian Cyrillic to Latin: the two alphabets letter for letter,
/// љ њ џ as the digraphs they are.
const SR_LATN: &[(char, &str)] = &[
    ('а', "a"),
    ('б', "b"),
    ('в', "v"),
    ('г', "g"),
    ('д', "d"),
    ('ђ', "đ"),
    ('е', "e"),
    ('ж', "ž"),
    ('з', "z"),
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
    ('ћ', "ć"),
    ('у', "u"),
    ('ф', "f"),
    ('х', "h"),
    ('ц', "c"),
    ('ч', "č"),
    ('џ', "dž"),
    ('ш', "š"),
];

/// Serbian Latin to Cyrillic: lj, nj and dž are one letter each.
fn sr_cyrl(text: &str) -> String {
    let text: String = text.nfc().collect();
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let l = lower(c);
        let next = chars.get(i + 1).map(|n| lower(*n));
        let (letter, span) = match (l, next) {
            ('l', Some('j')) => ('љ', 2),
            ('n', Some('j')) => ('њ', 2),
            ('d', Some('ž')) => ('џ', 2),
            _ => {
                let single = SR_LATN
                    .iter()
                    .find(|(_, v)| v.chars().count() == 1 && v.starts_with(l));
                match single {
                    Some((k, _)) => (*k, 1),
                    None => {
                        out.push(c);
                        i += 1;
                        continue;
                    }
                }
            }
        };
        if c.is_uppercase() {
            out.extend(letter.to_uppercase());
        } else {
            out.push(letter);
        }
        i += span;
    }
    out
}

/// ELOT 743 (ISO 843 type 2), the transcription: β is v, η is i,
/// ω is o, χ is ch, the diphthongs αυ ευ ηυ are av/ev/iv before a
/// vowel or a voiced consonant and af/ef/if elsewhere, ου is ou, μπ
/// is b at the edges of a word and mp within, γγ γκ γξ γχ are ng gk
/// nx nch; the accents go, a dialytika breaks a diphthong.
const ELOT: &[(char, &str)] = &[
    ('α', "a"),
    ('β', "v"),
    ('γ', "g"),
    ('δ', "d"),
    ('ε', "e"),
    ('ζ', "z"),
    ('η', "i"),
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
    ('φ', "f"),
    ('χ', "ch"),
    ('ψ', "ps"),
    ('ω', "o"),
];

fn elot(text: &str) -> String {
    // one entry per base character: its marks are flags for the
    // Greek letters and kept verbatim for everything else
    struct Base {
        c: char,
        marks: String,
        dialytika: bool,
        accent: bool,
    }
    let mut bases: Vec<Base> = Vec::new();
    for c in text.nfd() {
        if unicode_normalization::char::is_combining_mark(c)
            && let Some(b) = bases.last_mut()
        {
            b.marks.push(c);
            if c == '\u{308}' {
                b.dialytika = true;
            } else {
                b.accent = true;
            }
            continue;
        }
        bases.push(Base {
            c,
            marks: String::new(),
            dialytika: false,
            accent: false,
        });
    }
    let chars: Vec<char> = bases.iter().map(|b| b.c).collect();
    let is_greek = |i: usize| {
        bases
            .get(i)
            .is_some_and(|b| lookup(ELOT, lower(b.c)).is_some())
    };
    let vowel = |c: char| "αεηιουω".contains(c);
    let voiced = |c: char| "βγδζλμνραεηιουω".contains(c);
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bases.len() {
        let b = &bases[i];
        let l = lower(b.c);
        if lookup(ELOT, l).is_none() {
            out.push(b.c);
            out.push_str(&b.marks);
            i += 1;
            continue;
        }
        let next = bases.get(i + 1);
        let nl = next.map(|n| lower(n.c));
        let diphthong =
            |second: char| nl == Some(second) && !next.is_some_and(|n| n.dialytika) && !b.accent;
        let after = bases.get(i + 2).map(|n| lower(n.c));
        let (piece, span): (String, usize) = if matches!(l, 'α' | 'ε' | 'η') && diphthong('υ') {
            let first = lookup(ELOT, l).unwrap_or("");
            let voiced_next = after.is_some_and(voiced);
            (format!("{first}{}", if voiced_next { "v" } else { "f" }), 2)
        } else if l == 'ο' && diphthong('υ') {
            ("ou".into(), 2)
        } else if l == 'γ' && matches!(nl, Some('γ' | 'κ' | 'ξ' | 'χ')) {
            (
                match nl {
                    Some('γ') => "ng",
                    Some('κ') => "gk",
                    Some('ξ') => "nx",
                    _ => "nch",
                }
                .into(),
                2,
            )
        } else if l == 'μ' && nl == Some('π') {
            let initial = i == 0 || !is_greek(i - 1);
            let final_ = !is_greek(i + 2);
            ((if initial || final_ { "b" } else { "mp" }).into(), 2)
        } else {
            (lookup(ELOT, l).unwrap_or("").into(), 1)
        };
        let _ = vowel;
        push_cased(&mut out, &chars, i, span, &piece);
        i += span;
    }
    out
}

// ---------------------------------------------------------------- Latin

/// DIN 5007-2: the German umlauts as digraphs and ß as ss, nothing
/// else.
const DIN_5007: &[(char, &str)] = &[('ä', "ae"), ('ö', "oe"), ('ü', "ue"), ('ß', "ss")];

/// The letters outside ASCII that a mark-stripping fold cannot
/// reach: ligatures and the special letters, decomposed by
/// convention.
const ASCII_LETTERS: &[(char, &str)] = &[
    ('œ', "oe"),
    ('æ', "ae"),
    ('ß', "ss"),
    ('ø', "o"),
    ('ł', "l"),
    ('đ', "d"),
    ('þ', "th"),
    ('ð', "d"),
    ('ı', "i"),
    ('ħ', "h"),
    ('ŧ', "t"),
    ('ĳ', "ij"),
    ('ſ', "s"),
];

/// The neutral fold: marks dropped, the special letters decomposed,
/// case kept.
fn ascii(text: &str) -> String {
    let text: String = text.nfc().collect();
    let letters = cased(&text, |c| lookup(ASCII_LETTERS, c));
    letters
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .nfc()
        .collect()
}

/// ICAO Doc 9303 part 3, the machine-readable zone: the national
/// renderings of the letters with marks (Ä AE, Ö OE, Ü UE, ß SS, Æ
/// AE, Œ OE, Ø OE, Å AA, Þ TH, Ð D, Ł L, Ñ N, Ĳ IJ), every other
/// mark dropped, the whole in capitals as the zone is.
const MRZ: &[(char, &str)] = &[
    ('ä', "ae"),
    ('ö', "oe"),
    ('ü', "ue"),
    ('ß', "ss"),
    ('æ', "ae"),
    ('œ', "oe"),
    ('ø', "oe"),
    ('å', "aa"),
    ('þ', "th"),
    ('ð', "d"),
    ('ł', "l"),
    ('ñ', "n"),
    ('ĳ', "ij"),
    ('ı', "i"),
];

fn mrz(text: &str) -> String {
    let text: String = text.nfc().collect();
    let letters = plain(&text, |c| lookup(MRZ, lower(c)));
    letters
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .nfc()
        .flat_map(char::to_uppercase)
        .collect()
}

// ---------------------------------------------------------------- Greek

/// ISO 843:1997 transliteration: the base letters, with any accent
/// or diaeresis carried over as the combining mark it is.
const GREEK: &[(char, &str)] = &[
    ('α', "a"),
    ('β', "v"),
    ('γ', "g"),
    ('δ', "d"),
    ('ε', "e"),
    ('ζ', "z"),
    ('η', "ī"),
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
    ('φ', "f"),
    ('χ', "ch"),
    ('ψ', "ps"),
    ('ω', "ō"),
    ('ϝ', "w"),
    ('ϛ', "st"),
    ('ϟ', "q"),
    ('ϡ', "ss"),
];

fn greek(text: &str) -> String {
    // decompose so the base letter maps and the marks follow it
    let nfd: String = text.nfd().collect();
    let out = cased(&nfd, |c| lookup(GREEK, c));
    out.nfc().collect()
}

// ---------------------------------------------------------------- Armenian

/// ISO 9985:1996.
const ARMENIAN: &[(char, &str)] = &[
    ('ա', "a"),
    ('բ', "b"),
    ('գ', "g"),
    ('դ', "d"),
    ('ե', "e"),
    ('զ', "z"),
    ('է', "ē"),
    ('ը', "ë"),
    ('թ', "t’"),
    ('ժ', "ž"),
    ('ի', "i"),
    ('լ', "l"),
    ('խ', "x"),
    ('ծ', "ç"),
    ('կ', "k"),
    ('հ', "h"),
    ('ձ', "j"),
    ('ղ', "ġ"),
    ('ճ', "č̣"),
    ('մ', "m"),
    ('յ', "y"),
    ('ն', "n"),
    ('շ', "š"),
    ('ո', "o"),
    ('չ', "č"),
    ('պ', "p"),
    ('ջ', "ǰ"),
    ('ռ', "ṙ"),
    ('ս', "s"),
    ('վ', "v"),
    ('տ', "t"),
    ('ր', "r"),
    ('ց', "c’"),
    ('ւ', "w"),
    ('փ', "p’"),
    ('ք', "k’"),
    ('օ', "ò"),
    ('ֆ', "f"),
    ('և', "ew"),
];

// ---------------------------------------------------------------- Georgian

/// ISO 9984:1996 (Mkhedruli, with the archaic letters).
const GEORGIAN: &[(char, &str)] = &[
    ('ა', "a"),
    ('ბ', "b"),
    ('გ', "g"),
    ('დ', "d"),
    ('ე', "e"),
    ('ვ', "v"),
    ('ზ', "z"),
    ('თ', "t’"),
    ('ი', "i"),
    ('კ', "k"),
    ('ლ', "l"),
    ('მ', "m"),
    ('ნ', "n"),
    ('ო', "o"),
    ('პ', "p"),
    ('ჟ', "ž"),
    ('რ', "r"),
    ('ს', "s"),
    ('ტ', "t"),
    ('უ', "u"),
    ('ფ', "p’"),
    ('ქ', "k’"),
    ('ღ', "ḡ"),
    ('ყ', "q"),
    ('შ', "š"),
    ('ჩ', "č’"),
    ('ც', "c’"),
    ('ძ', "j"),
    ('წ', "c"),
    ('ჭ', "č"),
    ('ხ', "x"),
    ('ჯ', "ǰ"),
    ('ჰ', "h"),
    ('ჱ', "ē"),
    ('ჲ', "y"),
    ('ჳ', "w"),
    ('ჴ', "ẖ"),
    ('ჵ', "ō"),
    ('ჶ', "f"),
];

// ---------------------------------------------------------------- Hebrew

/// ISO 259:1984: the letters, the shin and sin dots, and the vowel
/// points; a dagesh doubles nothing here (the letter table has no
/// spirant pairs), so it passes silently.
const HEBREW: &[(char, &str)] = &[
    ('א', "ʾ"),
    ('ב', "b"),
    ('ג', "g"),
    ('ד', "d"),
    ('ה', "h"),
    ('ו', "w"),
    ('ז', "z"),
    ('ח', "ḥ"),
    ('ט', "ṭ"),
    ('י', "y"),
    ('כ', "k"),
    ('ך', "k"),
    ('ל', "l"),
    ('מ', "m"),
    ('ם', "m"),
    ('נ', "n"),
    ('ן', "n"),
    ('ס', "s"),
    ('ע', "ʿ"),
    ('פ', "p"),
    ('ף', "p"),
    ('צ', "ṣ"),
    ('ץ', "ṣ"),
    ('ק', "q"),
    ('ר', "r"),
    ('ש', "š"),
    ('ת', "t"),
    // points
    ('\u{5b8}', "ā"),
    ('\u{5b7}', "a"),
    ('\u{5b5}', "ē"),
    ('\u{5b6}', "e"),
    ('\u{5b4}', "i"),
    ('\u{5b9}', "ō"),
    ('\u{5ba}', "ō"),
    ('\u{5bb}', "u"),
    ('\u{5b0}', "ə"),
    ('\u{5b1}', "ĕ"),
    ('\u{5b2}', "ă"),
    ('\u{5b3}', "ŏ"),
    ('\u{5bc}', ""),
    ('\u{5be}', "-"),
    ('\u{5c3}', ":"),
    ('\u{5f3}', "'"),
    ('\u{5f4}', "\""),
];

fn hebrew(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let is_point = |c: char| matches!(c as u32, 0x05B0..=0x05C7);
    while i < chars.len() {
        let c = chars[i];
        if c == 'ש' {
            // the shin or sin dot decides, wherever NFC put it among
            // the letter's points; a dotless shin is š
            let mut j = i + 1;
            let mut sin = false;
            let mut dot_at = None;
            while j < chars.len() && is_point(chars[j]) {
                match chars[j] {
                    '\u{5c2}' => {
                        sin = true;
                        dot_at = Some(j);
                    }
                    '\u{5c1}' => dot_at = Some(j),
                    _ => {}
                }
                j += 1;
            }
            out.push(if sin { 'ś' } else { 'š' });
            i += 1;
            // emit the remaining points, skipping the dot
            while i < j {
                if Some(i) != dot_at
                    && let Some(m) = lookup(HEBREW, chars[i])
                {
                    out.push_str(m);
                }
                i += 1;
            }
            continue;
        }
        if c == 'ו' {
            // holam male (וֹ) is ô, shuruq (וּ) is û
            match chars.get(i + 1) {
                Some('\u{5b9}') | Some('\u{5ba}') => {
                    out.push('ô');
                    i += 2;
                    continue;
                }
                Some('\u{5bc}') => {
                    out.push('û');
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        match lookup(HEBREW, c) {
            Some(m) => out.push_str(m),
            None => out.push(c),
        }
        i += 1;
    }
    out
}

// ---------------------------------------------------------------- Arabic

/// ISO 233:1984 with the Persian additions; the short vowels and
/// tanwīn follow their letters, shadda doubles the consonant.
const ARABIC: &[(char, &str)] = &[
    ('ء', "ʾ"),
    ('آ', "ʾā"),
    ('أ', "ʾ"),
    ('ؤ', "ʾ"),
    ('إ', "ʾ"),
    ('ئ', "ʾ"),
    ('ا', "ā"),
    ('ب', "b"),
    ('ة', "ẗ"),
    ('ت', "t"),
    ('ث', "ṯ"),
    ('ج', "ǧ"),
    ('ح', "ḥ"),
    ('خ', "ẖ"),
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
    ('ى', "ỳ"),
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

fn arabic(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    // the last consonant's piece and where it ended, so a shadda —
    // which NFC may place after the vowel sign — doubles the
    // consonant in place, before its vowel
    let mut last: Option<(&str, usize)> = None;
    let is_sign = |c: char| matches!(c as u32, 0x064B..=0x0652 | 0x0670);
    for c in text.chars() {
        if c == '\u{651}' {
            if let Some((piece, at)) = last {
                out.insert_str(at, piece);
                last = Some((piece, at + piece.len()));
            }
            continue;
        }
        match lookup(ARABIC, c) {
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
    }
    out
}

// ---------------------------------------------------------------- Devanagari

/// ISO 15919:2001, Devanagari.
const DEVA_VOWELS: &[(char, &str)] = &[
    ('अ', "a"),
    ('आ', "ā"),
    ('इ', "i"),
    ('ई', "ī"),
    ('उ', "u"),
    ('ऊ', "ū"),
    ('ऋ', "r̥"),
    ('ॠ', "r̥̄"),
    ('ऌ', "l̥"),
    ('ॡ', "l̥̄"),
    ('ऍ', "ê"),
    ('ए', "e"),
    ('ऐ', "ai"),
    ('ऑ', "ô"),
    ('ओ', "o"),
    ('औ', "au"),
];
const DEVA_MATRAS: &[(char, &str)] = &[
    ('ा', "ā"),
    ('ि', "i"),
    ('ी', "ī"),
    ('ु', "u"),
    ('ू', "ū"),
    ('ृ', "r̥"),
    ('ॄ', "r̥̄"),
    ('ॢ', "l̥"),
    ('ॣ', "l̥̄"),
    ('ॅ', "ê"),
    ('े', "e"),
    ('ै', "ai"),
    ('ॉ', "ô"),
    ('ो', "o"),
    ('ौ', "au"),
];
const DEVA_CONSONANTS: &[(char, &str)] = &[
    ('क', "k"),
    ('ख', "kh"),
    ('ग', "g"),
    ('घ', "gh"),
    ('ङ', "ṅ"),
    ('च', "c"),
    ('छ', "ch"),
    ('ज', "j"),
    ('झ', "jh"),
    ('ञ', "ñ"),
    ('ट', "ṭ"),
    ('ठ', "ṭh"),
    ('ड', "ḍ"),
    ('ढ', "ḍh"),
    ('ण', "ṇ"),
    ('त', "t"),
    ('थ', "th"),
    ('द', "d"),
    ('ध', "dh"),
    ('न', "n"),
    ('प', "p"),
    ('फ', "ph"),
    ('ब', "b"),
    ('भ', "bh"),
    ('म', "m"),
    ('य', "y"),
    ('र', "r"),
    ('ल', "l"),
    ('ळ', "ḷ"),
    ('व', "v"),
    ('श', "ś"),
    ('ष', "ṣ"),
    ('स', "s"),
    ('ह', "h"),
];
const DEVA_NUKTA: &[(char, &str)] = &[
    ('क', "q"),
    ('ख', "k͟h"),
    ('ग', "ġ"),
    ('ज', "z"),
    ('ड', "ṛ"),
    ('ढ', "ṛh"),
    ('फ', "f"),
    ('य', "ẏ"),
];
const DEVA_SIGNS: &[(char, &str)] = &[
    ('ं', "ṁ"),
    ('ँ', "m̐"),
    ('ः', "ḥ"),
    ('ऽ', "’"),
    ('।', "."),
    ('॥', ".."),
    ('ॐ', "oṁ"),
    ('०', "0"),
    ('१', "1"),
    ('२', "2"),
    ('३', "3"),
    ('४', "4"),
    ('५', "5"),
    ('६', "6"),
    ('७', "7"),
    ('८', "8"),
    ('९', "9"),
];

fn devanagari(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() * 2);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if let Some(m) = lookup(DEVA_CONSONANTS, c) {
            let mut cons = m;
            let mut j = i + 1;
            if chars.get(j) == Some(&'\u{93c}') {
                cons = lookup(DEVA_NUKTA, c).unwrap_or(m);
                j += 1;
            }
            out.push_str(cons);
            match chars.get(j) {
                Some('\u{94d}') => {
                    // virama: no vowel
                    j += 1;
                }
                Some(&n) if lookup(DEVA_MATRAS, n).is_some() => {
                    out.push_str(lookup(DEVA_MATRAS, n).unwrap());
                    j += 1;
                }
                _ => out.push('a'),
            }
            i = j;
            continue;
        }
        if let Some(m) = lookup(DEVA_VOWELS, c)
            .or_else(|| lookup(DEVA_SIGNS, c))
            .or_else(|| lookup(DEVA_MATRAS, c))
        {
            out.push_str(m);
        } else if c != '\u{94d}' && c != '\u{93c}' {
            out.push(c);
        }
        i += 1;
    }
    out
}

// ---------------------------------------------------------------- Thai

/// ISO 11940:1998: a letter-for-letter transliteration (the standard
/// is reversible by design), letters, vowel signs, tone marks, digits.
const THAI: &[(char, &str)] = &[
    ('ก', "k"),
    ('ข', "k̄h"),
    ('ฃ', "ḳ̄h"),
    ('ค', "kh"),
    ('ฅ', "k̛h"),
    ('ฆ', "ḳh"),
    ('ง', "ng"),
    ('จ', "c"),
    ('ฉ', "c̄h"),
    ('ช', "ch"),
    ('ซ', "s"),
    ('ฌ', "c̣h"),
    ('ญ', "ỵ"),
    ('ฎ', "ḍ"),
    ('ฏ', "ṭ"),
    ('ฐ', "ṭ̄h"),
    ('ฑ', "ṯh"),
    ('ฒ', "t̛h"),
    ('ณ', "ṇ"),
    ('ด', "d"),
    ('ต', "t"),
    ('ถ', "t̄h"),
    ('ท', "th"),
    ('ธ', "ṭh"),
    ('น', "n"),
    ('บ', "b"),
    ('ป', "p"),
    ('ผ', "p̄h"),
    ('ฝ', "f̄"),
    ('พ', "ph"),
    ('ฟ', "f"),
    ('ภ', "p̣h"),
    ('ม', "m"),
    ('ย', "y"),
    ('ร', "r"),
    ('ฤ', "v"),
    ('ล', "l"),
    ('ฦ', "ł"),
    ('ว', "w"),
    ('ศ', "ṣ̄"),
    ('ษ', "s̛̄"),
    ('ส', "s̄"),
    ('ห', "h̄"),
    ('ฬ', "ḷ"),
    ('อ', "x"),
    ('ฮ', "ḥ"),
    ('ะ', "a"),
    ('\u{e31}', "ạ"),
    ('า', "ā"),
    ('ำ', "å"),
    ('\u{e34}', "i"),
    ('\u{e35}', "ī"),
    ('\u{e36}', "ụ"),
    ('\u{e37}', "ụ̄"),
    ('\u{e38}', "u"),
    ('\u{e39}', "ū"),
    ('เ', "e"),
    ('แ', "æ"),
    ('โ', "o"),
    ('ใ', "ı"),
    ('ไ', "ị"),
    ('ๅ', "ǂ"),
    ('\u{e47}', "\u{306}"),
    ('ๆ', "ǂ"),
    ('\u{e3a}', "\u{323}"),
    ('\u{e48}', "\u{300}"),
    ('\u{e49}', "\u{302}"),
    ('\u{e4a}', "\u{301}"),
    ('\u{e4b}', "\u{30c}"),
    ('\u{e4c}', "\u{312}"),
    ('\u{e4d}', "\u{30a}"),
    ('\u{e4e}', "\u{30d}"),
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
    ('฿', "฿"),
];

// ---------------------------------------------------------------- Japanese kana

/// ISO 3602:1989 (Kunrei-shiki), hiragana and katakana: the
/// syllables, the small ya/yu/yo contractions, the sokuon
/// doubling, and the circumflex for a long vowel.
const KANA: &[(char, &str)] = &[
    ('あ', "a"),
    ('い', "i"),
    ('う', "u"),
    ('え', "e"),
    ('お', "o"),
    ('か', "ka"),
    ('き', "ki"),
    ('く', "ku"),
    ('け', "ke"),
    ('こ', "ko"),
    ('さ', "sa"),
    ('し', "si"),
    ('す', "su"),
    ('せ', "se"),
    ('そ', "so"),
    ('た', "ta"),
    ('ち', "ti"),
    ('つ', "tu"),
    ('て', "te"),
    ('と', "to"),
    ('な', "na"),
    ('に', "ni"),
    ('ぬ', "nu"),
    ('ね', "ne"),
    ('の', "no"),
    ('は', "ha"),
    ('ひ', "hi"),
    ('ふ', "hu"),
    ('へ', "he"),
    ('ほ', "ho"),
    ('ま', "ma"),
    ('み', "mi"),
    ('む', "mu"),
    ('め', "me"),
    ('も', "mo"),
    ('や', "ya"),
    ('ゆ', "yu"),
    ('よ', "yo"),
    ('ら', "ra"),
    ('り', "ri"),
    ('る', "ru"),
    ('れ', "re"),
    ('ろ', "ro"),
    ('わ', "wa"),
    ('ゐ', "wi"),
    ('ゑ', "we"),
    ('を', "o"),
    ('ん', "n"),
    ('が', "ga"),
    ('ぎ', "gi"),
    ('ぐ', "gu"),
    ('げ', "ge"),
    ('ご', "go"),
    ('ざ', "za"),
    ('じ', "zi"),
    ('ず', "zu"),
    ('ぜ', "ze"),
    ('ぞ', "zo"),
    ('だ', "da"),
    ('ぢ', "zi"),
    ('づ', "zu"),
    ('で', "de"),
    ('ど', "do"),
    ('ば', "ba"),
    ('び', "bi"),
    ('ぶ', "bu"),
    ('べ', "be"),
    ('ぼ', "bo"),
    ('ぱ', "pa"),
    ('ぴ', "pi"),
    ('ぷ', "pu"),
    ('ぺ', "pe"),
    ('ぽ', "po"),
    ('ぁ', "a"),
    ('ぃ', "i"),
    ('ぅ', "u"),
    ('ぇ', "e"),
    ('ぉ', "o"),
    ('ゔ', "vu"),
];

fn kana(text: &str) -> String {
    // katakana → hiragana, then one table
    let chars: Vec<char> = text
        .chars()
        .map(|c| match c as u32 {
            0x30A1..=0x30F6 => char::from_u32(c as u32 - 0x60).unwrap_or(c),
            _ => c,
        })
        .collect();
    let mut out = String::with_capacity(text.len());
    let mut sokuon = false;
    for &c in &chars {
        match c {
            'っ' => {
                sokuon = true;
                continue;
            }
            'ゃ' | 'ゅ' | 'ょ' => {
                // きゃ → kya: the i of the preceding syllable yields
                if out.ends_with('i') {
                    out.pop();
                }
                out.push_str(match c {
                    'ゃ' => "ya",
                    'ゅ' => "yu",
                    _ => "yo",
                });
                continue;
            }
            'ー' => {
                circumflex(&mut out);
                continue;
            }
            _ => {}
        }
        match lookup(KANA, c) {
            Some(m) => {
                if sokuon {
                    if let Some(first) = m.chars().next() {
                        out.push(first);
                    }
                    sokuon = false;
                }
                // a lone vowel kana after the same vowel lengthens it: おう, おお, うう, ええ, ああ
                let long = m.len() == 1
                    && matches!(
                        (out.chars().last(), m),
                        (Some('o'), "u" | "o")
                            | (Some('u'), "u")
                            | (Some('e'), "e")
                            | (Some('a'), "a")
                    );
                if long {
                    circumflex(&mut out);
                } else {
                    out.push_str(m);
                }
            }
            None => {
                sokuon = false;
                out.push(c);
            }
        }
    }
    out
}

fn circumflex(out: &mut String) {
    let hat = match out.pop() {
        Some('a') => 'â',
        Some('i') => 'î',
        Some('u') => 'û',
        Some('e') => 'ê',
        Some('o') => 'ô',
        Some(other) => {
            out.push(other);
            return;
        }
        None => return,
    };
    out.push(hat);
}

#[cfg(test)]
mod tests {
    use super::transliterate as translit;

    fn t(text: &str, scheme: &str) -> String {
        translit(text, scheme).unwrap()
    }

    #[test]
    fn russian_three_ways() {
        assert_eq!(t("Щука, ёж и объём", "iso9"), "Ŝuka, ëž i obʺëm");
        assert_eq!(t("Щука, ёж и объём", "scientific"), "Ščuka, ëž i obʺëm");
        assert_eq!(t("Щука, ёж и объём", "научная"), "Ščuka, ëž i obʺëm");
        assert_eq!(t("Щука, ёж и объём", "bgn"), "Shchuka, yëzh i obʺyëm");
        assert_eq!(t("Щука, ёж и объём", "гост"), "Shhuka, yozh i ob``yom");
        assert_eq!(t("Цирк, царь, отец", "gost"), "Cirk, czar`, otecz");
        assert_eq!(t("Щука, ёж и объём", "паспорт"), "Shchuka, ezh i obieem");
        assert_eq!(t("Юлия Ильинична", "passport"), "Iuliia Ilinichna");
        assert_eq!(t("Ельцин", "bgn"), "Yel\u{2b9}tsin");
        assert_eq!(t("замо\u{301}к", "scholarly"), "zamo\u{301}k");
        assert_eq!(t("ЩУКА", "iso9"), "ŜUKA");
        assert_eq!(t("Київ", "iso9"), "Kiïv");
        assert_eq!(t("Ђорђе", "iso9"), "Đorđe");
    }

    #[test]
    fn the_other_scripts() {
        assert_eq!(t("Αθήνα", "iso843"), "Athī́na");
        assert_eq!(t("Երևան", "iso9985"), "Erewan");
        assert_eq!(t("თბილისი", "iso9984"), "t’bilisi");
        assert_eq!(t("שָׁלוֹם", "iso259"), "šālôm");
        assert_eq!(t("שִׂמְחָה", "iso259"), "śiməḥāh");
        assert_eq!(t("مُحَمَّد", "iso233"), "muḥammad");
        assert_eq!(t("हिन्दी", "iso15919"), "hindī");
        assert_eq!(t("नमस्ते", "iso15919"), "namaste");
        assert_eq!(t("กข", "iso11940"), "kk̄h");
        assert_eq!(t("とうきょう", "iso3602"), "tôkyô");
        assert_eq!(t("きって しんぶん", "iso3602"), "kitte sinbun");
        assert_eq!(t("トウキョウ", "iso3602"), "tôkyô");
    }

    #[test]
    fn auto_picks_the_script_per_run() {
        assert_eq!(t("Москва and Αθήνα", "iso"), "Moskva and Athī́na");
        assert_eq!(t("plain latin", ""), "plain latin");
        assert!(translit("x", "klingon").is_err());
    }
}

#[cfg(feature = "spelling")]
mod spelling {
    use super::*;
    /// The spelling tables `modernize(table)` knows: a
    /// historical orthography folded to the modern one, letter by
    /// letter, case kept. `ru-1918` is the pre-reform Russian
    /// alphabet (ѣ→е, і→и, ѳ→ф, ѵ→и, the word-final ъ dropped);
    /// `long-s` is early-modern Latin typography (ſ→s, the ff/fi/fl/
    /// ffi/ffl/ft/st ligatures opened).
    pub const SPELLINGS: &[&str] = &["ru-1918", "long-s"];

    /// Fold `text` from a historical spelling to the modern one under
    /// `table`. An unknown table is an error.
    pub fn modernize(text: &str, table: &str) -> Result<String, Error> {
        let table = match table.to_ascii_lowercase().as_str() {
            "ru-1918"
            | "ru1918"
            | "pre-1918"
            | "pre1918"
            | "дореформенная"
            | "дореф"
            | "старая" => "ru-1918",
            "long-s" | "longs" | "long_s" | "early-modern" | "ligatures" => "long-s",
            other => {
                return Err(Error::UnknownSpelling(other.to_string()));
            }
        };
        let text: String = text.nfc().collect();
        Ok(match table {
            "ru-1918" => ru_1918(&text),
            "long-s" => long_s(&text),
            _ => unreachable!(),
        })
    }

    /// The letters the 1918 reform removed, to their replacements.
    const RU_1918: &[(char, &str)] = &[
        ('ѣ', "е"),
        ('Ѣ', "Е"),
        ('і', "и"),
        ('І', "И"),
        ('ѳ', "ф"),
        ('Ѳ', "Ф"),
        ('ѵ', "и"),
        ('Ѵ', "И"),
    ];

    fn ru_1918(text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::with_capacity(text.len());
        for (i, &c) in chars.iter().enumerate() {
            if let Some((_, m)) = RU_1918.iter().find(|(k, _)| *k == c) {
                out.push_str(m);
                continue;
            }
            // The hard sign at the end of a word — before a non-letter
            // or the end — goes; inside a word (объ-) it stays.
            if (c == 'ъ' || c == 'Ъ')
                && i > 0
                && chars[i - 1].is_alphabetic()
                && !chars.get(i + 1).is_some_and(|n| n.is_alphabetic())
            {
                continue;
            }
            out.push(c);
        }
        out
    }

    /// Early-modern Latin typography: the long s and the typographic
    /// ligatures, opened.
    const LONG_S: &[(char, &str)] = &[
        ('ſ', "s"),
        ('ﬀ', "ff"),
        ('ﬁ', "fi"),
        ('ﬂ', "fl"),
        ('ﬃ', "ffi"),
        ('ﬄ', "ffl"),
        ('ﬅ', "ft"),
        ('ﬆ', "st"),
    ];

    fn long_s(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        for c in text.chars() {
            match LONG_S.iter().find(|(k, _)| *k == c) {
                Some((_, m)) => out.push_str(m),
                None => out.push(c),
            }
        }
        out
    }
}

#[cfg(feature = "spelling")]
pub use spelling::{SPELLINGS, modernize};

#[cfg(all(test, feature = "spelling"))]
mod spelling_tests {
    use super::{modernize, transliterate as translit};

    #[test]
    fn ru_1918_folds_the_removed_letters_and_the_final_hard_sign() {
        assert_eq!(
            modernize("Повѣсти Бѣлкина", "ru-1918").unwrap(),
            "Повести Белкина"
        );
        assert_eq!(modernize("міръ и миръ", "ru-1918").unwrap(), "мир и мир");
        assert_eq!(
            modernize("Ѳедоръ объѣхалъ", "дореформенная").unwrap(),
            "Федор объехал"
        );
        assert_eq!(modernize("ъ", "ru-1918").unwrap(), "ъ");
        assert!(modernize("x", "klingon").is_err());
    }

    #[test]
    fn ala_lc_ties_and_case() {
        assert_eq!(
            translit("Царь Юрий", "ala").unwrap(),
            "T\u{361}sarʹ I\u{361}uriĭ"
        );
        assert_eq!(translit("ЩУКА", "ala").unwrap(), "SHCHUKA");
    }

    #[test]
    fn slug_is_ascii_bgn() {
        assert_eq!(
            translit("Ёлка объём Щука", "slug").unwrap(),
            "Yelka obyem Shchuka"
        );
        assert!(translit("Пётр Ильич", "slug").unwrap().is_ascii());
    }

    #[test]
    fn ukrainian_national_system() {
        assert_eq!(
            translit("Київ Запоріжжя Згорани", "uk").unwrap(),
            "Kyiv Zaporizhzhia Zghorany"
        );
        assert_eq!(
            translit("Єнакієве Ялта Юрій Їжакевич", "uk").unwrap(),
            "Yenakiieve Yalta Yurii Yizhakevych"
        );
        assert_eq!(translit("Знам'янка", "uk").unwrap(), "Znamianka");
        assert_eq!(translit("ЙОСИП", "uk").unwrap(), "YOSYP");
    }

    #[test]
    fn serbian_both_ways() {
        assert_eq!(
            translit("Љубљана Ђорђе Џек", "sr-latn").unwrap(),
            "Ljubljana Đorđe Džek"
        );
        assert_eq!(translit("ЉУБЉАНА", "sr-latn").unwrap(), "LJUBLJANA");
        assert_eq!(
            translit("Ljubljana Đorđe Džek", "sr-cyrl").unwrap(),
            "Љубљана Ђорђе Џек"
        );
        assert_eq!(translit("NJEGOŠ čvrst", "sr-cyrl").unwrap(), "ЊЕГОШ чврст");
    }

    #[test]
    fn elot_transcription() {
        assert_eq!(translit("Ευαγγελία", "elot").unwrap(), "Evangelia");
        assert_eq!(translit("Αυτοκίνητο", "elot").unwrap(), "Aftokinito");
        assert_eq!(
            translit("Μπάμπης μπουζούκι Λάμπρος", "elot").unwrap(),
            "Bampis bouzouki Lampros"
        );
        assert_eq!(
            translit("Άγκυρα Χανιά Ωμέγα βήτα", "elot").unwrap(),
            "Agkyra Chania Omega vita"
        );
        // a dialytika or an accent on the first vowel breaks the diphthong
        assert_eq!(translit("άυλος αϋπνία", "elot").unwrap(), "aylos aypnia");
        assert_eq!(translit("ΕΥΡΩΠΗ", "elot").unwrap(), "EVROPI");
        // ISO 843 type 1 is unchanged
        assert_eq!(translit("βήτα", "iso843").unwrap(), "vī́ta");
    }

    #[test]
    fn latin_to_ascii_tables() {
        assert_eq!(
            translit("Müller Straße Ödön Łódź", "de").unwrap(),
            "Mueller Strasse Oedoen Łódź"
        );
        assert_eq!(
            translit("Müller Straße Ødegård Łódź Þór Ñandú", "mrz").unwrap(),
            "MUELLER STRASSE OEDEGAARD LODZ THOR NANDU"
        );
        assert_eq!(
            translit("Müller Straße Ødegård Łódź Þór œuvre Ærø", "ascii").unwrap(),
            "Muller Strasse Odegard Lodz Thor oeuvre Aero"
        );
        assert_eq!(translit("ÆRØ", "ascii").unwrap(), "AERO");
        assert!(translit("x", "sci").is_err());
    }

    #[test]
    fn long_s_and_ligatures_open() {
        assert_eq!(
            modernize("ſhall ﬁnd ﬆill", "long-s").unwrap(),
            "shall find still"
        );
    }
}
