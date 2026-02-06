use std::path::Path;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

use crate::error::{Result, TodoError};
use crate::model::TodoItem;
use crate::scanner::languages::LanguageDatabase;
use crate::scanner::regex::RegexScanner;
use crate::scanner::FileScanner;

/// Statistics for precision scanning accuracy.
#[derive(Debug, Clone, Default)]
pub struct PrecisionStats {
    pub total_candidates: usize,
    pub verified: usize,
    pub filtered_false_positives: usize,
}

impl PrecisionStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accuracy_percentage(&self) -> f64 {
        if self.total_candidates == 0 {
            return 100.0;
        }
        (self.verified as f64 / self.total_candidates as f64) * 100.0
    }

    pub fn print_if_filtered(&self) {
        if self.filtered_false_positives > 0 {
            eprintln!(
                "[TreeSitter] Filtered {} false positives from {} candidates ({:.1}% accuracy)",
                self.filtered_false_positives,
                self.total_candidates,
                self.accuracy_percentage()
            );
        }
    }
}

/// Tree-sitter based precision scanner that verifies regex candidates against AST comment nodes.
pub struct TreeSitterScanner {
    inner: RegexScanner,
}

impl TreeSitterScanner {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: RegexScanner::new()?,
        })
    }

    /// Get the tree-sitter language for a given language name.
    fn get_tree_sitter_language(language_name: &str) -> Option<Language> {
        match language_name {
            "Rust" => Some(tree_sitter_rust::LANGUAGE.into()),
            "JavaScript" => Some(tree_sitter_javascript::LANGUAGE.into()),
            "TypeScript" => Some(tree_sitter_javascript::LANGUAGE.into()), // Use JS grammar for TS
            "Python" => Some(tree_sitter_python::LANGUAGE.into()),
            "Go" => Some(tree_sitter_go::LANGUAGE.into()),
            "Java" => Some(tree_sitter_java::LANGUAGE.into()),
            "C" => Some(tree_sitter_c::LANGUAGE.into()),
            "C++" => Some(tree_sitter_cpp::LANGUAGE.into()),
            "Ruby" => Some(tree_sitter_ruby::LANGUAGE.into()),
            _ => None,
        }
    }

    /// Extract all comment node byte ranges from the parsed tree.
    fn extract_comment_ranges(
        language: Language,
        source_code: &str,
    ) -> Result<Vec<(usize, usize)>> {
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .map_err(|e| TodoError::Scan {
                file: "treesitter".to_string(),
                message: format!("Failed to set language: {}", e),
            })?;

        let tree = parser.parse(source_code, None).ok_or_else(|| {
            TodoError::Scan {
                file: "treesitter".to_string(),
                message: "Failed to parse source code".to_string(),
            }
        })?;

        // Query for comment nodes - tree-sitter comment nodes are typically named "comment"
        let query_string = "(comment) @comment";
        let query = Query::new(&language, query_string).map_err(|e| TodoError::Scan {
            file: "treesitter".to_string(),
            message: format!("Failed to create comment query: {}", e),
        })?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

        let mut ranges = Vec::new();
        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                ranges.push((node.start_byte(), node.end_byte()));
            }
        }

        Ok(ranges)
    }

    /// Check if a line number falls within any of the comment ranges.
    fn is_line_in_comments(
        line_number: usize,
        comment_ranges: &[(usize, usize)],
        source_code: &str,
    ) -> bool {
        // Convert line number to byte offset range
        let line_start = Self::line_number_to_byte_offset(source_code, line_number);
        if line_start.is_none() {
            return false;
        }
        let line_start = line_start.unwrap();

        // Check if this byte offset falls within any comment range
        for (start, end) in comment_ranges {
            if line_start >= *start && line_start < *end {
                return true;
            }
        }

        false
    }

    /// Convert a 1-based line number to a byte offset (start of that line).
    fn line_number_to_byte_offset(source_code: &str, line_number: usize) -> Option<usize> {
        if line_number == 0 {
            return None;
        }

        let mut current_line = 1;
        let mut byte_offset = 0;

        if line_number == 1 {
            return Some(0);
        }

        for byte in source_code.as_bytes() {
            if *byte == b'\n' {
                current_line += 1;
                if current_line == line_number {
                    return Some(byte_offset + 1);
                }
            }
            byte_offset += 1;
        }

        // If we've reached the end and we're on the target line
        if current_line == line_number {
            return Some(byte_offset);
        }

        None
    }

    /// Verify candidates against tree-sitter AST and return verified items with stats.
    fn verify_candidates(
        candidates: Vec<TodoItem>,
        comment_ranges: &[(usize, usize)],
        source_code: &str,
    ) -> (Vec<TodoItem>, PrecisionStats) {
        let mut stats = PrecisionStats::new();
        stats.total_candidates = candidates.len();

        let verified: Vec<TodoItem> = candidates
            .into_iter()
            .filter(|item| {
                let is_valid = Self::is_line_in_comments(item.line, comment_ranges, source_code);
                if is_valid {
                    stats.verified += 1;
                } else {
                    stats.filtered_false_positives += 1;
                }
                is_valid
            })
            .collect();

        (verified, stats)
    }
}

