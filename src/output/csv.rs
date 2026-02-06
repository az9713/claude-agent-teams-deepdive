use ::csv as csv_crate;

use crate::error::Result;
use crate::model::ScanResult;
use crate::output::OutputFormatter;

pub struct CsvFormatter;

impl OutputFormatter for CsvFormatter {
    fn format(&self, result: &ScanResult) -> Result<String> {
        let mut wtr = csv_crate::WriterBuilder::new().from_writer(Vec::new());

        // Write header row
        wtr.write_record(["file", "line", "column", "tag", "message", "author", "issue", "priority"])
            .map_err(|e| crate::error::TodoError::Config(e.to_string()))?;

        // Write one row per item
        for item in &result.items {
            let priority_str = item.priority.as_ref().map_or(String::new(), |p| {
                match p {
                    crate::model::Priority::Low => "low".to_string(),
                    crate::model::Priority::Medium => "medium".to_string(),
                    crate::model::Priority::High => "high".to_string(),
                    crate::model::Priority::Critical => "critical".to_string(),
                }
            });

            wtr.write_record(&[
                item.file.display().to_string(),
                item.line.to_string(),
                item.column.to_string(),
                item.tag.as_str().to_string(),
                item.message.clone(),
                item.author.clone().unwrap_or_default(),
                item.issue.clone().unwrap_or_default(),
                priority_str,
            ])
            .map_err(|e| crate::error::TodoError::Config(e.to_string()))?;
        }

        let bytes = wtr
            .into_inner()
            .map_err(|e| crate::error::TodoError::Config(e.to_string()))?;

        String::from_utf8(bytes)
            .map_err(|e| crate::error::TodoError::Config(e.to_string()))
    }
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
        ];

        let mut by_tag = HashMap::new();
        by_tag.insert("TODO".to_string(), 1);
        by_tag.insert("FIXME".to_string(), 1);

        ScanResult {
            items,
            stats: ScanStats {
                files_scanned: 10,
                files_with_todos: 1,
                total_todos: 2,
                by_tag,
            },
            metadata: ScanMetadata {
                scan_duration_ms: 25,
                root_path: PathBuf::from("."),
                timestamp: "2026-02-05T00:00:00Z".to_string(),
            },
        }
    }

    #[test]
    fn test_csv_has_header() {
        let formatter = CsvFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let first_line = output.lines().next().unwrap();
        assert_eq!(first_line, "file,line,column,tag,message,author,issue,priority");
    }

    #[test]
    fn test_csv_row_count() {
        let formatter = CsvFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        // 1 header + 2 data rows
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_csv_data_row_content() {
        let formatter = CsvFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let lines: Vec<&str> = output.lines().collect();

        assert!(lines[1].contains("src/main.rs"));
        assert!(lines[1].contains("12"));
        assert!(lines[1].contains("TODO"));
        assert!(lines[1].contains("Add error handling"));
        assert!(lines[1].contains("alice"));
        assert!(lines[1].contains("123"));
    }

    #[test]
    fn test_csv_priority_field() {
        let formatter = CsvFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let lines: Vec<&str> = output.lines().collect();

        // Second data row has High priority
        assert!(lines[2].contains("high"));
    }

    #[test]
    fn test_csv_empty_result() {
        let formatter = CsvFormatter;
        let result = ScanResult {
            items: vec![],
            stats: ScanStats {
                files_scanned: 0,
                files_with_todos: 0,
                total_todos: 0,
                by_tag: HashMap::new(),
            },
            metadata: ScanMetadata {
                scan_duration_ms: 0,
                root_path: PathBuf::from("."),
                timestamp: "2026-02-05T00:00:00Z".to_string(),
            },
        };
        let output = formatter.format(&result).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        // Only header row
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "file,line,column,tag,message,author,issue,priority");
    }

    #[test]
    fn test_csv_message_with_comma() {
        let formatter = CsvFormatter;
        let items = vec![TodoItem {
            tag: TodoTag::Todo,
            message: "Fix this, please".to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            column: 1,
            author: None,
            issue: None,
            priority: None,
            context_line: String::new(),
            git_author: None,
            git_date: None,
        }];

        let mut by_tag = HashMap::new();
        by_tag.insert("TODO".to_string(), 1);

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
        // The csv crate should quote the field containing a comma
        assert!(
            output.contains("\"Fix this, please\""),
            "Commas in fields should be properly quoted"
        );
    }
}
