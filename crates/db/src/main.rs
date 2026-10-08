//! CLI tool to download Bible data and Whisper models, and build `data/rhema.db`.
//!
//! Subcommands:
//! - `setup [--model <name>] [--all]` (default if no args: downloads sources, builds DB, downloads Whisper model)
//! - `download-sources [--all]`
//! - `download-model [--model <name>]`
//! - `build [--out <path>]`

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

const SCHEMA_SQL: &str = include_str!("../schema.sql");

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
    is_copyrighted: bool,
}

const TRANSLATIONS_META: &[TranslationMeta] = &[
    // Public Domain
    TranslationMeta { file: "KJV.json", abbreviation: "KJV", title: "King James Version", language: "en", license: "Public Domain", is_copyrighted: false },
    TranslationMeta { file: "SpaRV.json", abbreviation: "SpaRV", title: "Reina-Valera 1909", language: "es", license: "Public Domain", is_copyrighted: false },
    TranslationMeta { file: "FreJND.json", abbreviation: "FreJND", title: "J.N. Darby French 1885", language: "fr", license: "Public Domain", is_copyrighted: false },
    TranslationMeta { file: "PorBLivre.json", abbreviation: "PorBLivre", title: "Biblia Livre", language: "pt", license: "Public Domain", is_copyrighted: false },
    // Copyrighted (included if present in sources)
    TranslationMeta { file: "NIV.json", abbreviation: "NIV", title: "New International Version", language: "en", license: "Biblica", is_copyrighted: true },
    TranslationMeta { file: "ESV.json", abbreviation: "ESV", title: "English Standard Version", language: "en", license: "Crossway", is_copyrighted: true },
    TranslationMeta { file: "NASB.json", abbreviation: "NASB", title: "New American Standard Bible", language: "en", license: "Lockman Foundation", is_copyrighted: true },
    TranslationMeta { file: "NKJV.json", abbreviation: "NKJV", title: "New King James Version", language: "en", license: "Thomas Nelson", is_copyrighted: true },
    TranslationMeta { file: "NLT.json", abbreviation: "NLT", title: "New Living Translation", language: "en", license: "Tyndale House", is_copyrighted: true },
    TranslationMeta { file: "AMP.json", abbreviation: "AMP", title: "Amplified Bible", language: "en", license: "Lockman Foundation", is_copyrighted: true },
];

const SCROLLMAPPER_BASE_URL: &str =
    "https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json";
const CROSS_REFS_URL: &str = "https://a.openbible.info/data/cross-references.zip";
const ALL_SOURCES_ZIP_URL: &str =
    "https://drive.google.com/uc?export=download&id=1HQiNf_nCVRQrMbdmzVG7vq-Fvfqh1nzW&confirm=t";

struct OsisRef {
    book: i64,
    chapter: i64,
    verse: i64,
}

fn parse_osis(reference: &str) -> Option<OsisRef> {
    let mut parts = reference.split('.');
    let book_abbrev = parts.next()?;
    let chapter: i64 = parts.next()?.trim().parse().ok()?;
    let verse: i64 = parts.next()?.trim().parse().ok()?;
    let book = *OSIS_TO_NUM.get(book_abbrev)?;
    Some(OsisRef { book, chapter, verse })
}

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

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    println!("  ⬇ Downloading {} -> {}", url, dest.display());

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    let mut response = client.get(url).send()?.error_for_status()?;

    let tmp_path = dest.with_extension("tmp");
    let mut file = File::create(&tmp_path)?;
    let mut buffer = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;

    loop {
        let n = response.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        downloaded += n as u64;
    }
    file.flush()?;
    drop(file);

    fs::rename(&tmp_path, dest)?;
    let mb = (downloaded as f64) / (1024.0 * 1024.0);
    println!("  ✓ Saved {} ({:.1} MB)", dest.display(), mb);
    Ok(())
}

