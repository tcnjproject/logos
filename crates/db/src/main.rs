//! A port of https://github.com/openbezal/rhema/blob/main/data/build-bible-db.ts to Rust
//! Builds rhema.db from Bible JSON sources + cross-references.
//! Run: cargo run --release
//! Prereq: bun run data/download-sources.ts (TODO: eventually integrate source download into this build script)


use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs;
use std::path::Path;

const DATA_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// OSIS book abbreviation -> book number mapping.
static OSIS_TO_NUM: phf::Map<&'static str, i64> = phf::phf_map! {
    "Gen" => 1, "Exod" => 2, "Lev" => 3, "Num" => 4, "Deut" => 5, "Josh" => 6, "Judg" => 7, "Ruth" => 8,
    "1Sam" => 9, "2Sam" => 10, "1Kgs" => 11, "2Kgs" => 12, "1Chr" => 13, "2Chr" => 14,
    "Ezra" => 15, "Neh" => 16, "Esth" => 17, "Job" => 18, "Ps" => 19, "Prov" => 20, "Eccl" => 21,
    "Song" => 22, "Isa" => 23, "Jer" => 24, "Lam" => 25, "Ezek" => 26, "Dan" => 27, "Hos" => 28,
    "Joel" => 29, "Amos" => 30, "Obad" => 31, "Jonah" => 32, "Mic" => 33, "Nah" => 34, "Hab" => 35,
    "Zeph" => 36, "Hag" => 37, "Zech" => 38, "Mal" => 39, "Matt" => 40, "Mark" => 41, "Luke" => 42,
    "John" => 43, "Acts" => 44, "Rom" => 45, "1Cor" => 46, "2Cor" => 47, "Gal" => 48, "Eph" => 49,
    "Phil" => 50, "Col" => 51, "1Thess" => 52, "2Thess" => 53, "1Tim" => 54, "2Tim" => 55,
    "Titus" => 56, "Phlm" => 57, "Heb" => 58, "Jas" => 59, "1Pet" => 60, "2Pet" => 61,
    "1John" => 62, "2John" => 63, "3John" => 64, "Jude" => 65, "Rev" => 66,
};

/// Standard book abbreviations for our DB, keyed by the full book name found in sources.
static BOOK_ABBREVS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "Genesis" => "Gen", "Exodus" => "Exod", "Leviticus" => "Lev", "Numbers" => "Num",
    "Deuteronomy" => "Deut", "Joshua" => "Josh", "Judges" => "Judg", "Ruth" => "Ruth",
    "1 Samuel" => "1Sam", "2 Samuel" => "2Sam", "1 Kings" => "1Kgs", "2 Kings" => "2Kgs",
    "1 Chronicles" => "1Chr", "2 Chronicles" => "2Chr", "Ezra" => "Ezra", "Nehemiah" => "Neh",
    "Esther" => "Esth", "Job" => "Job", "Psalms" => "Ps", "Proverbs" => "Prov",
    "Ecclesiastes" => "Eccl", "Song of Solomon" => "Song", "Isaiah" => "Isa", "Jeremiah" => "Jer",
    "Lamentations" => "Lam", "Ezekiel" => "Ezek", "Daniel" => "Dan", "Hosea" => "Hos",
    "Joel" => "Joel", "Amos" => "Amos", "Obadiah" => "Obad", "Jonah" => "Jonah",
    "Micah" => "Mic", "Nahum" => "Nah", "Habakkuk" => "Hab", "Zephaniah" => "Zeph",
    "Haggai" => "Hag", "Zechariah" => "Zech", "Malachi" => "Mal", "Matthew" => "Matt",
    "Mark" => "Mark", "Luke" => "Luke", "John" => "John", "Acts" => "Acts", "Romans" => "Rom",
    "1 Corinthians" => "1Cor", "2 Corinthians" => "2Cor", "Galatians" => "Gal",
    "Ephesians" => "Eph", "Philippians" => "Phil", "Colossians" => "Col",
    "1 Thessalonians" => "1Thess", "2 Thessalonians" => "2Thess",
    "1 Timothy" => "1Tim", "2 Timothy" => "2Tim", "Titus" => "Titus", "Philemon" => "Phlm",
    "Hebrews" => "Heb", "James" => "Jas", "1 Peter" => "1Pet", "2 Peter" => "2Pet",
    "1 John" => "1John", "2 John" => "2John", "3 John" => "3John", "Jude" => "Jude",
    "Revelation" => "Rev",
};

