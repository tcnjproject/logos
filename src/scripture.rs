//! Repository boundary: the application depends on this trait, not SQLite queries.
use crate::{
    data::{SearchMode, Verse},
    detection::{BibleReferenceDetector, Reference, ReferenceDetector, normalize},
};
use rhema_bible::BibleDb;
use std::{path::Path, sync::Arc};

pub trait VerseRepository: Send + Sync + std::fmt::Debug {
    fn lookup(&self, reference: &Reference, translation: &str) -> Result<Vec<Verse>, String>;
    fn search(
        &self,
        query: &str,
        mode: &SearchMode,
        translation: &str,
    ) -> Result<Vec<Verse>, String>;
}

#[derive(Debug)]
pub struct SqliteVerseRepository {
    db: BibleDb,
}

impl SqliteVerseRepository {
    pub fn open(path: &Path) -> Result<Arc<dyn VerseRepository>, String> {
        if !path.is_file() {
            return Err(format!(
                "Bible database missing: {}. Set LOGOS_BIBLE_DB.",
                path.display()
            ));
        }
        let db = BibleDb::open(path).map_err(|e| e.to_string())?;
        db.list_translations()
            .map_err(|e| format!("Invalid Bible database: {e}"))?;
        Ok(Arc::new(Self { db }))
    }
    fn translation_id(&self, label: &str) -> Result<i64, String> {
        self.db.list_translations().map_err(|e| e.to_string())?.into_iter()
            .find(|t| t.abbreviation.eq_ignore_ascii_case(label) && t.is_downloaded)
            .map(|t| t.id).ok_or_else(|| format!("{label} is not installed in the Bible database. Select an installed translation or rebuild the database with its source."))
    }
    fn convert(rows: Vec<rhema_bible::Verse>, translation: &str) -> Vec<Verse> {
        rows.into_iter()
            .map(|v| {
                Verse::new(
                    &format!("{} {}:{}", v.book_name, v.chapter, v.verse),
                    &v.text,
                    translation,
                )
            })
            .collect()
    }
}

impl VerseRepository for SqliteVerseRepository {
    fn lookup(&self, r: &Reference, translation: &str) -> Result<Vec<Verse>, String> {
        let id = self.translation_id(translation)?;
        let rows = self
            .db
            .get_verse_range(id, r.book, r.chapter, r.first, r.last)
            .map_err(|e| e.to_string())?;
        if rows.len() != (r.last - r.first + 1) as usize {
            return Err("Reference is not present in the selected translation.".into());
        }
        Ok(Self::convert(rows, translation))
    }
    fn search(
        &self,
        query: &str,
        mode: &SearchMode,
        translation: &str,
    ) -> Result<Vec<Verse>, String> {
        let id = self.translation_id(translation)?;
        match mode {
            SearchMode::Book => {
                let refs = BibleReferenceDetector.detect(query);
                if !refs.is_empty() {
                    let mut results = Vec::new();
                    for r in refs {
                        results.extend(self.lookup(&r, translation)?);
                    }
                    return Ok(results);
                }
                let query = normalize(query);
                if let Some((name, chapter)) = query.rsplit_once(' ')
                    && let Ok(chapter) = chapter.parse::<i32>()
                {
                    let name = name.trim_end_matches(" chapter");
                    let books = self.db.search_books(name).map_err(|e| e.to_string())?;
                    if let Some(book) = books.into_iter().find(|b| b.translation_id == id) {
                        return self
                            .db
                            .get_chapter(id, book.book_number, chapter)
                            .map(|v| Self::convert(v, translation))
                            .map_err(|e| e.to_string());
                    }
                }
                Err("Enter a reference such as John 3:16, Romans 8:28-30, or Psalm 23.".into())
            }
            SearchMode::Context => {
                // Quote each term so user punctuation cannot become FTS query syntax.
                let terms = query
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|s| !s.is_empty())
                    .take(12)
                    .map(|s| format!("\"{s}\""))
                    .collect::<Vec<_>>()
                    .join(" AND ");
                if terms.is_empty() {
                    return Ok(Vec::new());
                }
                self.db
                    .search_verses(&terms, id, 30)
                    .map(|v| Self::convert(v, translation))
                    .map_err(|e| e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    struct Fixture {
        repository: Arc<dyn VerseRepository>,
        path: std::path::PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "logos-bible-test-{}-{}.db",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute_batch("CREATE TABLE translations (id INTEGER PRIMARY KEY, abbreviation TEXT, title TEXT, language TEXT, is_copyrighted INTEGER, is_downloaded INTEGER);
                CREATE TABLE books (id INTEGER PRIMARY KEY, translation_id INTEGER, book_number INTEGER, name TEXT, abbreviation TEXT, testament TEXT);
                CREATE TABLE verses (id INTEGER PRIMARY KEY, translation_id INTEGER, book_number INTEGER, book_name TEXT, book_abbreviation TEXT, chapter INTEGER, verse INTEGER, text TEXT);
                CREATE VIRTUAL TABLE verses_fts USING fts5(text);
                INSERT INTO translations VALUES (1, 'KJV', 'King James', 'en', 0, 1), (2, 'ESV', 'English Standard', 'en', 1, 1);
                INSERT INTO books VALUES (1,1,43,'John','Jn','NT'), (2,2,43,'John','Jn','NT');
                INSERT INTO verses VALUES (1,1,43,'John','Jn',3,16,'For God so loved the world'), (2,1,43,'John','Jn',3,17,'For God sent not his Son'), (3,2,43,'John','Jn',3,16,'For God so loved the world ESV');
                INSERT INTO verses_fts(rowid,text) SELECT id,text FROM verses;").unwrap();
            drop(conn);
            Self {
                repository: SqliteVerseRepository::open(&path).unwrap(),
                path,
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }
    #[test]
    fn detected_references_load_exact_translation_and_ranges() {
        let fixture = Fixture::new();
        let refs =
            BibleReferenceDetector.detect("John chapter three verses sixteen through seventeen");
        let verses = fixture.repository.lookup(&refs[0], "KJV").unwrap();
        assert_eq!(verses.len(), 2);
        assert_eq!(verses[0].reference, "John 3:16");
        assert_eq!(verses[0].translation, "KJV");
        let esv = fixture
            .repository
            .search("John 3:16", &SearchMode::Book, "ESV")
            .unwrap();
        assert!(esv[0].text.ends_with("ESV"));
        assert!(fixture.repository.lookup(&refs[0], "ESV").is_err());
        assert!(
            fixture
                .repository
                .lookup(&refs[0], "NIV")
                .unwrap_err()
                .contains("not installed")
        );
    }
    #[test]
    fn chapter_and_context_search_are_database_backed() {
        let fixture = Fixture::new();
        assert_eq!(
            fixture
                .repository
                .search("John 3", &SearchMode::Book, "KJV")
                .unwrap()
                .len(),
            2
        );
        let verses = fixture
            .repository
            .search("God loved", &SearchMode::Context, "KJV")
            .unwrap();
        assert_eq!(verses.len(), 1);
        assert_eq!(verses[0].reference, "John 3:16");
        assert!(
            fixture
                .repository
                .search("\" * ()", &SearchMode::Context, "KJV")
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn missing_database_is_not_created() {
        let path = std::env::temp_dir().join(format!("logos-missing-{}.db", std::process::id()));
        assert!(SqliteVerseRepository::open(&path).is_err());
        assert!(!path.exists());
    }
}
