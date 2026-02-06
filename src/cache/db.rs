use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::cache::migrations;
use crate::model::{Priority, TodoItem, TodoTag};

pub struct CacheDb {
    conn: Connection,
}

impl CacheDb {
    /// Open or create cache database at .todo-tracker/cache.db
    pub fn open(root: &Path) -> Result<Self, String> {
        let cache_dir = root.join(".todo-tracker");
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache dir: {}", e))?;
        let db_path = cache_dir.join("cache.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open cache db: {}", e))?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| e.to_string())?;
        migrations::run_migrations(&conn).map_err(|e| e.to_string())?;
        Ok(CacheDb { conn })
    }

    /// Open an in-memory database (for testing)
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        migrations::run_migrations(&conn).map_err(|e| e.to_string())?;
        Ok(CacheDb { conn })
    }

    /// Check if a file needs rescanning by comparing mtime and size
    pub fn is_file_fresh(&self, path: &Path, mtime: u64, size: u64) -> bool {
        let path_str = path.display().to_string();
        let result: Result<(i64, i64), _> = self.conn.query_row(
            "SELECT mtime, size FROM file_fingerprints WHERE path = ?1",
            [&path_str],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        match result {
            Ok((cached_mtime, cached_size)) => {
                cached_mtime as u64 == mtime && cached_size as u64 == size
            }
            Err(_) => false,
        }
    }

    /// Get cached TODOs for a file
    pub fn get_todos(&self, path: &Path) -> Vec<TodoItem> {
        let path_str = path.display().to_string();
        let mut stmt = match self.conn.prepare(
            "SELECT file_path, line, col, tag, message, author, issue, priority, context_line \
             FROM todos WHERE file_path = ?1",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        let items = stmt.query_map([&path_str], |row| {
            let tag_str: String = row.get(3)?;
            let priority_str: Option<String> = row.get(7)?;
            Ok(TodoItem {
                file: PathBuf::from(row.get::<_, String>(0)?),
                line: row.get::<_, i64>(1)? as usize,
                column: row.get::<_, i64>(2)? as usize,
                tag: TodoTag::from_str(&tag_str),
                message: row.get(4)?,
                author: row.get(5)?,
                issue: row.get(6)?,
                priority: priority_str.and_then(|s| Priority::from_str_tag(&s)),
                context_line: row.get(8)?,
                git_author: None,
                git_date: None,
            })
        });

        match items {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(_) => vec![],
        }
    }

    /// Store file fingerprint and its TODOs
    pub fn store_file(
        &self,
        path: &Path,
        mtime: u64,
        size: u64,
        items: &[TodoItem],
    ) -> Result<(), String> {
        let path_str = path.display().to_string();

        // Update fingerprint
        self.conn
            .execute(
                "INSERT OR REPLACE INTO file_fingerprints (path, mtime, size) VALUES (?1, ?2, ?3)",
                rusqlite::params![path_str, mtime as i64, size as i64],
            )
            .map_err(|e| e.to_string())?;

        // Delete old TODOs for this file
        self.conn
            .execute("DELETE FROM todos WHERE file_path = ?1", [&path_str])
            .map_err(|e| e.to_string())?;

        // Insert new TODOs
        let mut stmt = self
            .conn
            .prepare(
                "INSERT INTO todos (file_path, line, col, tag, message, author, issue, priority, context_line) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )
            .map_err(|e| e.to_string())?;

        for item in items {
            let priority_str = item.priority.as_ref().map(|p| match p {
                Priority::Low => "low",
                Priority::Medium => "medium",
                Priority::High => "high",
                Priority::Critical => "critical",
            });
            stmt.execute(rusqlite::params![
                path_str,
                item.line as i64,
                item.column as i64,
                item.tag.as_str(),
                item.message,
                item.author,
                item.issue,
                priority_str,
                item.context_line,
            ])
            .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Clear all cached data
    pub fn clear(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "DELETE FROM todos; DELETE FROM file_fingerprints; DELETE FROM scan_meta;",
            )
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Priority, TodoTag};
    use std::path::PathBuf;

    fn make_todo(file: &str, line: usize, tag: TodoTag, message: &str) -> TodoItem {
        let context_line = format!("// {}: {}", tag.as_str(), message);
        TodoItem {
            file: PathBuf::from(file),
            line,
            column: 1,
            tag,
            message: message.to_string(),
            author: None,
            issue: None,
            priority: None,
            context_line,
            git_author: None,
            git_date: None,
        }
    }

    #[test]
    fn test_open_in_memory() {
        let db = CacheDb::open_in_memory().unwrap();
        // Should be able to clear without error
        db.clear().unwrap();
    }

    #[test]
    fn test_file_freshness_unknown_file() {
        let db = CacheDb::open_in_memory().unwrap();
        assert!(!db.is_file_fresh(Path::new("nonexistent.rs"), 100, 200));
    }

    #[test]
    fn test_store_and_retrieve_todos() {
        let db = CacheDb::open_in_memory().unwrap();
        let path = Path::new("src/main.rs");
        let items = vec![
            make_todo("src/main.rs", 10, TodoTag::Todo, "fix this"),
            make_todo("src/main.rs", 20, TodoTag::Fixme, "urgent fix"),
        ];

        db.store_file(path, 1000, 500, &items).unwrap();

        let retrieved = db.get_todos(path);
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].line, 10);
        assert_eq!(retrieved[0].tag, TodoTag::Todo);
        assert_eq!(retrieved[0].message, "fix this");
        assert_eq!(retrieved[1].line, 20);
        assert_eq!(retrieved[1].tag, TodoTag::Fixme);
        assert_eq!(retrieved[1].message, "urgent fix");
    }

    #[test]
    fn test_file_freshness_after_store() {
        let db = CacheDb::open_in_memory().unwrap();
        let path = Path::new("src/lib.rs");

        db.store_file(path, 1000, 500, &[]).unwrap();

        // Same mtime and size: fresh
        assert!(db.is_file_fresh(path, 1000, 500));

        // Different mtime: stale
        assert!(!db.is_file_fresh(path, 1001, 500));

        // Different size: stale
        assert!(!db.is_file_fresh(path, 1000, 501));
    }

    #[test]
    fn test_store_replaces_old_todos() {
        let db = CacheDb::open_in_memory().unwrap();
        let path = Path::new("src/main.rs");

        let items_v1 = vec![make_todo("src/main.rs", 10, TodoTag::Todo, "old task")];
        db.store_file(path, 1000, 500, &items_v1).unwrap();

        let items_v2 = vec![
            make_todo("src/main.rs", 5, TodoTag::Hack, "new hack"),
            make_todo("src/main.rs", 15, TodoTag::Bug, "new bug"),
        ];
        db.store_file(path, 1001, 600, &items_v2).unwrap();

        let retrieved = db.get_todos(path);
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].message, "new hack");
        assert_eq!(retrieved[1].message, "new bug");
    }

    #[test]
    fn test_store_with_priority_and_metadata() {
        let db = CacheDb::open_in_memory().unwrap();
        let path = Path::new("src/main.rs");

        let mut item = make_todo("src/main.rs", 10, TodoTag::Todo, "critical task");
        item.priority = Some(Priority::Critical);
        item.author = Some("alice".to_string());
        item.issue = Some("#123".to_string());

        db.store_file(path, 1000, 500, &[item]).unwrap();

        let retrieved = db.get_todos(path);
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].priority, Some(Priority::Critical));
        assert_eq!(retrieved[0].author, Some("alice".to_string()));
        assert_eq!(retrieved[0].issue, Some("#123".to_string()));
    }

    #[test]
    fn test_get_todos_empty_file() {
        let db = CacheDb::open_in_memory().unwrap();
        let retrieved = db.get_todos(Path::new("no/such/file.rs"));
        assert!(retrieved.is_empty());
    }

    #[test]
    fn test_clear() {
        let db = CacheDb::open_in_memory().unwrap();
        let path = Path::new("src/main.rs");
        let items = vec![make_todo("src/main.rs", 10, TodoTag::Todo, "task")];

        db.store_file(path, 1000, 500, &items).unwrap();
        assert!(db.is_file_fresh(path, 1000, 500));
        assert_eq!(db.get_todos(path).len(), 1);

        db.clear().unwrap();
        assert!(!db.is_file_fresh(path, 1000, 500));
        assert!(db.get_todos(path).is_empty());
    }
}
