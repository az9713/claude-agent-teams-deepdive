use std::path::Path;

use regex::Regex;

use crate::error::Result;
use crate::model::{Priority, TodoItem, TodoTag};
use crate::scanner::languages::{Language, LanguageDatabase};
use crate::scanner::FileScanner;

pub struct RegexScanner {
    pattern: Regex,
    metadata_pattern: Regex,
    language_db: LanguageDatabase,
}

impl RegexScanner {
    pub fn new() -> Result<Self> {
        let pattern = Regex::new(r"\b(TODO|FIXME|HACK|BUG|XXX)\b")?;
        let metadata_pattern = Regex::new(r"\b(TODO|FIXME|HACK|BUG|XXX)\(([^)]*)\)")?;
        Ok(RegexScanner {
            pattern,
            metadata_pattern,
            language_db: LanguageDatabase::new(),
        })
    }
}

/// Check if a trimmed line starts with any of the language's line comment prefixes.
fn is_line_comment(trimmed: &str, lang: &Language) -> bool {
    lang.line_comments
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
}

/// Parse metadata from the parenthesized content of a tag, e.g. "alice, #123, p:high".
/// Returns (author, issue, priority).
fn parse_metadata(contents: &str) -> (Option<String>, Option<String>, Option<Priority>) {
    let mut author: Option<String> = None;
    let mut issue: Option<String> = None;
    let mut priority: Option<Priority> = None;

    for part in contents.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.starts_with('#') {
            issue = Some(part.to_string());
        } else if let Some(p) = Priority::from_str_tag(part) {
            priority = Some(p);
        } else if author.is_none() {
            // First non-issue, non-priority token is the author
            author = Some(part.to_string());
        }
    }

    (author, issue, priority)
}

/// Extract the message text that follows a TODO tag (and optional metadata parens) on the line.
fn extract_message(line: &str, tag_start: usize, tag_end: usize) -> String {
    let rest = &line[tag_end..];

    // If the tag is followed by parenthesized metadata, skip past the closing paren
    let after_meta = if rest.starts_with('(') {
        if let Some(close) = rest.find(')') {
            &rest[close + 1..]
        } else {
            rest
        }
    } else {
        rest
    };

    // Strip leading punctuation/whitespace like ": " or " - "
    let msg = after_meta.trim_start_matches(|c: char| c == ':' || c == '-' || c.is_whitespace());

    // If there's nothing useful after the tag, use the whole line as context
    if msg.is_empty() {
        let before = line[..tag_start].trim();
        // Try to return just the comment text
        if before.is_empty() {
            return String::new();
        }
    }

    msg.to_string()
}

