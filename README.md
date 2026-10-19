# transliteratio

Transliteration tables, string to string, with no dependency
beyond Unicode normalization: the ISO letter tables for the major
non-Latin scripts, the national and library systems in use beside
them, the inverses of the reversible tables, the Latin-to-ASCII
folds, and beside them the accent tables by language and the
historical spelling tables.

```rust
use transliteratio::{transliterate, deaccent, strip};

assert_eq!(transliterate("Щука", "iso9").unwrap(), "Ŝuka");
assert_eq!(transliterate("Щука", "gost").unwrap(), "Shhuka");
assert_eq!(transliterate("Київ", "uk").unwrap(), "Kyiv");
assert_eq!(transliterate("Ευαγγελία", "elot").unwrap(), "Evangelia");
assert_eq!(transliterate("한글", "rr").unwrap(), "hangeul");
assert_eq!(transliterate("ㄅㄟˇㄐㄧㄥ", "pinyin").unwrap(), "běijīng");
assert_eq!(deaccent("élève garçon", "fr").unwrap(), "eleve garcon");
assert_eq!(strip("Ἀχιλλεύς"), "Αχιλλευς");
```

## Tables

Every table is letter-wise, with the few context rules its
standard has (the yotated letters at the start of a word, the
Greek diphthongs, the jamo positions in a Korean syllable). A
character the table does not name passes through unchanged, so a
text that mixes scripts transliterates its own scripts and keeps
the rest. Case is kept: an uppercase letter titlecases its
output, and uppercases it whole when the next letter is uppercase
too (Щука → Shchuka, ЩУКА → SHCHUKA).

| Code | System | Script |
|------|--------|--------|
| `iso` | each script's ISO table, chosen per run of script | any |
| `iso9` | ISO 9:1995 (GOST 7.79 system A), reversible | Cyrillic |
| `gost` | GOST 7.79-2000 system B, ASCII | Cyrillic |
| `scientific` | the scholarly system (DIN 1460, ISO/R 9:1968) | Cyrillic |
| `bgn` | BGN/PCGN, by language (ru uk be bg sr mk kk, el, ar fa, he, hy, ka) | several |
| `passport` | the Russian passport system (ICAO Doc 9303) | Cyrillic |
| `ala` | ALA-LC, by script and language | several |
| `slug` | BGN/PCGN folded to ASCII | Cyrillic |
| `uk` | the Ukrainian national system (2010) | Cyrillic |
| `bg` | the Bulgarian Streamlined System (2009) | Cyrillic |
| `be` | the Belarusian national system (2007) | Cyrillic |
| `kk` | the Kazakh Latin alphabet (2021) | Cyrillic |
| `mn` | Mongolian, MNS 5217:2012 | Cyrillic |
| `sr-latn`, `sr-cyrl` | the two Serbian alphabets, each way | Cyrillic, Latin |
| `iso9-cyrl`, `gost-cyrl` | ISO 9 and GOST 7.79 B read back | Latin |
| `iso843` | ISO 843:1997 type 1 | Greek |
| `elot` | ELOT 743 (ISO 843 type 2), the transcription | Greek |
| `betacode` | TLG Beta Code | Greek |
| `iso233` | ISO 233:1984 | Arabic |
| `din31635` | DIN 31635:2011, Arabic and Persian | Arabic |
| `buckwalter` | Buckwalter's ASCII, reversible | Arabic |
| `iso259` | ISO 259:1984 | Hebrew |
| `heb` | the Academy of the Hebrew Language, 2006 | Hebrew |
| `iso9985` | ISO 9985:1996 | Armenian |
| `iso9984` | ISO 9984:1996 | Georgian |
| `ka` | the Georgian national system (2002) | Georgian |
| `iso15919` | ISO 15919:2001 | Devanagari |
| `iast` | IAST | Devanagari |
| `hunterian` | the Hunterian system, no diacritics | Devanagari |
| `iso11940` | ISO 11940:1998 | Thai |
| `iso3602` | ISO 3602:1989 (Kunrei-shiki) | kana |
| `hepburn` | modified Hepburn | kana |
| `pinyin` | Hanyu Pinyin (ISO 7098), tone marks | bopomofo, pinyin readings with tone numbers |
| `wade` | Wade–Giles, superscript tones | bopomofo, pinyin readings |
| `rr` | the Revised Romanization of Korean (2000) | hangul |
| `mr` | McCune–Reischauer (1939) | hangul |
| `de` | DIN 5007-2 (ä ö ü ß as ae oe ue ss) | Latin |
| `mrz` | ICAO Doc 9303 part 3, the machine-readable zone | Latin |
| `ascii` | the neutral fold to ASCII | Latin |

`TABLES` lists them with their metadata. A table that needs a
lexicon rather than letters (Chinese characters under `pinyin`
and `wade`, kanji under `hepburn` and `iso3602`, Thai under
`rtgs`) is not here: asking one over such text is
`Error::NeedsLexicon`, so a reader never takes a passed-through
character for a table gap.

## Accents and spelling

`deaccent(text, lang)` removes the combining marks a language
counts as accents and keeps the ones that make letters: the stress
mark alone for Russian, Ukrainian, Belarusian and Bulgarian (ё й ї ў
stay), the accents, breathings and quantities for Ancient Greek
(the iota subscript and the dialytika stay), the tonos for Modern
Greek, the five marks and the cedilla for the Romance languages
with each one's exceptions (ñ, ç and l·l in Catalan, ș ț in
Romanian), the tone marks for Vietnamese (â ă ê ô ơ ư đ stay), and
nothing for the languages whose marks are all letters. `strip`
removes every nonspacing mark. `modernize(text, table)` folds a
historical spelling to the modern one (`ru-1918`, `long-s`).

## Features

Every table is a Cargo feature, so a build carries only what it
needs: one feature per table (`gost`, `elot`, `hepburn`, …), one per
family member (`bgn-uk`, `ala-greek`, …), umbrellas per script
(`cyrillic`, `greek`, `arabic`, `hebrew`, `armenian`, `georgian`,
`indic`, `thai`, `japanese`, `chinese`, `korean`, `latin`) and per
family (`iso`, `ala`, `bgn`), and `spelling`, `deaccent` (with
`deaccent-cyrillic`, `deaccent-greek`, `deaccent-romance`,
`deaccent-vietnamese`) and `strip`. The default is `all`. A table
whose feature is off is refused at run time by name,
`Error::FeatureOff`, naming the feature.

```toml
[dependencies]
transliteratio = { version = "0.1", default-features = false, features = ["gost", "elot"] }
```

## Licence

MIT or Apache-2.0, at your option.