#[derive(Deserialize)]
struct ScrollmapperJson {
    books: Vec<BookJson>,
}

#[derive(Deserialize)]
struct BookJson {
    name: String,
    chapters: Vec<ChapterJson>,
}

#[derive(Deserialize)]
struct ChapterJson {
    chapter: i64,
    verses: Vec<VerseJson>,
}

#[derive(Deserialize)]
struct VerseJson {
    verse: i64,
    text: String,
}

struct TranslationMeta {
    file: &'static str,
    abbreviation: &'static str,
    title: &'static str,
    language: &'static str,
    license: &'static str,
}

const TRANSLATIONS_META: &[TranslationMeta] = &[
    // English
    TranslationMeta { file: "KJV.json", abbreviation: "KJV", title: "King James Version", language: "en", license: "Public Domain" },
    TranslationMeta { file: "NIV.json", abbreviation: "NIV", title: "New International Version", language: "en", license: "Biblica" },
    TranslationMeta { file: "ESV.json", abbreviation: "ESV", title: "English Standard Version", language: "en", license: "Crossway" },
    TranslationMeta { file: "NASB.json", abbreviation: "NASB", title: "New American Standard Bible", language: "en", license: "Lockman Foundation" },
    TranslationMeta { file: "NKJV.json", abbreviation: "NKJV", title: "New King James Version", language: "en", license: "Thomas Nelson" },
    TranslationMeta { file: "NLT.json", abbreviation: "NLT", title: "New Living Translation", language: "en", license: "Tyndale House" },
    TranslationMeta { file: "AMP.json", abbreviation: "AMP", title: "Amplified Bible", language: "en", license: "Lockman Foundation" },
    // Non-English
    TranslationMeta { file: "SpaRV.json", abbreviation: "SpaRV", title: "Reina-Valera 1909", language: "es", license: "Public Domain" },
    TranslationMeta { file: "FreJND.json", abbreviation: "FreJND", title: "J.N. Darby French 1885", language: "fr", license: "Public Domain" },
    TranslationMeta { file: "PorBLivre.json", abbreviation: "PorBLivre", title: "Biblia Livre", language: "pt", license: "Public Domain" },
];

struct OsisRef {
    book: i64,
    chapter: i64,
    verse: i64,
}

/// Parses an OSIS reference like "Gen.1.1" or "1John.3.16".
fn parse_osis(reference: &str) -> Option<OsisRef> {
    let mut parts = reference.split('.');
    let book_abbrev = parts.next()?;
    let chapter: i64 = parts.next()?.trim().parse().ok()?;
    let verse: i64 = parts.next()?.trim().parse().ok()?;
    let book = *OSIS_TO_NUM.get(book_abbrev)?;
    Some(OsisRef { book, chapter, verse })
}

/// Formats an integer with thousands separators, e.g. 1234567 -> "1,234,567".
fn thousands(n: i64) -> String {
    let digits = n.abs().to_string();
    let grouped: String = digits
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(",");
    if n < 0 {
        format!("-{grouped}")
    } else {
        grouped
    }
}

