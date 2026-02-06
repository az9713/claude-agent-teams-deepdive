use std::collections::BTreeMap;
use std::fmt::Write;

use colored::*;

use crate::error::Result;
use crate::model::{Priority, ScanResult, TodoItem, TodoTag};
use crate::output::OutputFormatter;

pub struct TextFormatter {
    pub show_summary: bool,
}

impl OutputFormatter for TextFormatter {
    fn format(&self, result: &ScanResult) -> Result<String> {
        let mut out = String::new();

        // Group items by file path
        let mut groups: BTreeMap<String, Vec<&TodoItem>> = BTreeMap::new();
        for item in &result.items {
            let path = item.file.display().to_string();
            groups.entry(path).or_default().push(item);
        }

        let mut first_group = true;
        for (path, items) in &groups {
            if !first_group {
                writeln!(out).unwrap();
            }
            first_group = false;

            // File path in bold
            writeln!(out, "{}", path.bold()).unwrap();

            for item in items {
                let line_str = format!("L{}", item.line);
                let tag_str = colorize_tag(&item.tag);
                let meta = format_metadata(item);

                write!(
                    out,
                    "  {:>5}  {:<6} {}",
                    line_str.dimmed().cyan(),
                    tag_str,
                    item.message
                )
                .unwrap();

                if !meta.is_empty() {
                    write!(out, " {}", meta.dimmed()).unwrap();
                }
                writeln!(out).unwrap();
            }
        }

        if self.show_summary {
            writeln!(out).unwrap();
            writeln!(out, "{}", format_summary_rule()).unwrap();
            writeln!(
                out,
                "{} TODOs in {} files (scanned {} files in {}ms)",
                result.stats.total_todos,
                result.stats.files_with_todos,
                result.stats.files_scanned,
                result.metadata.scan_duration_ms,
            )
            .unwrap();

            let breakdown = format_tag_breakdown(&result.stats.by_tag);
            if !breakdown.is_empty() {
                writeln!(out, "  {}", breakdown).unwrap();
            }
        }

        Ok(out)
    }
}

