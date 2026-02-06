use std::collections::BTreeMap;
use std::fmt::Write;

use crate::error::Result;
use crate::model::{Priority, ScanResult, TodoItem};
use crate::output::OutputFormatter;

pub struct MarkdownFormatter;

impl OutputFormatter for MarkdownFormatter {
    fn format(&self, result: &ScanResult) -> Result<String> {
        let mut out = String::new();

        writeln!(out, "# TODO Report").unwrap();
        writeln!(out).unwrap();

        if result.items.is_empty() {
            writeln!(out, "No TODO items found.").unwrap();
            writeln!(out).unwrap();
            write_summary(&mut out, result);
            return Ok(out);
        }

        // Group items by file path
        let mut groups: BTreeMap<String, Vec<&TodoItem>> = BTreeMap::new();
        for item in &result.items {
            let path = item.file.display().to_string();
            groups.entry(path).or_default().push(item);
        }

        for (path, items) in &groups {
            writeln!(out, "## {}", path).unwrap();
            writeln!(out).unwrap();

            for item in items {
                let meta = format_metadata(item);
                write!(
                    out,
                    "- **{}** (L{}): {}",
                    item.tag.as_str(),
                    item.line,
                    item.message
                )
                .unwrap();
                if !meta.is_empty() {
                    write!(out, " *{}*", meta).unwrap();
                }
                writeln!(out).unwrap();
            }
            writeln!(out).unwrap();
        }

        write_summary(&mut out, result);

        Ok(out)
    }
}

fn format_metadata(item: &TodoItem) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(ref author) = item.author {
        parts.push(author.clone());
    }

    if let Some(ref issue) = item.issue {
        if issue.starts_with('#') {
            parts.push(issue.clone());
        } else {
            parts.push(format!("#{}", issue));
        }
    }

    if let Some(ref priority) = item.priority {
        let p = match priority {
            Priority::Low => "p:low",
            Priority::Medium => "p:medium",
            Priority::High => "p:high",
            Priority::Critical => "p:critical",
        };
        parts.push(p.to_string());
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("({})", parts.join(", "))
    }
}

fn write_summary(out: &mut String, result: &ScanResult) {
    writeln!(out, "---").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "**{} TODOs** in {} files (scanned {} files in {}ms)",
        result.stats.total_todos,
        result.stats.files_with_todos,
        result.stats.files_scanned,
        result.metadata.scan_duration_ms,
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Priority, ScanMetadata, ScanStats, TodoItem, TodoTag};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn sample_result() -> ScanResult {
        let items = vec![
            TodoItem {
                tag: TodoTag::Todo,
                message: "Add error handling".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 12,
                column: 5,
                author: Some("alice".to_string()),
                issue: Some("123".to_string()),
                priority: None,
                context_line: "// TODO(alice): Add error handling #123".to_string(),
                git_author: None,
                git_date: None,
            },
            TodoItem {
                tag: TodoTag::Fixme,
                message: "This is broken".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 45,
                column: 3,
                author: None,
                issue: None,
                priority: Some(Priority::High),
                context_line: "// FIXME: This is broken".to_string(),
                git_author: None,
                git_date: None,
            },
            TodoItem {
                tag: TodoTag::Hack,
                message: "Temporary workaround".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 3,
                column: 1,
                author: None,
                issue: None,
                priority: None,
                context_line: "// HACK: Temporary workaround".to_string(),
                git_author: None,
                git_date: None,
            },
        ];

        let mut by_tag = HashMap::new();
        by_tag.insert("TODO".to_string(), 1);
        by_tag.insert("FIXME".to_string(), 1);
        by_tag.insert("HACK".to_string(), 1);

        ScanResult {
            items,
            stats: ScanStats {
                files_scanned: 15,
                files_with_todos: 2,
                total_todos: 3,
                by_tag,
            },
            metadata: ScanMetadata {
                scan_duration_ms: 42,
                root_path: PathBuf::from("."),
                timestamp: "2026-02-05T00:00:00Z".to_string(),
            },
        }
    }

    #[test]
    fn test_markdown_has_title() {
        let formatter = MarkdownFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        assert!(output.starts_with("# TODO Report\n"));
    }

    #[test]
    fn test_markdown_has_file_headings() {
        let formatter = MarkdownFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("## src/main.rs"), "Should have file heading for src/main.rs");
        assert!(output.contains("## src/lib.rs"), "Should have file heading for src/lib.rs");
    }

    #[test]
    fn test_markdown_has_items_with_tags() {
        let formatter = MarkdownFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("**TODO** (L12): Add error handling"));
        assert!(output.contains("**FIXME** (L45): This is broken"));
        assert!(output.contains("**HACK** (L3): Temporary workaround"));
    }

    #[test]
    fn test_markdown_has_metadata() {
        let formatter = MarkdownFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("*(alice, #123)*"), "Should show author and issue");
        assert!(output.contains("*(p:high)*"), "Should show priority");
    }

    #[test]
    fn test_markdown_has_summary() {
        let formatter = MarkdownFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("---"), "Should have horizontal rule before summary");
        assert!(output.contains("**3 TODOs** in 2 files"));
        assert!(output.contains("scanned 15 files in 42ms"));
    }

    #[test]
    fn test_markdown_empty_result() {
        let formatter = MarkdownFormatter;
        let result = ScanResult {
            items: vec![],
            stats: ScanStats {
                files_scanned: 5,
                files_with_todos: 0,
                total_todos: 0,
                by_tag: HashMap::new(),
            },
            metadata: ScanMetadata {
                scan_duration_ms: 10,
                root_path: PathBuf::from("."),
                timestamp: "2026-02-05T00:00:00Z".to_string(),
            },
        };
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("No TODO items found."));
        assert!(output.contains("**0 TODOs** in 0 files"));
    }

    #[test]
    fn test_markdown_no_metadata_no_italic() {
        let formatter = MarkdownFormatter;
        let items = vec![TodoItem {
            tag: TodoTag::Hack,
            message: "Temporary workaround".to_string(),
            file: PathBuf::from("src/lib.rs"),
            line: 3,
            column: 1,
            author: None,
            issue: None,
            priority: None,
            context_line: "// HACK: Temporary workaround".to_string(),
            git_author: None,
            git_date: None,
        }];

        let mut by_tag = HashMap::new();
        by_tag.insert("HACK".to_string(), 1);

        let result = ScanResult {
            items,
            stats: ScanStats {
                files_scanned: 1,
                files_with_todos: 1,
                total_todos: 1,
                by_tag,
            },
            metadata: ScanMetadata {
                scan_duration_ms: 1,
                root_path: PathBuf::from("."),
                timestamp: "2026-02-05T00:00:00Z".to_string(),
            },
        };
        let output = formatter.format(&result).unwrap();
        // The line should end with the message, no italic metadata
        assert!(output.contains("- **HACK** (L3): Temporary workaround\n"));
    }
}
