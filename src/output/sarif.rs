use serde_json::{json, Value};

use crate::error::Result;
use crate::model::ScanResult;
use crate::output::OutputFormatter;

pub struct SarifFormatter;

impl OutputFormatter for SarifFormatter {
    fn format(&self, result: &ScanResult) -> Result<String> {
        let results: Vec<Value> = result
            .items
            .iter()
            .map(|item| {
                json!({
                    "ruleId": format!("todo-tracker/{}", item.tag.as_str().to_lowercase()),
                    "level": match item.tag.as_str() {
                        "FIXME" | "BUG" => "error",
                        "HACK" | "XXX" => "warning",
                        _ => "note"
                    },
                    "message": {
                        "text": format!("{}: {}", item.tag, item.message)
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": item.file.display().to_string().replace('\\', "/")
                            },
                            "region": {
                                "startLine": item.line,
                                "startColumn": item.column
                            }
                        }
                    }]
                })
            })
            .collect();

        let sarif = json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "todo-tracker",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": [
                            {"id": "todo-tracker/todo", "shortDescription": {"text": "TODO comment found"}},
                            {"id": "todo-tracker/fixme", "shortDescription": {"text": "FIXME comment found"}},
                            {"id": "todo-tracker/hack", "shortDescription": {"text": "HACK comment found"}},
                            {"id": "todo-tracker/bug", "shortDescription": {"text": "BUG comment found"}},
                            {"id": "todo-tracker/xxx", "shortDescription": {"text": "XXX comment found"}}
                        ]
                    }
                },
                "results": results
            }]
        });

        serde_json::to_string_pretty(&sarif)
            .map_err(|e| crate::error::TodoError::Config(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ScanMetadata, ScanStats, TodoItem, TodoTag};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn sample_result() -> ScanResult {
        let items = vec![
            TodoItem {
                tag: TodoTag::Todo,
                message: "Add tests".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 10,
                column: 5,
                author: None,
                issue: None,
                priority: None,
                context_line: "// TODO: Add tests".to_string(),
                git_author: None,
                git_date: None,
            },
            TodoItem {
                tag: TodoTag::Fixme,
                message: "Handle error".to_string(),
                file: PathBuf::from("src/lib.rs"),
                line: 20,
                column: 3,
                author: None,
                issue: None,
                priority: None,
                context_line: "// FIXME: Handle error".to_string(),
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
                files_scanned: 5,
                files_with_todos: 2,
                total_todos: 2,
                by_tag,
            },
            metadata: ScanMetadata {
                scan_duration_ms: 10,
                root_path: PathBuf::from("."),
                timestamp: "2026-02-05T00:00:00Z".to_string(),
            },
        }
    }

    #[test]
    fn test_sarif_is_valid_json() {
        let formatter = SarifFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_sarif_version() {
        let formatter = SarifFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], "2.1.0");
    }

    #[test]
    fn test_sarif_has_runs() {
        let formatter = SarifFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let runs = parsed["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn test_sarif_tool_name() {
        let formatter = SarifFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            parsed["runs"][0]["tool"]["driver"]["name"],
            "todo-tracker"
        );
    }

    #[test]
    fn test_sarif_results_count() {
        let formatter = SarifFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_sarif_rule_id_format() {
        let formatter = SarifFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["ruleId"], "todo-tracker/todo");
        assert_eq!(results[1]["ruleId"], "todo-tracker/fixme");
    }

    #[test]
    fn test_sarif_level_mapping() {
        let formatter = SarifFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "note"); // TODO -> note
        assert_eq!(results[1]["level"], "error"); // FIXME -> error
    }

    #[test]
    fn test_sarif_location_info() {
        let formatter = SarifFormatter;
        let result = sample_result();
        let output = formatter.format(&result).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let loc = &parsed["runs"][0]["results"][0]["locations"][0]["physicalLocation"];
        assert_eq!(loc["artifactLocation"]["uri"], "src/main.rs");
        assert_eq!(loc["region"]["startLine"], 10);
        assert_eq!(loc["region"]["startColumn"], 5);
    }

    #[test]
    fn test_sarif_empty_result() {
        let formatter = SarifFormatter;
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
        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert!(results.is_empty());
    }
}
