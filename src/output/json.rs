use crate::error::Result;
use crate::model::ScanResult;
use crate::output::OutputFormatter;

pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format(&self, result: &ScanResult) -> Result<String> {
        serde_json::to_string_pretty(result)
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
    fn test_json_is_valid() {
        let formatter = JsonFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object(), "Output should be a JSON object");
    }

    #[test]
    fn test_json_contains_items() {
        let formatter = JsonFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let items = parsed["items"].as_array().unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_json_item_fields() {
        let formatter = JsonFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let first_item = &parsed["items"][0];
        assert_eq!(first_item["message"], "Add error handling");
        assert_eq!(first_item["line"], 12);
        assert_eq!(first_item["author"], "alice");
    }

    #[test]
    fn test_json_contains_stats() {
        let formatter = JsonFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let stats = &parsed["stats"];
        assert_eq!(stats["total_todos"], 2);
        assert_eq!(stats["files_scanned"], 10);
    }

    #[test]
    fn test_json_empty_result() {
        let formatter = JsonFormatter;
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

        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let items = parsed["items"].as_array().unwrap();
        assert!(items.is_empty());
    }
}