impl FileScanner for RegexScanner {
    fn scan_file(&self, path: &Path) -> Result<Vec<TodoItem>> {
        let content = std::fs::read_to_string(path)?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let language = self.language_db.from_extension(ext);

        let mut items = Vec::new();
        let mut block_depth: usize = 0;

        for (line_idx, line) in content.lines().enumerate() {
            let line_number = line_idx + 1;
            let trimmed = line.trim_start();

            // Track block comment depth and determine if this line is in a comment
            let in_comment = if let Some(lang) = language {
                let was_in_block = block_depth > 0;
                let mut entered_block_on_this_line = false;

                // Update block comment depth for this line
                if let (Some(start), Some(end)) = (lang.block_comment_start, lang.block_comment_end)
                {
                    let mut search_pos = 0;
                    let bytes = line.as_bytes();
                    while search_pos < bytes.len() {
                        let remaining = &line[search_pos..];
                        let next_start = remaining.find(start);
                        let next_end = if block_depth > 0 {
                            remaining.find(end)
                        } else {
                            None
                        };

                        match (next_start, next_end) {
                            (Some(s), Some(e)) if s < e => {
                                block_depth += 1;
                                entered_block_on_this_line = true;
                                search_pos += s + start.len();
                            }
                            (Some(s), None) => {
                                block_depth += 1;
                                entered_block_on_this_line = true;
                                search_pos += s + start.len();
                            }
                            (_, Some(e)) => {
                                block_depth = block_depth.saturating_sub(1);
                                search_pos += e + end.len();
                            }
                            (None, None) => break,
                        }
                    }
                }

                // Line is in a comment if:
                // 1. We were inside a block comment at the start of this line, or
                // 2. A block comment was opened on this line (e.g. /* TODO */ on one line), or
                // 3. The trimmed line starts with a line comment prefix
                was_in_block || entered_block_on_this_line || is_line_comment(trimmed, lang)
            } else {
                // Unknown language: scan all lines
                true
            };

            if !in_comment {
                continue;
            }

            // Try metadata pattern first (TAG with parens)
            for cap in self.metadata_pattern.captures_iter(line) {
                let full_match = cap.get(0).unwrap();
                let tag_str = &cap[1];
                let metadata_str = &cap[2];
                let tag = TodoTag::from_str(tag_str);
                let (author, issue, priority) = parse_metadata(metadata_str);
                let message =
                    extract_message(line, full_match.start(), full_match.end());

                items.push(TodoItem {
                    tag,
                    message,
                    file: path.to_path_buf(),
                    line: line_number,
                    column: full_match.start() + 1,
                    author,
                    issue,
                    priority,
                    context_line: line.to_string(),
                    git_author: None,
                    git_date: None,
                });
            }

            // If metadata pattern didn't match, try bare pattern
            if self.metadata_pattern.captures_iter(line).count() == 0 {
                for mat in self.pattern.find_iter(line) {
                    let tag = TodoTag::from_str(mat.as_str());
                    let message = extract_message(line, mat.start(), mat.end());

                    items.push(TodoItem {
                        tag,
                        message,
                        file: path.to_path_buf(),
                        line: line_number,
                        column: mat.start() + 1,
                        author: None,
                        issue: None,
                        priority: None,
                        context_line: line.to_string(),
                        git_author: None,
                        git_date: None,
                    });
                }
            }
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_file(content: &str, extension: &str) -> tempfile::TempPath {
        let mut file = tempfile::Builder::new()
            .suffix(&format!(".{}", extension))
            .tempfile()
            .unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.into_temp_path()
    }

    #[test]
    fn test_bare_todo_in_line_comment() {
        let scanner = RegexScanner::new().unwrap();
        let path = write_temp_file("// TODO fix this later\n", "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].message, "fix this later");
        assert_eq!(items[0].line, 1);
        assert!(items[0].author.is_none());
        assert!(items[0].issue.is_none());
        assert!(items[0].priority.is_none());
    }

    #[test]
    fn test_todo_with_author() {
        let scanner = RegexScanner::new().unwrap();
        let path = write_temp_file("// TODO(alice): refactor this\n", "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].author, Some("alice".to_string()));
        assert_eq!(items[0].message, "refactor this");
    }

    #[test]
    fn test_todo_with_author_and_issue() {
        let scanner = RegexScanner::new().unwrap();
        let path = write_temp_file("// FIXME(bob, #123): memory leak here\n", "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Fixme);
        assert_eq!(items[0].author, Some("bob".to_string()));
        assert_eq!(items[0].issue, Some("#123".to_string()));
        assert_eq!(items[0].message, "memory leak here");
    }

    #[test]
    fn test_todo_with_author_issue_and_priority() {
        let scanner = RegexScanner::new().unwrap();
        let path = write_temp_file("// HACK(carol, #456, p:high): temporary workaround\n", "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Hack);
        assert_eq!(items[0].author, Some("carol".to_string()));
        assert_eq!(items[0].issue, Some("#456".to_string()));
        assert_eq!(items[0].priority, Some(Priority::High));
        assert_eq!(items[0].message, "temporary workaround");
    }

    #[test]
    fn test_todo_with_priority_only() {
        let scanner = RegexScanner::new().unwrap();
        let path = write_temp_file("// BUG(p:critical): crashes on startup\n", "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Bug);
        assert!(items[0].author.is_none());
        assert_eq!(items[0].priority, Some(Priority::Critical));
        assert_eq!(items[0].message, "crashes on startup");
    }

    #[test]
    fn test_block_comment_todo() {
        let scanner = RegexScanner::new().unwrap();
        let content = "fn main() {\n    /* TODO: implement this */\n}\n";
        let path = write_temp_file(content, "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].line, 2);
    }

    #[test]
    fn test_multiline_block_comment_todo() {
        let scanner = RegexScanner::new().unwrap();
        let content = "/*\n * TODO: fix this\n * across lines\n */\nfn main() {}\n";
        let path = write_temp_file(content, "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].line, 2);
    }

    #[test]
    fn test_false_positive_rejection_in_code() {
        let scanner = RegexScanner::new().unwrap();
        let content = "let todo_list = vec![\"TODO\"];\nlet x = \"FIXME\";\n";
        let path = write_temp_file(content, "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        // These TODOs are in string literals, not in comments, so they should be rejected
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_multiple_todos_in_one_file() {
        let scanner = RegexScanner::new().unwrap();
        let content = "\
// TODO: first thing
fn foo() {}
// FIXME: second thing
fn bar() {}
// HACK: third thing
";
        let path = write_temp_file(content, "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[1].tag, TodoTag::Fixme);
        assert_eq!(items[2].tag, TodoTag::Hack);
        assert_eq!(items[0].line, 1);
        assert_eq!(items[1].line, 3);
        assert_eq!(items[2].line, 5);
    }

    #[test]
    fn test_python_hash_comment() {
        let scanner = RegexScanner::new().unwrap();
        let content = "# TODO: add error handling\ndef foo():\n    pass\n";
        let path = write_temp_file(content, "py");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
        assert_eq!(items[0].message, "add error handling");
    }

    #[test]
    fn test_python_rejects_non_comment_todo() {
        let scanner = RegexScanner::new().unwrap();
        let content = "todo_items = []\nprint(\"TODO\")\n";
        let path = write_temp_file(content, "py");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_javascript_block_comment() {
        let scanner = RegexScanner::new().unwrap();
        let content = "/* FIXME(bob, #42): broken sort */\nfunction sort() {}\n";
        let path = write_temp_file(content, "js");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Fixme);
        assert_eq!(items[0].author, Some("bob".to_string()));
        assert_eq!(items[0].issue, Some("#42".to_string()));
    }

    #[test]
    fn test_unknown_extension_scans_all_lines() {
        let scanner = RegexScanner::new().unwrap();
        let content = "TODO: this should be found\nnot a comment but unknown lang\n";
        let path = write_temp_file(content, "xyz");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Todo);
    }

    #[test]
    fn test_xxx_tag() {
        let scanner = RegexScanner::new().unwrap();
        let path = write_temp_file("// XXX: dangerous code\n", "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tag, TodoTag::Xxx);
    }

    #[test]
    fn test_column_number() {
        let scanner = RegexScanner::new().unwrap();
        let path = write_temp_file("    // TODO: indented\n", "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        // "    // TODO" - TODO starts at byte index 7 (0-based), column 8 (1-based)
        assert_eq!(items[0].column, 8);
    }

    #[test]
    fn test_context_line_preserved() {
        let scanner = RegexScanner::new().unwrap();
        let line = "    // TODO(alice): important fix";
        let path = write_temp_file(&format!("{}\n", line), "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].context_line, line);
    }

    #[test]
    fn test_issue_slug_format() {
        let scanner = RegexScanner::new().unwrap();
        let path = write_temp_file("// TODO(dave, #issue-slug): handle edge case\n", "rs");
        let items = scanner.scan_file(Path::new(&path)).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].issue, Some("#issue-slug".to_string()));
    }
}
