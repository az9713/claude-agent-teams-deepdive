use std::fs;
use std::path::Path;

use crate::cache::CacheDb;
use crate::error::Result;
use crate::model::TodoItem;
use crate::scanner::FileScanner;

pub struct IncrementalScanner<'a> {
    scanner: &'a dyn FileScanner,
    cache: &'a CacheDb,
}

impl<'a> IncrementalScanner<'a> {
    pub fn new(scanner: &'a dyn FileScanner, cache: &'a CacheDb) -> Self {
        Self { scanner, cache }
    }

    /// Scan a file, using cache if fingerprint matches.
    /// Returns (items, from_cache) where from_cache indicates if results came from cache.
    pub fn scan_file(&self, path: &Path) -> Result<(Vec<TodoItem>, bool)> {
        let metadata = fs::metadata(path)?;
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let size = metadata.len();

        // Check cache
        if self.cache.is_file_fresh(path, mtime, size) {
            let items = self.cache.get_todos(path);
            return Ok((items, true));
        }

        // Scan and cache
        let items = self.scanner.scan_file(path)?;
        let _ = self.cache.store_file(path, mtime, size, &items);
        Ok((items, false))
    }
}
