use crate::model::{Priority, TodoItem};

#[derive(Debug, Default)]
pub struct FilterCriteria {
    pub tags: Option<Vec<String>>,
    pub authors: Option<Vec<String>>,
    pub file_pattern: Option<String>,
    pub priority: Option<Priority>,
    pub has_issue: Option<bool>,
}

impl FilterCriteria {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_none()
            && self.authors.is_none()
            && self.file_pattern.is_none()
            && self.priority.is_none()
            && self.has_issue.is_none()
    }

    pub fn apply(&self, items: &[TodoItem]) -> Vec<TodoItem> {
        items
            .iter()
            .filter(|item| self.matches(item))
            .cloned()
            .collect()
    }

    fn matches(&self, item: &TodoItem) -> bool {
        // All filters are AND-combined
        if let Some(ref tags) = self.tags {
            let tag_str = item.tag.as_str().to_uppercase();
            if !tags.iter().any(|t| t.to_uppercase() == tag_str) {
                return false;
            }
        }

        if let Some(ref authors) = self.authors {
            match &item.author {
                Some(author) => {
                    if !authors
                        .iter()
                        .any(|a| a.to_lowercase() == author.to_lowercase())
                    {
                        return false;
                    }
                }
                None => return false,
            }
        }

        if let Some(ref pattern) = self.file_pattern {
            let path_str = item.file.display().to_string();
            if !glob_match(pattern, &path_str) {
                return false;
            }
        }

        if let Some(ref priority) = self.priority {
            match &item.priority {
                Some(p) if p == priority => {}
                _ => return false,
            }
        }

        if let Some(has_issue) = self.has_issue {
            if has_issue != item.issue.is_some() {
                return false;
            }
        }

        true
    }
}

