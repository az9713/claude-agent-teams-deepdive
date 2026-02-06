use std::path::Path;
use std::process::Command;

/// Run a git command in the given repo directory and return stdout as a String.
pub fn git_command(args: &[&str], repo_root: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("Failed to execute git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {}", args.join(" "), stderr.trim()));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in git output: {}", e))
}

/// Check if the given path is inside a git repository.
pub fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the root directory of the git repository containing `path`.
pub fn repo_root(path: &Path) -> Result<std::path::PathBuf, String> {
    let output = git_command(&["rev-parse", "--show-toplevel"], path)?;
    Ok(std::path::PathBuf::from(output.trim()))
}