fn unzip_file(zip_path: &Path, extract_to: &Path) -> Result<()> {
    fs::create_dir_all(extract_to)?;
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => extract_to.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn download_sources(root: &Path, download_all: bool) -> Result<()> {
    let sources_dir = root.join("data").join("sources");
    let cross_refs_dir = root.join("data").join("cross-refs");
    fs::create_dir_all(&sources_dir)?;
    fs::create_dir_all(&cross_refs_dir)?;

    if download_all {
        println!("\n📦 Downloading full translations bundle (10 translations)...");
        let zip_dest = root.join("data").join("sources.zip");
        download_file(ALL_SOURCES_ZIP_URL, &zip_dest)?;
        println!("  📦 Extracting sources.zip...");
        unzip_file(&zip_dest, &root.join("data"))?;
        let _ = fs::remove_file(&zip_dest);
    } else {
        println!("\n📖 Downloading public-domain Bible translations (KJV, SpaRV, FreJND, PorBLivre)...");
        let public_domain_files = ["KJV.json", "SpaRV.json", "FreJND.json", "PorBLivre.json"];
        for f in public_domain_files {
            let dest = sources_dir.join(f);
            if dest.exists() && fs::metadata(&dest)?.len() > 1000 {
                println!("  ⏭ {} already exists, skipping", f);
                continue;
            }
            let url = format!("{SCROLLMAPPER_BASE_URL}/{f}");
            download_file(&url, &dest)?;
        }
    }

    // Cross-references
    let cross_ref_txt = cross_refs_dir.join("cross_references.txt");
    if !cross_ref_txt.exists() || fs::metadata(&cross_ref_txt)?.len() < 1000 {
        println!("\n🔗 Downloading cross-references...");
        let zip_dest = cross_refs_dir.join("cross-references.zip");
        download_file(CROSS_REFS_URL, &zip_dest)?;
        println!("  📦 Extracting cross-references...");
        unzip_file(&zip_dest, &cross_refs_dir)?;
        let _ = fs::remove_file(&zip_dest);
    } else {
        println!("  ⏭ cross_references.txt already exists, skipping");
    }

    println!("✅ Source download complete!\n");
    Ok(())
}

fn download_whisper_model(root: &Path, model_name: &str) -> Result<()> {
    let models_dir = root.join("models").join("whisper");
    fs::create_dir_all(&models_dir)?;

    let (filename, url) = if model_name == "large-v3-turbo" {
        (
            "ggml-large-v3-turbo-q8_0.bin".to_string(),
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q8_0.bin".to_string(),
        )
    } else {
        let fname = format!("ggml-{model_name}.bin");
        let u = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{fname}");
        (fname, u)
    };

    let dest = models_dir.join(&filename);
    if dest.exists() && fs::metadata(&dest)?.len() > 1_000_000 {
        println!("⏭ Whisper model already exists: {}", dest.display());
        return Ok(());
    }

    println!("\n🎙 Downloading Whisper model '{model_name}'...");
    download_file(&url, &dest)?;
    println!("✅ Whisper model ready at {}\n", dest.display());
    Ok(())
}

fn build_database(root: &Path, db_out: Option<&Path>) -> Result<()> {
    let default_db_path = root.join("data").join("rhema.db");
    let db_path = db_out.unwrap_or(&default_db_path);
    let sources_dir = root.join("data").join("sources");
    let cross_refs_path = root.join("data").join("cross-refs").join("cross_references.txt");

    if let Some(p) = db_path.parent() {
        fs::create_dir_all(p)?;
    }

    println!("\n🔨 Building {} ...", db_path.display());

    // Clean existing database
    let _ = fs::remove_file(db_path);
    for ext in ["-wal", "-shm"] {
        let _ = fs::remove_file(format!("{}{ext}", db_path.display()));
    }

    let conn = Connection::open(db_path).context("failed to open database")?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(SCHEMA_SQL).context("failed to apply schema.sql")?;

    {
        let mut insert_translation = conn.prepare(
            "INSERT INTO translations (abbreviation, title, language, license, is_copyrighted) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        let mut insert_book = conn.prepare(
            "INSERT INTO books (translation_id, book_number, name, abbreviation, testament) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        let mut insert_verse = conn.prepare(
            "INSERT INTO verses (translation_id, book_id, book_number, book_name, book_abbreviation, chapter, verse, text) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        let mut imported_translations = 0;

        for meta in TRANSLATIONS_META {
            let file_path = sources_dir.join(meta.file);
            let raw = match fs::read_to_string(&file_path) {
                Ok(raw) => raw,
                Err(_) => {
                    continue;
                }
            };

            println!("  📖 Importing {} ({})...", meta.abbreviation, meta.title);

            let data: ScrollmapperJson = serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse {}", meta.file))?;

            conn.execute_batch("BEGIN TRANSACTION")?;

            insert_translation.execute(params![
                meta.abbreviation,
                meta.title,
                meta.language,
                meta.license,
                if meta.is_copyrighted { 1 } else { 0 },
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
                thousands(verse_count)
            );
            imported_translations += 1;
        }

        if imported_translations == 0 {
            anyhow::bail!(
                "No translation JSON files found in {}. Run 'cargo run -p build_bible_db -- download-sources' first.",
                sources_dir.display()
            );
        }
    }

    // Build FTS5 index
    println!("\n  🔍 Building FTS5 search index...");
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS verses_fts USING fts5(text, content='verses', content_rowid='id', tokenize='unicode61');
         INSERT INTO verses_fts(rowid, text) SELECT id, text FROM verses;",
    )
    .context("failed to build FTS5 index")?;
    println!("  ✓ FTS5 index built");

    // Cross-references
    if cross_refs_path.exists() {
        println!("\n  🔗 Importing cross-references...");
        let cross_ref_raw = fs::read_to_string(&cross_refs_path).unwrap_or_default();
        let mut insert_cross_ref = conn.prepare(
            "INSERT INTO cross_references (from_book, from_chapter, from_verse, to_book, to_chapter, to_verse_start, to_verse_end, votes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;

        conn.execute_batch("BEGIN TRANSACTION")?;
        let mut count: i64 = 0;
        for line in cross_ref_raw.lines() {
            if line.starts_with("From") || line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let mut fields = line.split('\t');
            let (Some(from_str), Some(to_str)) = (fields.next(), fields.next()) else {
                continue;
            };
            let votes_str = fields.next();

            let Some(from) = parse_osis(from_str) else { continue; };
            let Some(to) = parse_osis(to_str) else { continue; };
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
            count += 1;
        }
        conn.execute_batch("COMMIT")?;
        println!("  ✓ {} cross-references imported", thousands(count));
    }

    // Optimize and checkpoint WAL away so file can be opened read-only immutable
    println!("\n  ⚡ Checkpointing and optimizing database...");
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA optimize; ANALYZE;")?;

    let verse_total: i64 = conn.query_row("SELECT COUNT(*) FROM verses", [], |r| r.get(0))?;
    let trans_total: i64 = conn.query_row("SELECT COUNT(*) FROM translations", [], |r| r.get(0))?;
    let cross_total: i64 = conn.query_row("SELECT COUNT(*) FROM cross_references", [], |r| r.get(0))?;

    println!("\n✅ rhema.db ready!");
    println!("   {} translations", trans_total);
    println!("   {} verses", thousands(verse_total));
    println!("   {} cross-references", thousands(cross_total));
    println!("   📁 {}\n", db_path.display());

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let root = project_root();

    let cmd = args.get(1).map(String::as_str).unwrap_or("setup");

    match cmd {
        "download-sources" => {
            let download_all = args.iter().any(|a| a == "--all");
            download_sources(&root, download_all)?;
        }
        "download-model" => {
            let model = args
                .iter()
                .position(|a| a == "--model")
                .and_then(|i| args.get(i + 1))
                .map(String::as_str)
                .unwrap_or("base.en");
            download_whisper_model(&root, model)?;
        }
        "build" => {
            let out = args
                .iter()
                .position(|a| a == "--out")
                .and_then(|i| args.get(i + 1))
                .map(PathBuf::from);
            build_database(&root, out.as_deref())?;
        }
        "setup" => {
            let download_all = args.iter().any(|a| a == "--all");
            let model = args
                .iter()
                .position(|a| a == "--model")
                .and_then(|i| args.get(i + 1))
                .map(String::as_str)
                .unwrap_or("base.en");

            println!("🚀 Running full setup for Logos data and models...");
            download_sources(&root, download_all)?;
            build_database(&root, None)?;
            download_whisper_model(&root, model)?;
            println!("🎉 Setup completed successfully!");
        }
        "help" | "--help" | "-h" => {
            println!("build_bible_db - Logos data and model CLI tool\n");
            println!("Usage:");
            println!("  cargo run -p build_bible_db -- [COMMAND] [OPTIONS]\n");
            println!("Commands:");
            println!("  setup [--model <name>] [--all]   Download sources, build rhema.db, download Whisper model (default)");
            println!("  download-sources [--all]         Download Bible sources (default: public domain; --all: full set)");
            println!("  download-model [--model <name>]  Download Whisper GGML model (default: base.en)");
            println!("  build [--out <path>]             Build SQLite database from downloaded sources");
        }
        other => {
            anyhow::bail!("Unknown command '{}'. Run with '--help' for usage.", other);
        }
    }

    Ok(())
}
