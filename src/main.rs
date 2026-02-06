use anyhow::Result;
use clap::Parser;

use todo_tracker::cache::CacheDb;
use todo_tracker::cli::{Cli, ColorMode, Commands};
use todo_tracker::config::Config;
use todo_tracker::discovery::FileDiscovery;
use todo_tracker::filter::FilterCriteria;
use todo_tracker::model::{Priority, ScanResult, ScanStats};
use todo_tracker::output::{format_output, OutputFormat};
use todo_tracker::git::blame::enrich_with_blame;
use todo_tracker::git::diff::{diff_staged, diff_todos, DiffResult};
use todo_tracker::git::utils::{is_git_repo, repo_root};
use todo_tracker::policy::{check_policies, PolicyConfig};
use todo_tracker::scanner::regex::RegexScanner;
use todo_tracker::scanner::ScanOrchestrator;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle color mode
    match cli.color {
        ColorMode::Always => colored::control::set_override(true),
        ColorMode::Never => colored::control::set_override(false),
        ColorMode::Auto => {} // colored crate auto-detects TTY
    }

    // Handle commands
    match cli.command {
        Some(Commands::Init) => {
            let config_path = std::path::Path::new(".todo-tracker.toml");
            if config_path.exists() {
                eprintln!("Config file already exists: .todo-tracker.toml");
                std::process::exit(1);
            }
            std::fs::write(config_path, Config::default_template())?;
            println!("Created .todo-tracker.toml");
        }
        Some(Commands::Stats) => run_stats(&cli)?,
        Some(Commands::Diff { ref range, staged }) => run_diff(&cli, range, staged)?,
        Some(Commands::Check { ref max_todos, ref require_issue, ref deny, diff_only: _, staged_only: _ }) => {
            run_check(&cli, *max_todos, require_issue.clone(), deny.clone())?;
        }
        Some(Commands::Blame { ref sort, ref since }) => run_blame(&cli, sort.clone(), since.clone())?,
        Some(Commands::List) | Some(Commands::Scan) | None => run_scan(&cli)?,
    }

    Ok(())
}

fn build_filter(cli: &Cli) -> FilterCriteria {
    FilterCriteria {
        tags: cli
            .tag
            .as_ref()
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
        authors: cli
            .author
            .as_ref()
            .map(|a| a.split(',').map(|s| s.trim().to_string()).collect()),
        file_pattern: cli.file.clone(),
        priority: cli.priority.as_ref().and_then(|p| Priority::from_str_tag(p)),
        has_issue: if cli.has_issue { Some(true) } else { None },
    }
}

fn apply_filter(filter: &FilterCriteria, result: &mut ScanResult) {
    if !filter.is_empty() {
        let original_files_scanned = result.stats.files_scanned;
        result.items = filter.apply(&result.items);
        // Recompute stats after filtering
        result.stats = ScanStats::new();
        result.stats.files_scanned = original_files_scanned;
        let mut files_set = std::collections::HashSet::new();
        for item in &result.items {
            result.stats.add_item(item);
            files_set.insert(item.file.clone());
        }
        result.stats.files_with_todos = files_set.len();
    }
}

fn open_cache(cli: &Cli) -> Option<CacheDb> {
    let path = std::path::Path::new(&cli.path);
    match CacheDb::open(path) {
        Ok(db) => {
            if cli.clear_cache {
                let _ = db.clear();
            }
            Some(db)
        }
        Err(_) => None,
    }
}

fn run_scan(cli: &Cli) -> Result<()> {
    let scanner = RegexScanner::new()?;
    let discovery = FileDiscovery::new(&cli.path);
    let cache = open_cache(cli);
    let orchestrator = ScanOrchestrator::new(Box::new(scanner), discovery);

    let mut result = orchestrator.scan_with_cache(cache.as_ref())?;

    let filter = build_filter(cli);
    apply_filter(&filter, &mut result);

    let format = OutputFormat::from_str(&cli.format).map_err(|e| anyhow::anyhow!(e))?;

    let output = format_output(&result, format)?;
    print!("{}", output);

    Ok(())
}

fn run_stats(cli: &Cli) -> Result<()> {
    let scanner = RegexScanner::new()?;
    let discovery = FileDiscovery::new(&cli.path);
    let cache = open_cache(cli);
    let orchestrator = ScanOrchestrator::new(Box::new(scanner), discovery);

    let mut result = orchestrator.scan_with_cache(cache.as_ref())?;

    let filter = build_filter(cli);
    apply_filter(&filter, &mut result);

    // JSON output mode
    if cli.format == "json" {
        let json = serde_json::to_string_pretty(&result.stats)?;
        println!("{}", json);
        return Ok(());
    }

    // Text stats with Unicode bar charts
    print_stats(&result);

    Ok(())
}

