use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "todos", about = "A fast, cross-language TODO linter", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output color mode
    #[arg(long, default_value = "auto", global = true)]
    pub color: ColorMode,

    /// Filter by tag (comma-separated: TODO,FIXME,HACK)
    #[arg(long, global = true)]
    pub tag: Option<String>,

    /// Filter by file glob pattern
    #[arg(long, global = true)]
    pub file: Option<String>,

    /// Filter by author (comma-separated)
    #[arg(long, global = true)]
    pub author: Option<String>,

    /// Filter by priority level (low, medium, high, critical)
    #[arg(long, global = true)]
    pub priority: Option<String>,

    /// Only show items with issue references
    #[arg(long, global = true)]
    pub has_issue: bool,

    /// Path to scan (defaults to current directory)
    #[arg(long, default_value = ".", global = true)]
    pub path: String,

    /// Output format: text, json, csv, markdown, count
    #[arg(long, default_value = "text", global = true)]
    pub format: String,

    /// Clear the scan cache before running
    #[arg(long, global = true)]
    pub clear_cache: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all TODOs (default command)
    List,
    /// Scan for TODOs (alias for list)
    Scan,
    /// Initialize a .todo-tracker.toml config file
    Init,
    /// Show TODO statistics with charts
    Stats,
    /// Compare TODOs between git refs
    Diff {
        /// Git ref range (e.g., main..HEAD) or --staged
        #[arg(default_value = "")]
        range: String,
        /// Compare staged changes
        #[arg(long)]
        staged: bool,
    },
    /// Run policy checks (for CI)
    Check {
        /// Maximum TODOs allowed
        #[arg(long)]
        max_todos: Option<usize>,
        /// Tags requiring issue refs (comma-separated)
        #[arg(long)]
        require_issue: Option<String>,
        /// Denied tags (comma-separated)
        #[arg(long)]
        deny: Option<String>,
        /// Only check files in the diff (requires git)
        #[arg(long)]
        diff_only: bool,
        /// Only check staged files
        #[arg(long)]
        staged_only: bool,
    },
    /// Show TODOs with git blame information
    Blame {
        /// Sort by field (date)
        #[arg(long)]
        sort: Option<String>,
        /// Show only TODOs since this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}
