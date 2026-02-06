use std::collections::HashMap;
use std::path::Path;

use crate::git::utils::git_command;
use crate::model::TodoItem;

/// Blame info for a specific line
#[derive(Debug, Clone)]
pub struct BlameInfo {
    pub author: String,
    pub date: String,
    pub commit: String,
}

/// Run git blame on a file and parse the porcelain output.
/// Returns a map of line_number -> BlameInfo
pub fn blame_file(file_path: &Path, repo_root: &Path) -> Result<HashMap<usize, BlameInfo>, String> {
    let relative = file_path
        .strip_prefix(repo_root)
        .unwrap_or(file_path);

    let rel_str = relative.to_str().unwrap_or("");

    let output = git_command(&["blame", "--porcelain", rel_str], repo_root)?;

    parse_blame_porcelain(&output)
}

/// Parse git blame --porcelain output into line -> BlameInfo map
fn parse_blame_porcelain(output: &str) -> Result<HashMap<usize, BlameInfo>, String> {
    let mut result = HashMap::new();
    let mut current_author = String::new();
    let mut current_date = String::new();
    let mut current_commit = String::new();
    let mut current_line: usize = 0;

    for line in output.lines() {
        if line.len() >= 40 && line.chars().take(40).all(|c| c.is_ascii_hexdigit()) {
            // Commit line: <hash> <orig_line> <final_line> [<num_lines>]
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                current_commit = parts[0].to_string();
                current_line = parts[2].parse().unwrap_or(0);
            }
        } else if let Some(author) = line.strip_prefix("author ") {
            current_author = author.to_string();
        } else if let Some(date) = line.strip_prefix("author-time ") {
            if let Ok(ts) = date.parse::<i64>() {
                current_date = format_timestamp(ts);
            }
        } else if line.starts_with('\t') {
            // Content line -- save blame info for this line number
            if current_line > 0 {
                result.insert(
                    current_line,
                    BlameInfo {
                        author: current_author.clone(),
                        date: current_date.clone(),
                        commit: current_commit.clone(),
                    },
                );
            }
        }
    }

    Ok(result)
}

/// Format a Unix timestamp into YYYY-MM-DD without external dependencies.
fn format_timestamp(ts: i64) -> String {
    let days = ts / 86400;
    let mut y = 1970i64;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let months = [
        31,
        if is_leap_year(y) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 1;
    for &days_in_month in &months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        m += 1;
    }
    let d = remaining_days + 1;

    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Enrich TodoItems with git blame information.
/// Groups items by file to avoid blaming the same file multiple times.
pub fn enrich_with_blame(items: &mut [TodoItem], repo_root: &Path) {
    let mut files: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, item) in items.iter().enumerate() {
        files
            .entry(item.file.display().to_string())
            .or_default()
            .push(idx);
    }

    for (file_path, indices) in &files {
        let path = Path::new(file_path);
        if let Ok(blame_info) = blame_file(path, repo_root) {
            for &idx in indices {
                if let Some(info) = blame_info.get(&items[idx].line) {
                    items[idx].git_author = Some(info.author.clone());
                    items[idx].git_date = Some(info.date.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_timestamp_epoch() {
        // Unix epoch: 1970-01-01
        assert_eq!(format_timestamp(0), "1970-01-01");
    }

    #[test]
    fn test_format_timestamp_known_date() {
        // 2023-10-15 is 1697328000 (midnight UTC)
        assert_eq!(format_timestamp(1697328000), "2023-10-15");
    }

    #[test]
    fn test_format_timestamp_leap_year() {
        // 2024-02-29 is a leap day; 2024-03-01 00:00:00 UTC = 1709251200
        assert_eq!(format_timestamp(1709251200), "2024-03-01");
        // 2024-02-29 00:00:00 UTC = 1709164800
        assert_eq!(format_timestamp(1709164800), "2024-02-29");
    }

    #[test]
    fn test_format_timestamp_year_2000() {
        // 2000-01-01 00:00:00 UTC = 946684800
        assert_eq!(format_timestamp(946684800), "2000-01-01");
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000)); // divisible by 400
        assert!(!is_leap_year(1900)); // divisible by 100 but not 400
        assert!(is_leap_year(2024)); // divisible by 4 but not 100
        assert!(!is_leap_year(2023)); // not divisible by 4
    }

    #[test]
    fn test_parse_blame_porcelain_basic() {
        let porcelain = "\
abc1234567890abcdef1234567890abcdef123456 1 1 1
author Alice Smith
author-mail <alice@example.com>
author-time 1697328000
author-tz +0000
committer Alice Smith
committer-mail <alice@example.com>
committer-time 1697328000
committer-tz +0000
summary Initial commit
filename src/main.rs
\t// TODO: fix this
def4567890abcdef1234567890abcdef12345678 2 2 1
author Bob Jones
author-mail <bob@example.com>
author-time 1709164800
author-tz +0000
committer Bob Jones
committer-mail <bob@example.com>
committer-time 1709164800
committer-tz +0000
summary Second commit
filename src/main.rs
\tfn main() {}
";

        let result = parse_blame_porcelain(porcelain).unwrap();

        assert_eq!(result.len(), 2);

        let line1 = result.get(&1).expect("should have line 1");
        assert_eq!(line1.author, "Alice Smith");
        assert_eq!(line1.date, "2023-10-15");
        assert_eq!(
            line1.commit,
            "abc1234567890abcdef1234567890abcdef123456"
        );

        let line2 = result.get(&2).expect("should have line 2");
        assert_eq!(line2.author, "Bob Jones");
        assert_eq!(line2.date, "2024-02-29");
        assert_eq!(
            line2.commit,
            "def4567890abcdef1234567890abcdef12345678"
        );
    }

    #[test]
    fn test_parse_blame_porcelain_empty() {
        let result = parse_blame_porcelain("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_blame_porcelain_repeated_commit() {
        // When multiple lines share the same commit, porcelain only shows
        // the full header for the first occurrence and abbreviated for subsequent.
        let porcelain = "\
abc1234567890abcdef1234567890abcdef123456 1 1 3
author Alice Smith
author-mail <alice@example.com>
author-time 1697328000
author-tz +0000
committer Alice Smith
committer-mail <alice@example.com>
committer-time 1697328000
committer-tz +0000
summary Initial commit
filename src/main.rs
\tline one
abc1234567890abcdef1234567890abcdef123456 2 2
\tline two
abc1234567890abcdef1234567890abcdef123456 3 3
\tline three
";

        let result = parse_blame_porcelain(porcelain).unwrap();

        assert_eq!(result.len(), 3);

        // All three lines should have Alice's info (carried forward)
        for line_num in 1..=3 {
            let info = result.get(&line_num).unwrap();
            assert_eq!(info.author, "Alice Smith");
            assert_eq!(info.commit, "abc1234567890abcdef1234567890abcdef123456");
        }
    }
}