fn main() -> Result<()> {
    println!("\n🔨 Building rhema.db ...\n");

    let data_dir = Path::new(DATA_DIR);
    let db_path = data_dir.join("rhema.db");
    let schema_path = data_dir.join("schema.sql");
    let sources_dir = data_dir.join("sources");
    let cross_refs_path = data_dir.join("cross-refs").join("cross_references.txt");

    // Remove existing DB
    let _ = fs::remove_file(&db_path);

    let conn = Connection::open(&db_path).context("failed to open rhema.db")?;

    // Create schema
    let schema = fs::read_to_string(&schema_path)
        .with_context(|| format!("failed to read {}", schema_path.display()))?;
    conn.execute_batch(&schema).context("failed to apply schema.sql")?;

    // Prepare insert statements, reused across all translations
    let mut insert_translation = conn.prepare(
        "INSERT INTO translations (abbreviation, title, language, license) VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut insert_book = conn.prepare(
        "INSERT INTO books (translation_id, book_number, name, abbreviation, testament) VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    let mut insert_verse = conn.prepare(
        "INSERT INTO verses (translation_id, book_id, book_number, book_name, book_abbreviation, chapter, verse, text) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;

    // Process each translation
    for meta in TRANSLATIONS_META {
        let file_path = sources_dir.join(meta.file);
        println!("  📖 Processing {}...", meta.abbreviation);

        let raw = match fs::read_to_string(&file_path) {
            Ok(raw) => raw,
            Err(_) => {
                println!("  ⏭ {} not found, skipping", meta.file);
                continue;
            }
        };

        let data: ScrollmapperJson = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", meta.file))?;

        conn.execute_batch("BEGIN TRANSACTION")?;

        insert_translation.execute(params![
            meta.abbreviation,
            meta.title,
            meta.language,
            meta.license
        ])?;
        let translation_id = conn.last_insert_rowid();

        let mut verse_count: i64 = 0;

        for (book_idx, book) in data.books.iter().enumerate() {
            let book_number = (book_idx + 1) as i64;
            let abbrev: String = BOOK_ABBREVS
                .get(book.name.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| book.name.chars().take(4).collect());
            let testament = if book_number <= 39 { "OT" } else { "NT" };

            insert_book.execute(params![
                translation_id,
                book_number,
                book.name,
                abbrev,
                testament
            ])?;
            let book_id = conn.last_insert_rowid();

            for chapter in &book.chapters {
                for verse in &chapter.verses {
                    insert_verse.execute(params![
                        translation_id,
                        book_id,
                        book_number,
                        book.name,
                        abbrev,
                        chapter.chapter,
                        verse.verse,
                        verse.text
                    ])?;
                    verse_count += 1;
                }
            }
        }

        conn.execute_batch("COMMIT")?;
        println!(
            "  ✓ {}: {} books, {} verses",
            meta.abbreviation,
            data.books.len(),
            verse_count
        );
    }

    // Build FTS5 index
    println!("\n  🔍 Building FTS5 search index...");
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS verses_fts USING fts5(text, content='verses', content_rowid='id', tokenize='unicode61');
         INSERT INTO verses_fts(rowid, text) SELECT id, text FROM verses;",
    )
    .context("failed to build FTS5 index")?;
    println!("  ✓ FTS5 index built");

    // Import cross-references
    println!("\n  🔗 Importing cross-references...");
    let cross_ref_raw = match fs::read_to_string(&cross_refs_path) {
        Ok(raw) => raw,
        Err(_) => {
            println!("  ⏭ cross_references.txt not found, skipping");
            String::new()
        }
    };

    if !cross_ref_raw.is_empty() {
        let mut insert_cross_ref = conn.prepare(
            "INSERT INTO cross_references (from_book, from_chapter, from_verse, to_book, to_chapter, to_verse_start, to_verse_end, votes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        conn.execute_batch("BEGIN TRANSACTION")?;

        let mut cross_ref_count: i64 = 0;
        for line in cross_ref_raw.lines() {
            if line.starts_with("From") || line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let mut fields = line.split('\t');
            let (Some(from_str), Some(to_str)) = (fields.next(), fields.next()) else {
                continue;
            };
            let votes_str = fields.next();

            let Some(from) = parse_osis(from_str) else {
                continue;
            };
            let Some(to) = parse_osis(to_str) else {
                continue;
            };
            let votes: i64 = votes_str
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            insert_cross_ref.execute(params![
                from.book,
                from.chapter,
                from.verse,
                to.book,
                to.chapter,
                to.verse,
                to.verse,
                votes
            ])?;
            cross_ref_count += 1;
        }

        conn.execute_batch("COMMIT")?;
        println!("  ✓ {} cross-references imported", thousands(cross_ref_count));
    }

    // Optimize
    println!("\n  ⚡ Optimizing database...");
    conn.execute_batch("PRAGMA optimize; ANALYZE;")?;

    // Stats
    let verse_total: i64 = conn.query_row("SELECT COUNT(*) FROM verses", [], |r| r.get(0))?;
    let trans_total: i64 = conn.query_row("SELECT COUNT(*) FROM translations", [], |r| r.get(0))?;
    let cross_total: i64 =
        conn.query_row("SELECT COUNT(*) FROM cross_references", [], |r| r.get(0))?;

    println!("\n✅ rhema.db built successfully!");
    println!("   {trans_total} translations");
    println!("   {} verses", thousands(verse_total));
    println!("   {} cross-references", thousands(cross_total));
    println!("   📁 {}\n", db_path.display());

    Ok(())
}
