use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn from_str_tag(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" | "p:low" | "p3" => Some(Priority::Low),
            "medium" | "med" | "p:medium" | "p:med" | "p2" => Some(Priority::Medium),
            "high" | "p:high" | "p1" => Some(Priority::High),
            "critical" | "crit" | "p:critical" | "p:crit" | "p0" => Some(Priority::Critical),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoTag {
    Todo,
    Fixme,
    Hack,
    Bug,
    Xxx,
    Custom(String),
}

impl TodoTag {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TODO" => TodoTag::Todo,
            "FIXME" => TodoTag::Fixme,
            "HACK" => TodoTag::Hack,
            "BUG" => TodoTag::Bug,
            "XXX" => TodoTag::Xxx,
            other => TodoTag::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TodoTag::Todo => "TODO",
            TodoTag::Fixme => "FIXME",
            TodoTag::Hack => "HACK",
            TodoTag::Bug => "BUG",
            TodoTag::Xxx => "XXX",
            TodoTag::Custom(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for TodoTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub tag: TodoTag,
    pub message: String,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub author: Option<String>,
    pub issue: Option<String>,
    pub priority: Option<Priority>,
    pub context_line: String,
    // Git enrichment fields (Phase 3)
    pub git_author: Option<String>,
    pub git_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub files_scanned: usize,
    pub files_with_todos: usize,
    pub total_todos: usize,
    pub by_tag: std::collections::HashMap<String, usize>,
}

impl ScanStats {
    pub fn new() -> Self {
        Self {
            files_scanned: 0,
            files_with_todos: 0,
            total_todos: 0,
            by_tag: std::collections::HashMap::new(),
        }
    }

    pub fn add_item(&mut self, item: &TodoItem) {
        self.total_todos += 1;
        *self.by_tag.entry(item.tag.as_str().to_string()).or_insert(0) += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    pub scan_duration_ms: u64,
    pub root_path: PathBuf,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub items: Vec<TodoItem>,
    pub stats: ScanStats,
    pub metadata: ScanMetadata,
}