impl FileScanner for TreeSitterScanner {
    fn scan_file(&self, path: &Path) -> Result<Vec<TodoItem>> {
        // First, get regex candidates
        let candidates = self.inner.scan_file(path)?;

        // If no candidates, return early
        if candidates.is_empty() {
            return Ok(candidates);
        }

        // Get the language for this file
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang_db = LanguageDatabase::new();
        let language_info = lang_db.from_extension(ext);

        // If we don't know this language, or can't get a tree-sitter grammar, fall back to regex results
        let language_name = match language_info {
            Some(lang) => lang.name,
            None => return Ok(candidates), // Unknown language, keep all candidates
        };

        let ts_language = match Self::get_tree_sitter_language(language_name) {
            Some(lang) => lang,
            None => return Ok(candidates), // No tree-sitter grammar available, keep all candidates
        };

        // Read the file contents
        let source_code = crate::scanner::mmap::read_file_contents(path)?;

        // Extract comment ranges from tree-sitter
        let comment_ranges = match Self::extract_comment_ranges(ts_language, &source_code) {
            Ok(ranges) => ranges,
            Err(_) => return Ok(candidates), // Parse error, fall back to regex results
        };

        // Verify candidates against comment ranges
        let (verified, stats) = Self::verify_candidates(candidates, &comment_ranges, &source_code);

        // Print stats if we filtered anything
        stats.print_if_filtered();

        Ok(verified)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Priority, TodoTag};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp_file(content: &str, extension: &str) -> NamedTempFile {
        let mut file = tempfile::Builder::new()
            .suffix(&format!(".{}", extension))
            .tempfile()
            .unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_comment_detection_rust() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "// TODO: fix this\nfn main() {}\n";
        let file = write_temp_file(content, "rs");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].message, "fix this");
    }

    #[test]
    fn test_false_positive_rejection_rust() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = r#"fn main() {
    let todo = "TODO: not a real todo";
    println!("{}", todo);
}
"#;
        let file = write_temp_file(content, "rs");
        let items = scanner.scan_file(file.path()).unwrap();

        // The TODO is in a string literal, not a comment, should be filtered out
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_block_comment_detection_rust() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "/* TODO: refactor this */\nfn main() {}\n";
        let file = write_temp_file(content, "rs");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
    }

    #[test]
    fn test_multiline_block_comment_rust() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "/*\n * TODO: important task\n * more details\n */\nfn main() {}\n";
        let file = write_temp_file(content, "rs");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].line, 2);
    }

    #[test]
    fn test_mixed_valid_and_invalid_todos() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = r#"// TODO: real todo in comment
