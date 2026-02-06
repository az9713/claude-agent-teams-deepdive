use std::fmt::Write;

use crate::error::Result;
use crate::model::ScanResult;
use crate::output::OutputFormatter;

pub struct GithubActionsFormatter;

impl OutputFormatter for GithubActionsFormatter {
    fn format(&self, result: &ScanResult) -> Result<String> {
        let mut out = String::new();
        for item in &result.items {
            let level = match item.tag.as_str() {
                "FIXME" | "BUG" => "error",
                "HACK" | "XXX" => "warning",
                _ => "warning",
            };
            let file = item.file.display().to_string().replace('\\', "/");
            writeln!(
                out,
                "::{level} file={file},line={line},col={col}::{tag}: {msg}",
                level = level,
                file = file,
                line = item.line,
                col = item.column,
                tag = item.tag,
                msg = item.message
            )
            .unwrap();
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ScanMetadata, ScanStats, TodoItem, TodoTag};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_item(tag: TodoTag, message: &str, file: &str, line: usize, col: usize) -> TodoItem {
        TodoItem {
            tag,
            message: message.to_string(),
            file: PathBuf::from(file),
            line,
            column: col,
            author: None,
            issue: None,
            priority: None,
            context_line: String::new(),
            git_author: None,
            git_date: None,
        }
    }

    fn make_result(items: Vec<TodoItem>) -> ScanResult {
        let total = items.len();
        ScanResult {
            items,
            stats: ScanStats {
                files_scanned: 1,
                files_with_todos: 1,
                total_todos: total,
                by_tag: HashMap::new(),
            },
            metadata: ScanMetadata {
                scan_duration_ms: 0,
                root_path: PathBuf::from("."),
                timestamp: "2026-02-05T00:00:00Z".to_string(),
            },
        }
    }

    #[test]
    fn test_todo_emits_warning() {
        let result = make_result(vec![make_item(
            TodoTag::Todo,
            "fix this",
            "src/main.rs",
            10,
            5,
        )]);
        let formatter = GithubActionsFormatter;
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("::warning file=src/main.rs,line=10,col=5::TODO: fix this"));
    }

    #[test]
    fn test_fixme_emits_error() {
        let result = make_result(vec![make_item(
            TodoTag::Fixme,
            "broken",
            "src/lib.rs",
            20,
            3,
        )]);
        let formatter = GithubActionsFormatter;
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("::error file=src/lib.rs,line=20,col=3::FIXME: broken"));
    }

    #[test]
    fn test_bug_emits_error() {
        let result = make_result(vec![make_item(
            TodoTag::Bug,
            "crash here",
            "src/app.rs",
            5,
            1,
        )]);
        let formatter = GithubActionsFormatter;
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("::error file=src/app.rs,line=5,col=1::BUG: crash here"));
    }

    #[test]
    fn test_hack_emits_warning() {
        let result = make_result(vec![make_item(
            TodoTag::Hack,
            "workaround",
            "src/util.rs",
            15,
            2,
        )]);
        let formatter = GithubActionsFormatter;
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("::warning file=src/util.rs,line=15,col=2::HACK: workaround"));
    }

    #[test]
    fn test_multiple_items() {
        let result = make_result(vec![
            make_item(TodoTag::Todo, "first", "src/a.rs", 1, 1),
            make_item(TodoTag::Fixme, "second", "src/b.rs", 2, 1),
        ]);
        let formatter = GithubActionsFormatter;
        let output = formatter.format(&result).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_empty_result() {
        let result = make_result(vec![]);
        let formatter = GithubActionsFormatter;
        let output = formatter.format(&result).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_backslash_normalization() {
        let result = make_result(vec![make_item(
            TodoTag::Todo,
            "test",
            "src\\nested\\file.rs",
            1,
            1,
        )]);
        let formatter = GithubActionsFormatter;
        let output = formatter.format(&result).unwrap();
        assert!(output.contains("file=src/nested/file.rs"));
        assert!(!output.contains('\\'));
    }
}
