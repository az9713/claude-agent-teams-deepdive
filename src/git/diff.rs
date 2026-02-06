use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::git::utils::git_command;
use crate::model::TodoItem;
use crate::scanner::FileScanner;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffResult {
    pub added: Vec<TodoItem>,
    pub removed: Vec<TodoItem>,
    pub base_ref: String,
    pub head_ref: String,
}

/// Get list of files changed between two refs.
pub fn changed_files(base: &str, head: &str, repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let output = git_command(
        &["diff", "--name-only", &format!("{}...{}", base, head)],
        repo_root,
    )?;
    Ok(output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| repo_root.join(l.trim()))
        .collect())
}

/// Get list of staged files.
pub fn staged_files(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let output = git_command(&["diff", "--name-only", "--staged"], repo_root)?;
    Ok(output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| repo_root.join(l.trim()))
        .collect())
}

/// Get file content at a specific git ref.
fn file_at_ref(file_path: &Path, git_ref: &str, repo_root: &Path) -> Result<String, String> {
    let relative = file_path.strip_prefix(repo_root).unwrap_or(file_path);
    // Normalize path separators for git (Windows uses backslashes)
    let path_str = relative.to_str().unwrap_or("").replace('\\', "/");
    git_command(
        &["show", &format!("{}:{}", git_ref, path_str)],
        repo_root,
    )
}

/// Build an identity key for a TodoItem based on (file, tag, message).
fn item_key(item: &TodoItem) -> (String, String, String) {
    (
        item.file.display().to_string(),
        item.tag.as_str().to_string(),
        item.message.clone(),
    )
}

/// Scan TODOs in a set of files at a specific git ref.
fn scan_at_ref(
    scanner: &dyn FileScanner,
    files: &[PathBuf],
    git_ref: &str,
    repo_root: &Path,
) -> Vec<TodoItem> {
    let mut items = Vec::new();
    for file in files {
        if let Ok(content) = file_at_ref(file, git_ref, repo_root) {
            let ext = file
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if let Ok(temp) = tempfile::Builder::new()
                .suffix(&format!(".{}", ext))
                .tempfile()
            {
                let temp_path = temp.path().to_path_buf();
                if std::fs::write(&temp_path, &content).is_ok() {
                    if let Ok(mut file_items) = scanner.scan_file(&temp_path) {
                        // Fix the file paths to point to the original file
                        for item in &mut file_items {
                            item.file = file.clone();
                        }
                        items.extend(file_items);
                    }
                }
            }
        }
    }
    items
}

/// Compare TODOs between two git refs.
pub fn diff_todos(
    scanner: &dyn FileScanner,
    base_ref: &str,
    head_ref: &str,
    repo_root: &Path,
) -> Result<DiffResult, String> {
    let files = changed_files(base_ref, head_ref, repo_root)?;

    let base_todos = scan_at_ref(scanner, &files, base_ref, repo_root);
    let head_todos = scan_at_ref(scanner, &files, head_ref, repo_root);

    let base_keys: HashMap<(String, String, String), &TodoItem> =
        base_todos.iter().map(|item| (item_key(item), item)).collect();
    let head_keys: HashMap<(String, String, String), &TodoItem> =
        head_todos.iter().map(|item| (item_key(item), item)).collect();

    let added: Vec<TodoItem> = head_todos
        .iter()
        .filter(|item| !base_keys.contains_key(&item_key(item)))
        .cloned()
        .collect();

    let removed: Vec<TodoItem> = base_todos
        .iter()
        .filter(|item| !head_keys.contains_key(&item_key(item)))
        .cloned()
        .collect();

    Ok(DiffResult {
        added,
        removed,
        base_ref: base_ref.to_string(),
        head_ref: head_ref.to_string(),
    })
}

/// Diff against staged changes (scan working tree vs HEAD for staged files).
pub fn diff_staged(
    scanner: &dyn FileScanner,
    repo_root: &Path,
) -> Result<DiffResult, String> {
    let files = staged_files(repo_root)?;

    let head_todos = scan_at_ref(scanner, &files, "HEAD", repo_root);

    // Scan current working tree versions
    let mut working_todos = Vec::new();
    for file in &files {
        if file.exists() {
            if let Ok(items) = scanner.scan_file(file) {
                working_todos.extend(items);
            }
        }
    }

    let head_keys: HashMap<(String, String, String), &TodoItem> =
        head_todos.iter().map(|item| (item_key(item), item)).collect();
    let working_keys: HashMap<(String, String, String), &TodoItem> = working_todos
        .iter()
        .map(|item| (item_key(item), item))
        .collect();

    let added: Vec<TodoItem> = working_todos
        .iter()
        .filter(|item| !head_keys.contains_key(&item_key(item)))
        .cloned()
        .collect();

    let removed: Vec<TodoItem> = head_todos
        .iter()
        .filter(|item| !working_keys.contains_key(&item_key(item)))
        .cloned()
        .collect();

    Ok(DiffResult {
        added,
        removed,
        base_ref: "HEAD".to_string(),
        head_ref: "working-tree".to_string(),
    })
}