fn main() {
    let x = "TODO: fake todo in string";
    // FIXME: another real todo
}
"#;
        let file = write_temp_file(content, "rs");
        let items = scanner.scan_file(file.path()).unwrap();

        // Should only find the 2 real TODOs in comments, not the one in the string
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].line, 1);
        assert_eq!(items[1].tag, TodoTag::Fixme);
        assert_eq!(items[1].line, 4);
    }

    #[test]
    fn test_python_comment_detection() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "# TODO: add error handling\ndef foo():\n    pass\n";
        let file = write_temp_file(content, "py");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
    }

    #[test]
    fn test_python_false_positive_rejection() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "todo = 'TODO: not in comment'\nprint(todo)\n";
        let file = write_temp_file(content, "py");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_javascript_comment_detection() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "// TODO: implement feature\nfunction foo() {}\n";
        let file = write_temp_file(content, "js");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
    }

    #[test]
    fn test_javascript_block_comment() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "/* FIXME: broken logic */\nfunction bar() {}\n";
        let file = write_temp_file(content, "js");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Fixme);
    }

    #[test]
    fn test_unknown_language_fallback() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "TODO: should be found\nsome unknown language content\n";
        let file = write_temp_file(content, "xyz");
        let items = scanner.scan_file(file.path()).unwrap();

        // Unknown language should keep all regex matches (fallback behavior)
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
    }

    #[test]
    fn test_metadata_preserved() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "// TODO(alice, #123, p:high): critical fix\nfn main() {}\n";
        let file = write_temp_file(content, "rs");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].author, Some("alice".to_string()));
        assert_eq!(items[0].issue, Some("#123".to_string()));
        assert_eq!(items[0].priority, Some(Priority::High));
        assert_eq!(items[0].message, "critical fix");
    }

    #[test]
    fn test_accuracy_stats_computation() {
        let mut stats = PrecisionStats::new();
        stats.total_candidates = 10;
        stats.verified = 7;
        stats.filtered_false_positives = 3;

        assert_eq!(stats.accuracy_percentage(), 70.0);
    }

    #[test]
    fn test_accuracy_stats_zero_candidates() {
        let stats = PrecisionStats::new();
        assert_eq!(stats.accuracy_percentage(), 100.0);
    }

    #[test]
    fn test_accuracy_stats_all_valid() {
        let mut stats = PrecisionStats::new();
        stats.total_candidates = 5;
        stats.verified = 5;
        stats.filtered_false_positives = 0;

        assert_eq!(stats.accuracy_percentage(), 100.0);
    }

    #[test]
    fn test_line_number_to_byte_offset() {
        let source = "line1\nline2\nline3\n";

        assert_eq!(TreeSitterScanner::line_number_to_byte_offset(source, 1), Some(0));
        assert_eq!(TreeSitterScanner::line_number_to_byte_offset(source, 2), Some(6));
        assert_eq!(TreeSitterScanner::line_number_to_byte_offset(source, 3), Some(12));
        assert_eq!(TreeSitterScanner::line_number_to_byte_offset(source, 0), None);
        assert_eq!(TreeSitterScanner::line_number_to_byte_offset(source, 999), None);
    }

    #[test]
    fn test_go_comment_detection() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "// TODO: add tests\nfunc main() {}\n";
        let file = write_temp_file(content, "go");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
    }

    #[test]
    fn test_java_comment_detection() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "// FIXME: memory leak\npublic class Test {}\n";
        let file = write_temp_file(content, "java");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Fixme);
    }

    #[test]
    fn test_c_comment_detection() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "/* TODO: optimize */\nint main() { return 0; }\n";
        let file = write_temp_file(content, "c");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
    }

    #[test]
    fn test_cpp_comment_detection() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "// HACK: temporary fix\nint main() { return 0; }\n";
        let file = write_temp_file(content, "cpp");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Hack);
    }

    #[test]
    fn test_ruby_comment_detection() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "# BUG: crashes on nil\ndef foo; end\n";
        let file = write_temp_file(content, "rb");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Bug);
    }

    #[test]
    fn test_typescript_uses_javascript_grammar() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "// TODO: add type annotations\nfunction foo(): void {}\n";
        let file = write_temp_file(content, "ts");
        let items = scanner.scan_file(file.path()).unwrap();

        // TypeScript should fall back to JavaScript grammar
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
    }

    #[test]
    fn test_empty_file() {
        let scanner = TreeSitterScanner::new().unwrap();
        let file = write_temp_file("", "rs");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_no_todos() {
        let scanner = TreeSitterScanner::new().unwrap();
        let content = "// Just a regular comment\nfn main() {}\n";
        let file = write_temp_file(content, "rs");
        let items = scanner.scan_file(file.path()).unwrap();

        assert_eq!(items.len(), 0);
    }
}