fn print_stats(result: &ScanResult) {
    const MAX_BAR: usize = 20;

    println!("Tag Distribution:");
    if result.stats.by_tag.is_empty() {
        println!("  (no items found)");
    } else {
        let mut tag_counts: Vec<(&String, &usize)> = result.stats.by_tag.iter().collect();
        tag_counts.sort_by(|a, b| b.1.cmp(a.1));

        let max_count = *tag_counts.iter().map(|(_, c)| *c).max().unwrap_or(&1);
        let total = result.stats.total_todos;
        let max_label_len = tag_counts.iter().map(|(t, _)| t.len()).max().unwrap_or(0);

        for (tag, count) in &tag_counts {
            let bar_len = if max_count > 0 {
                (**count * MAX_BAR) / max_count
            } else {
                0
            }
            .max(1);
            let bar: String = "\u{2588}".repeat(bar_len);
            let pct = if total > 0 {
                (**count as f64 / total as f64 * 100.0) as usize
            } else {
                0
            };
            println!(
                "  {:<width$} {:20} {:>3} ({:>2}%)",
                tag,
                bar,
                count,
                pct,
                width = max_label_len
            );
        }
    }

    // Top files by TODO count
    println!();
    println!("Top Files (by TODO count):");
    let mut file_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for item in &result.items {
        *file_counts
            .entry(item.file.display().to_string())
            .or_insert(0) += 1;
    }

    if file_counts.is_empty() {
        println!("  (no items found)");
    } else {
        let mut file_list: Vec<(String, usize)> = file_counts.into_iter().collect();
        file_list.sort_by(|a, b| b.1.cmp(&a.1));
        file_list.truncate(10); // Show top 10

        let max_count = file_list.iter().map(|(_, c)| *c).max().unwrap_or(1);
        let max_label_len = file_list.iter().map(|(f, _)| f.len()).max().unwrap_or(0);

        for (file, count) in &file_list {
            let bar_len = if max_count > 0 {
                (*count * MAX_BAR) / max_count
            } else {
                0
            }
            .max(1);
            let bar: String = "\u{2588}".repeat(bar_len);
            println!(
                "  {:<width$} {:20} {}",
                file,
                bar,
                count,
                width = max_label_len
            );
        }
    }

    // Authors
    println!();
    println!("Authors:");
    let mut author_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for item in &result.items {
        if let Some(ref author) = item.author {
            *author_counts.entry(author.clone()).or_insert(0) += 1;
        }
    }

    if author_counts.is_empty() {
        println!("  (no authors found)");
    } else {
        let mut author_list: Vec<(String, usize)> = author_counts.into_iter().collect();
        author_list.sort_by(|a, b| b.1.cmp(&a.1));

        let max_count = author_list.iter().map(|(_, c)| *c).max().unwrap_or(1);
        let max_label_len = author_list.iter().map(|(a, _)| a.len()).max().unwrap_or(0);

        for (author, count) in &author_list {
            let bar_len = if max_count > 0 {
                (*count * MAX_BAR) / max_count
            } else {
                0
            }
            .max(1);
            let bar: String = "\u{2588}".repeat(bar_len);
            println!(
                "  {:<width$} {:20} {}",
                author,
                bar,
                count,
                width = max_label_len
            );
        }
    }

    // Summary line
    println!();
    println!(
        "Total: {} items in {} files ({} files scanned)",
        result.stats.total_todos, result.stats.files_with_todos, result.stats.files_scanned
    );
}

