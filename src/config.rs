use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub scan: Option<ScanConfig>,
    pub output: Option<OutputConfig>,
    pub filter: Option<FilterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanConfig {
    pub max_file_size: Option<u64>,
    pub respect_gitignore: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    pub format: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterConfig {
    pub exclude_patterns: Option<Vec<String>>,
}

impl Config {
    /// Load configuration from an explicit path, or by searching up from CWD,
    /// or from the user's home config directory. Returns default if nothing found.
    pub fn load(explicit_path: Option<&str>) -> Config {
        if let Some(path) = explicit_path {
            return Self::load_from_file(Path::new(path)).unwrap_or_default();
        }

        // Walk up from CWD looking for .todo-tracker.toml
        if let Ok(cwd) = std::env::current_dir() {
            let mut dir = Some(cwd.as_path().to_path_buf());
            while let Some(d) = dir {
                let candidate = d.join(".todo-tracker.toml");
                if candidate.is_file() {
                    return Self::load_from_file(&candidate).unwrap_or_default();
                }
                dir = d.parent().map(|p| p.to_path_buf());
            }
        }

        // Try user-level config
        if let Some(config_dir) = Self::user_config_dir() {
            let candidate = config_dir.join("todo-tracker").join("config.toml");
            if candidate.is_file() {
                return Self::load_from_file(&candidate).unwrap_or_default();
            }
        }

        Config::default()
    }

    /// Returns a commented TOML template suitable for writing to a new config file.
    pub fn default_template() -> String {
        r#"# todo-tracker configuration
# See: https://github.com/todo-tracker/todo-tracker

# [scan]
# max_file_size = 1048576  # 1MB
# respect_gitignore = true
# tags = ["TODO", "FIXME", "HACK", "BUG", "XXX"]

# [output]
# format = "text"  # text, json, csv, markdown, count
# color = "auto"   # auto, always, never

# [filter]
# exclude_patterns = []
"#
        .to_string()
    }

    /// Returns the configured max file size, or the default of 1MB.
    pub fn get_max_file_size(&self) -> u64 {
        self.scan
            .as_ref()
            .and_then(|s| s.max_file_size)
            .unwrap_or(1_048_576)
    }

    /// Returns the configured output format, or "text" as the default.
    pub fn get_format(&self) -> String {
        self.output
            .as_ref()
            .and_then(|o| o.format.clone())
            .unwrap_or_else(|| "text".to_string())
    }

    fn load_from_file(path: &Path) -> Result<Config, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file {}: {}", path.display(), e))?;
        toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse config file {}: {}", path.display(), e))
    }

    fn user_config_dir() -> Option<PathBuf> {
        // Try XDG_CONFIG_HOME first, then platform-specific fallbacks
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg));
        }

        // On Windows, try USERPROFILE; on Unix, try HOME
        if cfg!(windows) {
            std::env::var("USERPROFILE")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
        } else {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.scan.is_none());
        assert!(config.output.is_none());
        assert!(config.filter.is_none());
    }

    #[test]
    fn test_default_accessors() {
        let config = Config::default();
        assert_eq!(config.get_max_file_size(), 1_048_576);
        assert_eq!(config.get_format(), "text");
    }

    #[test]
    fn test_load_explicit_path() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("test-config.toml");
        fs::write(
            &config_path,
            r#"
[scan]
max_file_size = 2097152
respect_gitignore = false
tags = ["TODO", "FIXME"]

[output]
format = "json"
color = "never"
"#,
        )
        .unwrap();

        let config = Config::load(Some(config_path.to_str().unwrap()));
        assert_eq!(config.get_max_file_size(), 2_097_152);
        assert_eq!(config.get_format(), "json");

        let scan = config.scan.unwrap();
        assert_eq!(scan.respect_gitignore, Some(false));
        assert_eq!(scan.tags, Some(vec!["TODO".to_string(), "FIXME".to_string()]));

        let output = config.output.unwrap();
        assert_eq!(output.color, Some("never".to_string()));
    }

    #[test]
    fn test_load_missing_explicit_path_returns_default() {
        let config = Config::load(Some("/nonexistent/path/config.toml"));
        assert!(config.scan.is_none());
        assert_eq!(config.get_format(), "text");
    }

    #[test]
    fn test_load_invalid_toml_returns_default() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("bad-config.toml");
        fs::write(&config_path, "this is not valid toml [[[").unwrap();

        let config = Config::load(Some(config_path.to_str().unwrap()));
        assert!(config.scan.is_none());
    }

    #[test]
    fn test_load_partial_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("partial.toml");
        fs::write(
            &config_path,
            r#"
[output]
format = "csv"
"#,
        )
        .unwrap();

        let config = Config::load(Some(config_path.to_str().unwrap()));
        assert!(config.scan.is_none());
        assert_eq!(config.get_format(), "csv");
        assert_eq!(config.get_max_file_size(), 1_048_576);
    }

    #[test]
    fn test_load_with_filter_config() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("filter.toml");
        fs::write(
            &config_path,
            r#"
[filter]
exclude_patterns = ["*.min.js", "vendor/**"]
"#,
        )
        .unwrap();

        let config = Config::load(Some(config_path.to_str().unwrap()));
        let filter = config.filter.unwrap();
        let patterns = filter.exclude_patterns.unwrap();
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0], "*.min.js");
        assert_eq!(patterns[1], "vendor/**");
    }

    #[test]
    fn test_default_template_is_valid_comment_only() {
        let template = Config::default_template();
        // The template is all comments, so parsing it should yield an empty/default config
        let config: Config = toml::from_str(&template).unwrap();
        assert!(config.scan.is_none());
        assert!(config.output.is_none());
        assert!(config.filter.is_none());
    }

    #[test]
    fn test_load_none_returns_config() {
        // With no explicit path and likely no .todo-tracker.toml in ancestors,
        // this should return a config (possibly default)
        let config = Config::load(None);
        // Just verify it doesn't panic and returns something valid
        let _ = config.get_max_file_size();
        let _ = config.get_format();
    }
}