/// Simple glob matcher supporting `*` as a wildcard.
/// Path separators are normalized to `/` before matching.
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.replace('\\', "/");
    let text = text.replace('\\', "/");

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            match text[pos..].find(part) {
                Some(found) => {
                    if i == 0 && found != 0 {
                        // Pattern starts with a literal segment, must match from start
                        return false;
                    }
                    pos += found + part.len();
                }
                None => return false,
            }
        }
        true
    } else {
        text.contains(&pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Priority, TodoItem, TodoTag};
    use std::path::PathBuf;

    fn make_item(tag: &str, message: &str) -> TodoItem {
        TodoItem {
            tag: TodoTag::from_str(tag),
            message: message.to_string(),
            file: PathBuf::from("src/main.rs"),
            line: 1,
            column: 1,
            author: None,
            issue: None,
            priority: None,
            context_line: String::new(),
            git_author: None,
            git_date: None,
        }
    }

    fn make_item_full(
        tag: &str,
        file: &str,
        author: Option<&str>,
        issue: Option<&str>,
        priority: Option<Priority>,
    ) -> TodoItem {
        TodoItem {
            tag: TodoTag::from_str(tag),
            message: "test message".to_string(),
            file: PathBuf::from(file),
            line: 1,
            column: 1,
            author: author.map(|s| s.to_string()),
            issue: issue.map(|s| s.to_string()),
            priority,
            context_line: String::new(),
            git_author: None,
            git_date: None,
        }
    }

    #[test]
    fn test_empty_filter_matches_all() {
        let filter = FilterCriteria::new();
        assert!(filter.is_empty());

        let items = vec![make_item("TODO", "do something"), make_item("FIXME", "fix this")];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_by_single_tag() {
        let filter = FilterCriteria {
            tags: Some(vec!["TODO".to_string()]),
            ..Default::default()
        };

        let items = vec![
            make_item("TODO", "do something"),
            make_item("FIXME", "fix this"),
            make_item("HACK", "workaround"),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tag.as_str(), "TODO");
    }

    #[test]
    fn test_filter_by_multiple_tags() {
        let filter = FilterCriteria {
            tags: Some(vec!["TODO".to_string(), "FIXME".to_string()]),
            ..Default::default()
        };

        let items = vec![
            make_item("TODO", "do something"),
            make_item("FIXME", "fix this"),
            make_item("HACK", "workaround"),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_tags_case_insensitive() {
        let filter = FilterCriteria {
            tags: Some(vec!["todo".to_string()]),
            ..Default::default()
        };

        let items = vec![make_item("TODO", "do something")];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_by_author() {
        let filter = FilterCriteria {
            authors: Some(vec!["alice".to_string()]),
            ..Default::default()
        };

        let items = vec![
            make_item_full("TODO", "src/main.rs", Some("alice"), None, None),
            make_item_full("TODO", "src/main.rs", Some("bob"), None, None),
            make_item_full("TODO", "src/main.rs", None, None, None),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].author.as_deref(), Some("alice"));
    }

    #[test]
    fn test_filter_author_case_insensitive() {
        let filter = FilterCriteria {
            authors: Some(vec!["Alice".to_string()]),
            ..Default::default()
        };

        let items = vec![make_item_full(
            "TODO",
            "src/main.rs",
            Some("alice"),
            None,
            None,
        )];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_by_file_pattern_wildcard() {
        let filter = FilterCriteria {
            file_pattern: Some("*.rs".to_string()),
            ..Default::default()
        };

        let items = vec![
            make_item_full("TODO", "src/main.rs", None, None, None),
            make_item_full("TODO", "src/lib.js", None, None, None),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].file, PathBuf::from("src/main.rs"));
    }

    #[test]
    fn test_filter_by_file_pattern_directory() {
        let filter = FilterCriteria {
            file_pattern: Some("src/*".to_string()),
            ..Default::default()
        };

        let items = vec![
            make_item_full("TODO", "src/main.rs", None, None, None),
            make_item_full("TODO", "tests/test.rs", None, None, None),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_by_priority() {
        let filter = FilterCriteria {
            priority: Some(Priority::High),
            ..Default::default()
        };

        let items = vec![
            make_item_full("TODO", "src/main.rs", None, None, Some(Priority::High)),
            make_item_full("TODO", "src/main.rs", None, None, Some(Priority::Low)),
            make_item_full("TODO", "src/main.rs", None, None, None),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].priority, Some(Priority::High));
    }

    #[test]
    fn test_filter_has_issue_true() {
        let filter = FilterCriteria {
            has_issue: Some(true),
            ..Default::default()
        };

        let items = vec![
            make_item_full("TODO", "src/main.rs", None, Some("#123"), None),
            make_item_full("TODO", "src/main.rs", None, None, None),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].issue.as_deref(), Some("#123"));
    }

    #[test]
    fn test_filter_has_issue_false() {
        let filter = FilterCriteria {
            has_issue: Some(false),
            ..Default::default()
        };

        let items = vec![
            make_item_full("TODO", "src/main.rs", None, Some("#123"), None),
            make_item_full("TODO", "src/main.rs", None, None, None),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
        assert!(result[0].issue.is_none());
    }

    #[test]
    fn test_combined_filters_and_logic() {
        let filter = FilterCriteria {
            tags: Some(vec!["TODO".to_string()]),
            authors: Some(vec!["alice".to_string()]),
            priority: Some(Priority::High),
            ..Default::default()
        };

        let items = vec![
            // Matches all: TODO, alice, high
            make_item_full("TODO", "src/main.rs", Some("alice"), None, Some(Priority::High)),
            // Wrong tag
            make_item_full("FIXME", "src/main.rs", Some("alice"), None, Some(Priority::High)),
            // Wrong author
            make_item_full("TODO", "src/main.rs", Some("bob"), None, Some(Priority::High)),
            // Wrong priority
            make_item_full("TODO", "src/main.rs", Some("alice"), None, Some(Priority::Low)),
        ];
        let result = filter.apply(&items);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].tag.as_str(), "TODO");
        assert_eq!(result[0].author.as_deref(), Some("alice"));
        assert_eq!(result[0].priority, Some(Priority::High));
    }

    #[test]
    fn test_glob_match_star_extension() {
        assert!(glob_match("*.rs", "src/main.rs"));
        assert!(!glob_match("*.rs", "src/main.js"));
    }

    #[test]
    fn test_glob_match_prefix() {
        assert!(glob_match("src/*", "src/main.rs"));
        assert!(!glob_match("src/*", "tests/main.rs"));
    }

    #[test]
    fn test_glob_match_no_wildcard() {
        assert!(glob_match("main", "src/main.rs"));
        assert!(!glob_match("main", "src/lib.rs"));
    }

    #[test]
    fn test_glob_match_backslash_normalization() {
        assert!(glob_match("src/*", "src\\main.rs"));
        assert!(glob_match("src\\*", "src/main.rs"));
    }

    #[test]
    fn test_glob_match_leading_literal() {
        // Pattern starts with a literal (no leading *), must match from position 0
        assert!(glob_match("src/*.rs", "src/main.rs"));
        assert!(!glob_match("src/*.rs", "lib/src/main.rs"));
    }
}