fn colorize_tag(tag: &TodoTag) -> ColoredString {
    let s = tag.as_str();
    match tag {
        TodoTag::Todo => s.yellow(),
        TodoTag::Fixme => s.red(),
        TodoTag::Hack => s.magenta(),
        TodoTag::Bug => s.red().bold(),
        TodoTag::Xxx => s.magenta().bold(),
        TodoTag::Custom(_) => s.white(),
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

fn format_summary_rule() -> String {
    format!("\u{2500}\u{2500} Summary {}", "\u{2500}".repeat(30))
}

fn format_tag_breakdown(by_tag: &std::collections::HashMap<String, usize>) -> String {
    // Use a fixed order for known tags, then alphabetical for custom
    let known_order = ["TODO", "FIXME", "HACK", "BUG", "XXX"];
    let mut parts: Vec<String> = Vec::new();

    for tag_name in &known_order {
        if let Some(&count) = by_tag.get(*tag_name) {
            if count > 0 {
                parts.push(format!("{}: {}", tag_name, count));
            }
        }
    }

    // Add any custom tags not in known_order
    let mut custom: Vec<(&String, &usize)> = by_tag
        .iter()
        .filter(|(k, v)| !known_order.contains(&k.as_str()) && **v > 0)
        .collect();
    custom.sort_by_key(|(k, _)| k.to_string());
    for (tag_name, count) in custom {
        parts.push(format!("{}: {}", tag_name, count));
    }

    parts.join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ScanMetadata, ScanResult, ScanStats, TodoItem, TodoTag};
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
    fn test_format_contains_file_paths() {
        // Disable colors for deterministic test output
        colored::control::set_override(false);

        let formatter = TextFormatter { show_summary: true };
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        assert!(output.contains("src/main.rs"), "Should contain src/main.rs");
        assert!(output.contains("src/lib.rs"), "Should contain src/lib.rs");
    }

    #[test]
    fn test_format_contains_line_numbers() {
        colored::control::set_override(false);

        let formatter = TextFormatter { show_summary: true };
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        assert!(output.contains("L12"), "Should contain L12");
        assert!(output.contains("L45"), "Should contain L45");
        assert!(output.contains("L3"), "Should contain L3");
    }

    #[test]
    fn test_format_contains_tags() {
        colored::control::set_override(false);

        let formatter = TextFormatter { show_summary: true };
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        assert!(output.contains("TODO"), "Should contain TODO tag");
        assert!(output.contains("FIXME"), "Should contain FIXME tag");
        assert!(output.contains("HACK"), "Should contain HACK tag");
    }

    #[test]
    fn test_format_contains_messages() {
        colored::control::set_override(false);

        let formatter = TextFormatter { show_summary: true };
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        assert!(output.contains("Add error handling"), "Should contain message");
        assert!(output.contains("This is broken"), "Should contain message");
        assert!(output.contains("Temporary workaround"), "Should contain message");
    }

    #[test]
    fn test_format_contains_metadata() {
        colored::control::set_override(false);

        let formatter = TextFormatter { show_summary: true };
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        assert!(output.contains("alice"), "Should contain author");
        assert!(output.contains("#123"), "Should contain issue");
        assert!(output.contains("p:high"), "Should contain priority");
    }

    #[test]
    fn test_format_summary() {
        colored::control::set_override(false);

        let formatter = TextFormatter { show_summary: true };
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        assert!(output.contains("Summary"), "Should contain Summary header");
        assert!(
            output.contains("3 TODOs in 2 files"),
            "Should contain total summary"
        );
        assert!(
            output.contains("scanned 15 files in 42ms"),
            "Should contain scan info"
        );
        assert!(output.contains("TODO: 1"), "Should contain TODO count");
        assert!(output.contains("FIXME: 1"), "Should contain FIXME count");
        assert!(output.contains("HACK: 1"), "Should contain HACK count");
    }

    #[test]
    fn test_format_no_summary() {
        colored::control::set_override(false);

        let formatter = TextFormatter {
            show_summary: false,
        };
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        assert!(!output.contains("Summary"), "Should not contain Summary when disabled");
    }

    #[test]
    fn test_format_metadata_all_fields() {
        let item = TodoItem {
            tag: TodoTag::Todo,
            message: "test".to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            author: Some("bob".to_string()),
            issue: Some("456".to_string()),
            priority: Some(Priority::Critical),
            context_line: String::new(),
            git_author: None,
            git_date: None,
        };

        let meta = format_metadata(&item);
        assert_eq!(meta, "(bob, #456, p:critical)");
    }

    #[test]
    fn test_format_metadata_no_fields() {
        let item = TodoItem {
            tag: TodoTag::Todo,
            message: "test".to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            author: None,
            issue: None,
            priority: None,
            context_line: String::new(),
            git_author: None,
            git_date: None,
        };

        let meta = format_metadata(&item);
        assert!(meta.is_empty(), "No metadata should produce empty string");
    }

    #[test]
    fn test_format_metadata_issue_with_hash() {
        let item = TodoItem {
            tag: TodoTag::Todo,
            message: "test".to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            author: None,
            issue: Some("#789".to_string()),
            priority: None,
            context_line: String::new(),
            git_author: None,
            git_date: None,
        };

        let meta = format_metadata(&item);
        assert_eq!(meta, "(#789)", "Should not double-prefix #");
    }

    #[test]
    fn test_empty_result() {
        colored::control::set_override(false);

        let formatter = TextFormatter { show_summary: true };
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

        assert!(
            output.contains("0 TODOs in 0 files"),
            "Should show zero counts"
        );
    }

    #[test]
    fn test_colorize_tag_variants() {
        // Just verify the function doesn't panic for all variants
        colorize_tag(&TodoTag::Todo);
        colorize_tag(&TodoTag::Fixme);
        colorize_tag(&TodoTag::Hack);
        colorize_tag(&TodoTag::Bug);
        colorize_tag(&TodoTag::Xxx);
        colorize_tag(&TodoTag::Custom("WARN".to_string()));
    }
}
