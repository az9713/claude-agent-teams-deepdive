use serde::{Deserialize, Serialize};

use crate::model::ScanResult;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyConfig {
    /// Maximum number of TODOs allowed
    pub max_todos: Option<usize>,
    /// Tags that require an issue reference (e.g., ["FIXME", "BUG"])
    pub require_issue: Option<Vec<String>>,
    /// Tags that are completely denied (e.g., ["NOCOMMIT"])
    pub deny_tags: Option<Vec<String>>,
    /// Maximum age in days for TODOs (requires git blame data)
    pub max_age_days: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Error,
    Warning,
}

impl std::fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationSeverity::Error => write!(f, "error"),
            ViolationSeverity::Warning => write!(f, "warning"),
        }
    }
}

pub fn check_policies(result: &ScanResult, config: &PolicyConfig) -> Vec<PolicyViolation> {
    let mut violations = Vec::new();

    // Check max_todos
    if let Some(max) = config.max_todos {
        if result.stats.total_todos > max {
            violations.push(PolicyViolation {
                rule: "max_todos".to_string(),
                message: format!(
                    "Found {} TODOs, maximum allowed is {}",
                    result.stats.total_todos, max
                ),
                file: None,
                line: None,
                severity: ViolationSeverity::Error,
            });
        }
    }

    // Check require_issue
    if let Some(ref require_tags) = config.require_issue {
        for item in &result.items {
            let tag_upper = item.tag.as_str().to_uppercase();
            if require_tags.iter().any(|t| t.to_uppercase() == tag_upper) && item.issue.is_none() {
                violations.push(PolicyViolation {
                    rule: "require_issue".to_string(),
                    message: format!(
                        "{} at {}:{} requires an issue reference",
                        item.tag,
                        item.file.display(),
                        item.line
                    ),
                    file: Some(item.file.display().to_string()),
                    line: Some(item.line),
                    severity: ViolationSeverity::Error,
                });
            }
        }
    }

    // Check deny_tags
    if let Some(ref deny) = config.deny_tags {
        for item in &result.items {
            let tag_upper = item.tag.as_str().to_uppercase();
            if deny.iter().any(|t| t.to_uppercase() == tag_upper) {
                violations.push(PolicyViolation {
                    rule: "deny_tags".to_string(),
                    message: format!(
                        "Denied tag {} found at {}:{}",
                        item.tag,
                        item.file.display(),
                        item.line
                    ),
                    file: Some(item.file.display().to_string()),
                    line: Some(item.line),
                    severity: ViolationSeverity::Error,
                });
            }
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ScanMetadata, ScanStats, TodoItem, TodoTag};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn make_item(tag: &str, file: &str, line: usize, issue: Option<&str>) -> TodoItem {
        TodoItem {
            tag: TodoTag::from_str(tag),
            message: "test message".to_string(),
            file: PathBuf::from(file),
            line,
            column: 1,
            author: None,
            issue: issue.map(|s| s.to_string()),
            priority: None,
            context_line: String::new(),
            git_author: None,
            git_date: None,
        }
    }

    fn make_result(items: Vec<TodoItem>) -> ScanResult {
        let total = items.len();
        let mut by_tag = HashMap::new();
        for item in &items {
            *by_tag.entry(item.tag.as_str().to_string()).or_insert(0) += 1;
        }
        ScanResult {
            items,
            stats: ScanStats {
                files_scanned: 5,
                files_with_todos: 2,
                total_todos: total,
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
    fn test_max_todos_passes_when_under_limit() {
        let result = make_result(vec![make_item("TODO", "src/main.rs", 1, None)]);
        let config = PolicyConfig {
            max_todos: Some(5),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_max_todos_fails_when_over_limit() {
        let result = make_result(vec![
            make_item("TODO", "src/main.rs", 1, None),
            make_item("TODO", "src/main.rs", 2, None),
            make_item("TODO", "src/main.rs", 3, None),
        ]);
        let config = PolicyConfig {
            max_todos: Some(2),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "max_todos");
        assert!(violations[0].message.contains("3"));
        assert!(violations[0].message.contains("2"));
    }

    #[test]
    fn test_max_todos_passes_at_exact_limit() {
        let result = make_result(vec![
            make_item("TODO", "src/main.rs", 1, None),
            make_item("TODO", "src/main.rs", 2, None),
        ]);
        let config = PolicyConfig {
            max_todos: Some(2),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_require_issue_passes_when_issue_present() {
        let result = make_result(vec![make_item("FIXME", "src/main.rs", 10, Some("#123"))]);
        let config = PolicyConfig {
            require_issue: Some(vec!["FIXME".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_require_issue_fails_when_issue_missing() {
        let result = make_result(vec![make_item("FIXME", "src/main.rs", 10, None)]);
        let config = PolicyConfig {
            require_issue: Some(vec!["FIXME".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "require_issue");
        assert!(violations[0].message.contains("FIXME"));
        assert!(violations[0].message.contains("src/main.rs:10"));
    }

    #[test]
    fn test_require_issue_case_insensitive() {
        let result = make_result(vec![make_item("fixme", "src/main.rs", 10, None)]);
        let config = PolicyConfig {
            require_issue: Some(vec!["FIXME".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        // TodoTag::from_str normalizes "fixme" to Fixme which as_str() returns "FIXME"
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_require_issue_ignores_non_matching_tags() {
        let result = make_result(vec![make_item("TODO", "src/main.rs", 10, None)]);
        let config = PolicyConfig {
            require_issue: Some(vec!["FIXME".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_deny_tags_catches_denied_tag() {
        let result = make_result(vec![make_item("HACK", "src/main.rs", 5, None)]);
        let config = PolicyConfig {
            deny_tags: Some(vec!["HACK".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "deny_tags");
        assert!(violations[0].message.contains("HACK"));
    }

    #[test]
    fn test_deny_tags_passes_non_denied_tag() {
        let result = make_result(vec![make_item("TODO", "src/main.rs", 5, None)]);
        let config = PolicyConfig {
            deny_tags: Some(vec!["HACK".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_deny_tags_case_insensitive() {
        let result = make_result(vec![make_item("hack", "src/main.rs", 5, None)]);
        let config = PolicyConfig {
            deny_tags: Some(vec!["HACK".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn test_multiple_deny_tags() {
        let result = make_result(vec![
            make_item("HACK", "src/main.rs", 5, None),
            make_item("XXX", "src/lib.rs", 10, None),
            make_item("TODO", "src/lib.rs", 15, None),
        ]);
        let config = PolicyConfig {
            deny_tags: Some(vec!["HACK".to_string(), "XXX".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn test_empty_config_no_violations() {
        let result = make_result(vec![
            make_item("TODO", "src/main.rs", 1, None),
            make_item("HACK", "src/main.rs", 2, None),
        ]);
        let config = PolicyConfig::default();
        let violations = check_policies(&result, &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_combined_policies() {
        let result = make_result(vec![
            make_item("TODO", "src/main.rs", 1, None),
            make_item("FIXME", "src/main.rs", 2, None),
            make_item("HACK", "src/lib.rs", 3, None),
        ]);
        let config = PolicyConfig {
            max_todos: Some(5),
            require_issue: Some(vec!["FIXME".to_string()]),
            deny_tags: Some(vec!["HACK".to_string()]),
            ..Default::default()
        };
        let violations = check_policies(&result, &config);
        // FIXME without issue + HACK denied = 2 violations
        assert_eq!(violations.len(), 2);
        let rules: Vec<&str> = violations.iter().map(|v| v.rule.as_str()).collect();
        assert!(rules.contains(&"require_issue"));
        assert!(rules.contains(&"deny_tags"));
    }
}
