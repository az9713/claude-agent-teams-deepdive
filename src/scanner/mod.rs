pub mod languages;
pub mod regex;
pub mod incremental;
pub mod mmap;
#[cfg(feature = "precise")]
pub mod treesitter;

use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;

use crate::cache::CacheDb;
use crate::discovery::FileDiscovery;
use crate::error::Result;
use crate::model::{ScanMetadata, ScanResult, ScanStats, TodoItem};
use crate::progress::ScanProgress;
use crate::scanner::incremental::IncrementalScanner;

pub trait FileScanner: Send + Sync {
    fn scan_file(&self, path: &Path) -> Result<Vec<TodoItem>>;
}

pub struct ScanOrchestrator {
    scanner: Box<dyn FileScanner>,
    discovery: FileDiscovery,
}

impl ScanOrchestrator {
    pub fn new(scanner: Box<dyn FileScanner>, discovery: FileDiscovery) -> Self {
        Self { scanner, discovery }
    }

    pub fn scan(&self) -> Result<ScanResult> {
        let start = Instant::now();

        let files = self.discovery.discover()?;
        let files_scanned = files.len();

        let mut all_items: Vec<TodoItem> = files
            .par_iter()
            .filter_map(|path| self.scanner.scan_file(path).ok())
            .flatten()
            .collect();

        all_items.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

        // Build stats
        let files_with_todos = all_items
            .iter()
            .map(|item| &item.file)
            .collect::<HashSet<_>>()
            .len();

        let mut stats = ScanStats::new();
        stats.files_scanned = files_scanned;
        stats.files_with_todos = files_with_todos;
        for item in &all_items {
            stats.add_item(item);
        }

        let elapsed = start.elapsed();
        let metadata = ScanMetadata {
            scan_duration_ms: elapsed.as_millis() as u64,
            root_path: self.discovery.root().to_path_buf(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
        };

        Ok(ScanResult {
            items: all_items,
            stats,
            metadata,
        })
    }

    /// Scan with optional cache support for incremental scanning.
    pub fn scan_with_cache(&self, cache: Option<&CacheDb>) -> Result<ScanResult> {
        let cache = match cache {
            Some(c) => c,
            None => return self.scan(),
        };

        let start = Instant::now();
        let files = self.discovery.discover()?;
        let files_scanned = files.len();
        let progress = ScanProgress::new(files_scanned as u64);

        let incremental = IncrementalScanner::new(self.scanner.as_ref(), cache);

        let mut all_items: Vec<TodoItem> = Vec::new();
        let mut from_cache_count: usize = 0;

        // Use sequential iteration for cache (SQLite is single-writer)
        for path in &files {
            match incremental.scan_file(path) {
                Ok((items, was_cached)) => {
                    if was_cached {
                        from_cache_count += 1;
                    }
                    all_items.extend(items);
                }
                Err(_) => {
                    // Fallback: try direct scan
                    if let Ok(items) = self.scanner.scan_file(path) {
                        all_items.extend(items);
                    }
                }
            }
            progress.inc();
        }

        progress.finish();

        all_items.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

        let files_with_todos = all_items
            .iter()
            .map(|item| &item.file)
            .collect::<HashSet<_>>()
            .len();

        let mut stats = ScanStats::new();
        stats.files_scanned = files_scanned;
        stats.files_with_todos = files_with_todos;
        for item in &all_items {
            stats.add_item(item);
        }

        let elapsed = start.elapsed();
        let metadata = ScanMetadata {
            scan_duration_ms: elapsed.as_millis() as u64,
            root_path: self.discovery.root().to_path_buf(),
            timestamp: format!("{:?}", std::time::SystemTime::now()),
        };

        if from_cache_count > 0 {
            eprintln!(
                "Scanned {} files ({} from cache) in {}ms",
                files_scanned, from_cache_count, elapsed.as_millis()
            );
        }

        Ok(ScanResult {
            items: all_items,
            stats,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::TodoTag;
    use std::path::PathBuf;
    use tempfile::TempDir;

    struct MockScanner {
        items: Vec<TodoItem>,
    }

    impl MockScanner {
        fn new(items: Vec<TodoItem>) -> Self {
            Self { items }
        }
    }

    impl FileScanner for MockScanner {
        fn scan_file(&self, path: &Path) -> Result<Vec<TodoItem>> {
            Ok(self
                .items
                .iter()
                .filter(|item| item.file == path)
                .cloned()
                .collect())
        }
    }

    fn make_todo(file: &str, line: usize, tag: TodoTag, message: &str) -> TodoItem {
        TodoItem {
            tag,
            message: message.to_string(),
            file: PathBuf::from(file),
            line,
            column: 1,
            author: None,
            issue: None,
            priority: None,
            context_line: String::new(),
            git_author: None,
            git_date: None,
        }
    }

    #[test]
    fn test_orchestrator_scan_empty() {
        let dir = TempDir::new().unwrap();
        let discovery = FileDiscovery::new(dir.path());
        let scanner = MockScanner::new(vec![]);
        let orchestrator = ScanOrchestrator::new(Box::new(scanner), discovery);

        let result = orchestrator.scan().unwrap();
        assert_eq!(result.items.len(), 0);
        assert_eq!(result.stats.files_scanned, 0);
        assert_eq!(result.stats.total_todos, 0);
    }

    #[test]
    fn test_orchestrator_scan_with_items() {
        let dir = TempDir::new().unwrap();
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");
        std::fs::write(&file_a, "// TODO: task a").unwrap();
        std::fs::write(&file_b, "// FIXME: fix b").unwrap();

        let items = vec![
            make_todo(
                file_a.to_str().unwrap(),
                1,
                TodoTag::Todo,
                "task a",
            ),
            make_todo(
                file_b.to_str().unwrap(),
                1,
                TodoTag::Fixme,
                "fix b",
            ),
        ];

        let discovery = FileDiscovery::new(dir.path());
        let scanner = MockScanner::new(items);
        let orchestrator = ScanOrchestrator::new(Box::new(scanner), discovery);

        let result = orchestrator.scan().unwrap();
        assert_eq!(result.items.len(), 2);
        assert_eq!(result.stats.files_scanned, 2);
        assert_eq!(result.stats.files_with_todos, 2);
        assert_eq!(result.stats.total_todos, 2);
        assert_eq!(result.stats.by_tag.get("TODO"), Some(&1));
        assert_eq!(result.stats.by_tag.get("FIXME"), Some(&1));
    }

    #[test]
    fn test_orchestrator_items_sorted() {
        let dir = TempDir::new().unwrap();
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");
        std::fs::write(&file_a, "// line1\n// TODO: second").unwrap();
        std::fs::write(&file_b, "// TODO: first").unwrap();

        let items = vec![
            make_todo(file_b.to_str().unwrap(), 1, TodoTag::Todo, "first"),
            make_todo(file_a.to_str().unwrap(), 5, TodoTag::Todo, "second"),
            make_todo(file_a.to_str().unwrap(), 2, TodoTag::Todo, "first in a"),
        ];

        let discovery = FileDiscovery::new(dir.path());
        let scanner = MockScanner::new(items);
        let orchestrator = ScanOrchestrator::new(Box::new(scanner), discovery);

        let result = orchestrator.scan().unwrap();
        // Items should be sorted by file path, then line number
        let files_and_lines: Vec<_> = result
            .items
            .iter()
            .map(|i| (i.file.clone(), i.line))
            .collect();
        let is_sorted = files_and_lines
            .windows(2)
            .all(|w| w[0].0 < w[1].0 || (w[0].0 == w[1].0 && w[0].1 <= w[1].1));
        assert!(is_sorted);
    }

    #[test]
    fn test_orchestrator_metadata() {
        let dir = TempDir::new().unwrap();
        let discovery = FileDiscovery::new(dir.path());
        let scanner = MockScanner::new(vec![]);
        let orchestrator = ScanOrchestrator::new(Box::new(scanner), discovery);

        let result = orchestrator.scan().unwrap();
        assert_eq!(result.metadata.root_path, dir.path());
        assert!(!result.metadata.timestamp.is_empty());
    }
}
