//! Pure reference parsing, independent of audio, databases, and UI state.
use regex::Regex;
use std::{collections::HashSet, sync::LazyLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reference {
    pub book: i32,
    pub chapter: i32,
    pub first: i32,
    pub last: i32,
}

pub trait ReferenceDetector: Send + Sync {
    fn detect(&self, transcript: &str) -> Vec<Reference>;
}

pub struct BibleReferenceDetector;
const BOOKS: &[&str] = &[
    "Genesis|Gen",
    "Exodus|Exod|Ex",
    "Leviticus|Lev",
    "Numbers|Num",
    "Deuteronomy|Deut",
    "Joshua|Josh",
    "Judges|Judg",
    "Ruth",
    "1 Samuel|1Sam",
    "2 Samuel|2Sam",
    "1 Kings|1Kgs",
    "2 Kings|2Kgs",
    "1 Chronicles|1Chr",
    "2 Chronicles|2Chr",
    "Ezra",
    "Nehemiah|Neh",
    "Esther|Esth",
    "Job",
    "Psalms|Psalm|Ps",
    "Proverbs|Prov",
    "Ecclesiastes|Eccl",
    "Song of Solomon|Song of Songs|Song",
    "Isaiah|Isa",
    "Jeremiah|Jer",
    "Lamentations|Lam",
    "Ezekiel|Ezek",
    "Daniel|Dan",
    "Hosea|Hos",
    "Joel",
    "Amos",
    "Obadiah|Obad",
    "Jonah",
    "Micah|Mic",
    "Nahum|Nah",
    "Habakkuk|Hab",
    "Zephaniah|Zeph",
    "Haggai|Hag",
    "Zechariah|Zech",
    "Malachi|Mal",
    "Matthew|Matt",
    "Mark|Mk",
    "Luke|Lk",
    "John|Jn",
    "Acts",
    "Romans|Rom",
    "1 Corinthians|1Cor",
    "2 Corinthians|2Cor",
    "Galatians|Gal",
    "Ephesians|Eph",
    "Philippians|Phil",
    "Colossians|Col",
    "1 Thessalonians|1Thess",
    "2 Thessalonians|2Thess",
    "1 Timothy|1Tim",
    "2 Timothy|2Tim",
    "Titus",
    "Philemon|Phlm",
    "Hebrews|Heb",
    "James|Jas",
    "1 Peter|1Pet",
    "2 Peter|2Pet",
    "1 John|1John",
    "2 John|2John",
    "3 John|3John",
    "Jude",
    "Revelation|Rev",
];

static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let mut aliases: Vec<_> = BOOKS.iter().flat_map(|b| b.split('|')).collect();
    aliases.sort_by_key(|a| std::cmp::Reverse(a.len()));
    let books = aliases
        .into_iter()
        .map(regex::escape)
        .collect::<Vec<_>>()
        .join("|");
    Regex::new(&format!(r"(?i)\b(?P<book>{books})\.?\s+(?:chapter\s+)?(?P<chapter>\d{{1,3}})(?:\s*:\s*|\s+(?:(?:and\s+)?verses?\s+)?)(?P<first>\d{{1,3}})(?:\s*(?:-|to|through)\s*(?P<last>\d{{1,3}}))?\b")).expect("static reference expression")
});

fn number(word: &str) -> Option<i32> {
    match word {
        "zero" => Some(0),
        "one" | "first" => Some(1),
        "two" | "second" => Some(2),
        "three" | "third" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        "thirty" => Some(30),
        "forty" => Some(40),
        "fifty" => Some(50),
        "sixty" => Some(60),
        "seventy" => Some(70),
        "eighty" => Some(80),
        "ninety" => Some(90),
        _ => None,
    }
}

pub fn normalize(input: &str) -> String {
    // Hyphenated spoken numbers are joined; numeric verse ranges keep their separator.
    let input = input.to_lowercase().replace(['–', '—'], "-");
    let input = Regex::new(r"([a-z])-([a-z])")
        .unwrap()
        .replace_all(&input, "$1 $2");
    let input = input.replace([',', ';', '!', '?'], " ");
    let words: Vec<_> = input.split_whitespace().collect();
    let mut output = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let word = words[i].trim_end_matches('.');
        if let Some(mut value) = number(word) {
            if words.get(i + 1) == Some(&"hundred") {
                value *= 100;
                i += 1;
                if words.get(i + 1) == Some(&"and") {
                    i += 1;
                }
                if let Some(next) = words
                    .get(i + 1)
                    .and_then(|w| number(w.trim_end_matches('.')))
                {
                    value += next;
                    i += 1;
                    if next >= 20
                        && let Some(unit @ 1..=9) = words
                            .get(i + 1)
                            .and_then(|w| number(w.trim_end_matches('.')))
                    {
                        value += unit;
                        i += 1;
                    }
                }
            } else if value >= 20
                && let Some(unit @ 1..=9) = words
                    .get(i + 1)
                    .and_then(|w| number(w.trim_end_matches('.')))
            {
                value += unit;
                i += 1;
            }
            output.push(value.to_string());
        } else {
            output.push(words[i].to_string());
        }
        i += 1;
    }
    output.join(" ")
}

impl ReferenceDetector for BibleReferenceDetector {
    fn detect(&self, transcript: &str) -> Vec<Reference> {
        let text = normalize(transcript);
        let mut seen = HashSet::new();
        PATTERN
            .captures_iter(&text)
            .filter_map(|capture| {
                // Do not misread a cross-chapter endpoint as a verse range.
                if text[capture.get(0)?.end()..].trim_start().starts_with(':') {
                    return None;
                }
                let book = BOOKS.iter().position(|aliases| {
                    aliases
                        .split('|')
                        .any(|a| a.eq_ignore_ascii_case(&capture["book"]))
                })? as i32
                    + 1;
                let chapter = capture["chapter"].parse().ok()?;
                let first = capture["first"].parse().ok()?;
                let last = capture
                    .name("last")
                    .map(|v| v.as_str().parse())
                    .transpose()
                    .ok()?
                    .unwrap_or(first);
                let reference = Reference {
                    book,
                    chapter,
                    first,
                    last,
                };
                (chapter > 0
                    && chapter <= 150
                    && first > 0
                    && last >= first
                    && last <= 176
                    && seen.insert(reference.clone()))
                .then_some(reference)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spoken_numbered_books_and_ranges() {
        let refs = BibleReferenceDetector.detect("First John chapter three verse sixteen through eighteen. Psalm twenty-three verse one. John 3:16");
        assert_eq!(
            refs,
            vec![
                Reference {
                    book: 62,
                    chapter: 3,
                    first: 16,
                    last: 18
                },
                Reference {
                    book: 19,
                    chapter: 23,
                    first: 1,
                    last: 1
                },
                Reference {
                    book: 43,
                    chapter: 3,
                    first: 16,
                    last: 16
                }
            ]
        );
    }
    #[test]
    fn multiple_references_boundaries_and_deduplication() {
        let refs = BibleReferenceDetector
            .detect("John 3:16, Romans 8:28; John 3:16. notjohn 3:16 John 0:1 John 3:20-2");
        assert_eq!(refs.len(), 2);
    }
    #[test]
    fn cross_chapter_ranges_are_not_partially_detected() {
        assert!(BibleReferenceDetector.detect("John 3:1-4:2").is_empty());
    }
    #[test]
    fn hundred_and_verse_numbers() {
        assert_eq!(
            BibleReferenceDetector
                .detect("Psalm one hundred and nineteen verse one hundred and five")[0],
            Reference {
                book: 19,
                chapter: 119,
                first: 105,
                last: 105
            }
        );
    }
}