fn run_diff(cli: &Cli, range: &str, staged: bool) -> Result<()> {
    use colored::Colorize;

    let path = std::path::Path::new(&cli.path);
    if !is_git_repo(path) {
        anyhow::bail!("Not a git repository: {}", cli.path);
    }

    let root = repo_root(path).map_err(|e| anyhow::anyhow!(e))?;
    let scanner = RegexScanner::new()?;

    let result: DiffResult = if staged {
        diff_staged(&scanner, &root).map_err(|e| anyhow::anyhow!(e))?
    } else if range.is_empty() {
        anyhow::bail!("Specify a ref range (e.g., main..HEAD) or use --staged");
    } else {
        let parts: Vec<&str> = range.splitn(2, "..").collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            anyhow::bail!("Invalid range format. Use base..head (e.g., main..HEAD)");
        }
        diff_todos(&scanner, parts[0], parts[1], &root).map_err(|e| anyhow::anyhow!(e))?
    };

    // JSON output
    if cli.format == "json" {
        let json = serde_json::to_string_pretty(&result)?;
        println!("{}", json);
        return Ok(());
    }

    // Text output
    println!(
        "TODO diff: {} -> {}",
        result.base_ref, result.head_ref
    );
    println!();

    if result.added.is_empty() && result.removed.is_empty() {
        println!("No TODO changes detected.");
        return Ok(());
    }

    if !result.added.is_empty() {
        println!("{} ({}):", "Added".green().bold(), result.added.len());
        for item in &result.added {
            println!(
                "  {} {}:{} [{}] {}",
                "+".green(),
                item.file.display(),
                item.line,
                item.tag,
                item.message
            );
        }
        println!();
    }

    if !result.removed.is_empty() {
        println!("{} ({}):", "Removed".red().bold(), result.removed.len());
        for item in &result.removed {
            println!(
                "  {} {}:{} [{}] {}",
                "-".red(),
                item.file.display(),
                item.line,
                item.tag,
                item.message
            );
        }
        println!();
    }

    println!(
        "Summary: {} added, {} removed",
        result.added.len(),
        result.removed.len()
    );

    Ok(())
}

fn run_check(
    cli: &Cli,
    max_todos: Option<usize>,
    require_issue: Option<String>,
    deny: Option<String>,
) -> Result<()> {
    let scanner = RegexScanner::new()?;
    let discovery = FileDiscovery::new(&cli.path);
    let cache = open_cache(cli);
    let orchestrator = ScanOrchestrator::new(Box::new(scanner), discovery);

    let mut result = orchestrator.scan_with_cache(cache.as_ref())?;

    let filter = build_filter(cli);
    apply_filter(&filter, &mut result);

    let config = PolicyConfig {
        max_todos,
        require_issue: require_issue
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect()),
        deny_tags: deny
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect()),
        max_age_days: None,
    };

    let violations = check_policies(&result, &config);

    if violations.is_empty() {
        println!("All checks passed.");
        Ok(())
    } else {
        use colored::Colorize;
        for v in &violations {
            let prefix = match v.severity {
                todo_tracker::policy::ViolationSeverity::Error => "error".red().bold().to_string(),
                todo_tracker::policy::ViolationSeverity::Warning => "warning".yellow().bold().to_string(),
            };
            eprintln!("[{}] {}: {}", prefix, v.rule, v.message);
        }
        eprintln!();
        eprintln!("{} policy violation(s) found.", violations.len());
        std::process::exit(1);
    }
}

fn run_blame(cli: &Cli, sort: Option<String>, since: Option<String>) -> Result<()> {
    use colored::Colorize;

    let path = std::path::Path::new(&cli.path);
    if !is_git_repo(path) {
        anyhow::bail!("Not a git repository: {}", cli.path);
    }

    let root = repo_root(path).map_err(|e| anyhow::anyhow!(e))?;
    let scanner = RegexScanner::new()?;
    let root_str = root.to_str().unwrap_or(".");
    let discovery = FileDiscovery::new(root_str);
    let cache = open_cache(cli);
    let orchestrator = ScanOrchestrator::new(Box::new(scanner), discovery);

    let mut result = orchestrator.scan_with_cache(cache.as_ref())?;

    let filter = build_filter(cli);
    apply_filter(&filter, &mut result);

    enrich_with_blame(&mut result.items, &root);

    // Filter by --since if provided
    if let Some(ref since_date) = since {
        result.items.retain(|item| {
            item.git_date
                .as_ref()
                .map(|d| d.as_str() >= since_date.as_str())
                .unwrap_or(false)
        });
    }

    // Sort by date if requested
    if let Some(ref sort_field) = sort {
        if sort_field == "date" {
            result.items.sort_by(|a, b| {
                let da = a.git_date.as_deref().unwrap_or("");
                let db = b.git_date.as_deref().unwrap_or("");
                db.cmp(da)
            });
        }
    }

    // JSON output
    if cli.format == "json" {
        let json = serde_json::to_string_pretty(&result)?;
        println!("{}", json);
        return Ok(());
    }

    // Text output
    for item in &result.items {
        let author = item.git_author.as_deref().unwrap_or("unknown");
        let date = item.git_date.as_deref().unwrap_or("unknown");
        println!(
            "{} {} {} {}:{} [{}] {}",
            date.dimmed(),
            author.cyan(),
            "|".dimmed(),
            item.file.display(),
            item.line,
            item.tag.to_string().yellow(),
            item.message
        );
    }

    if result.items.is_empty() {
        println!("No TODOs found.");
    } else {
        println!();
        println!("Total: {} items", result.items.len());
    }

    Ok(())
}
