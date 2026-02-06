# Developer Guide: todo-tracker CLI (`todos`)

Welcome to the comprehensive developer guide for the `todo-tracker` Rust CLI project. This guide is designed for developers who may have experience with C/C++, Java, or other languages but are **new to Rust** and modern CLI tooling. Every step is explicit—no assumed knowledge.

## Table of Contents

1. [Prerequisites and Environment Setup](#1-prerequisites-and-environment-setup)
2. [Getting the Source Code](#2-getting-the-source-code)
3. [Understanding the Build System](#3-understanding-the-build-system)
4. [Understanding the Architecture](#4-understanding-the-architecture)
5. [Running Tests](#5-running-tests)
6. [Adding New Features](#6-adding-new-features-step-by-step-guides)
7. [Common Tasks](#7-common-tasks)
8. [CI/CD Pipeline](#8-cicd-pipeline)
9. [Docker](#9-docker)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Prerequisites and Environment Setup

### Installing Rust

**What is Rust?**

Rust is a compiled, systems programming language that emphasizes memory safety, concurrency, and performance. Think of it as combining the speed of C++ with memory safety guarantees that prevent entire classes of bugs (null pointer dereferences, buffer overflows, use-after-free errors).

Unlike C++, Rust has a "borrow checker" that enforces memory safety rules at compile time, so you don't need to manually manage memory or worry about data races in concurrent code.

**Why Rust for this project?**

- **Compiled**: Single binary, no runtime dependencies (like Go or C++)
- **Fast**: Performance comparable to C/C++
- **Safe**: Prevents memory errors at compile time
- **Ecosystem**: Excellent package management with Cargo

**Installing on Windows:**

1. Download the installer from https://rustup.rs/
2. Run `rustup-init.exe`
3. Follow the prompts (default options are recommended)
4. The installer will download Visual Studio Build Tools if needed (required for linking)
5. Open a new command prompt or PowerShell window
6. Verify installation:
   ```powershell
   rustc --version
   cargo --version
   ```
   You should see output like:
   ```
   rustc 1.75.0 (82e1608df 2023-12-21)
   cargo 1.75.0 (1d8b05cdd 2023-11-20)
   ```

**Installing on macOS:**

1. Open Terminal
2. Run:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
3. Follow the prompts (default options are recommended)
4. The script will modify your shell profile (`.bashrc`, `.zshrc`, etc.)
5. Restart your terminal or run:
   ```bash
   source "$HOME/.cargo/env"
   ```
6. Verify installation:
   ```bash
   rustc --version
   cargo --version
   ```

**Installing on Linux:**

1. Open your terminal
2. Run:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
3. Follow the prompts (default options are recommended)
4. Restart your terminal or run:
   ```bash
   source "$HOME/.cargo/env"
   ```
5. Install build dependencies (Ubuntu/Debian):
   ```bash
   sudo apt-get update
   sudo apt-get install build-essential
   ```
   For other distros, install your distribution's equivalent of `gcc`, `make`, and development headers.

6. Verify installation:
   ```bash
   rustc --version
   cargo --version
   ```

**What is Cargo?**

Cargo is Rust's build system and package manager. Think of it as:
- **Maven/Gradle** for Java developers
- **npm/yarn** for JavaScript developers
- **CMake + vcpkg** for C++ developers
- **pip + setuptools** for Python developers

Cargo handles:
- Building your project (`cargo build`)
- Running tests (`cargo test`)
- Managing dependencies (declared in `Cargo.toml`)
- Running your binary (`cargo run`)
- Publishing packages to crates.io (the Rust package registry)
- Formatting code (`cargo fmt`)
- Linting code (`cargo clippy`)

### Installing Git

**Why Git is needed:**

Git is essential for this project because:
1. The `todos blame` command uses `git blame` to show who authored each TODO
2. The `todos diff` command compares TODOs between git branches
3. Version control for your own development

**Installing on Windows:**

1. Download Git from https://git-scm.com/download/win
2. Run the installer
3. Use default options (or customize if you have preferences)
4. Open a new command prompt or PowerShell
5. Verify:
   ```powershell
   git --version
   ```
   Should output something like: `git version 2.43.0.windows.1`

**Installing on macOS:**

Option 1 - Using Homebrew (recommended):
```bash
brew install git
```

Option 2 - Using Xcode Command Line Tools:
```bash
xcode-select --install
```

Verify:
```bash
git --version
```

**Installing on Linux:**

Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install git
```

Fedora/RHEL/CentOS:
```bash
sudo dnf install git
```

Arch:
```bash
sudo pacman -S git
```

Verify:
```bash
git --version
```

### Editor Setup

**Recommended: VS Code with rust-analyzer**

Visual Studio Code with the rust-analyzer extension provides an excellent Rust development experience.

**Step 1: Install VS Code**

Download from https://code.visualstudio.com/ and install for your platform.

**Step 2: Install rust-analyzer Extension**

1. Open VS Code
2. Click the Extensions icon in the sidebar (or press `Ctrl+Shift+X` / `Cmd+Shift+X`)
3. Search for "rust-analyzer"
4. Click "Install" on the extension by "The Rust Programming Language Team"
5. Wait for installation to complete

**What rust-analyzer provides:**

- **Autocomplete**: Intelligent code completion as you type
- **Inline errors**: Red squiggles under errors with explanations
- **Go-to-definition**: `Ctrl+Click` on any function/type to jump to its definition
- **Type hints**: Shows inferred types inline
- **Code actions**: Quick fixes and refactorings (lightbulb icon)
- **Documentation on hover**: Hover over any function to see its docs
- **Run/Debug buttons**: Inline buttons to run tests

**Alternative editors:**

- **IntelliJ IDEA / CLion**: Excellent Rust support via the IntelliJ Rust plugin
- **Vim/Neovim**: Use rust-analyzer via LSP with plugins like `coc.nvim` or `nvim-lspconfig`
- **Emacs**: Use rust-analyzer via LSP mode

---

## 2. Getting the Source Code

### Cloning the Repository

If you haven't already:

```bash
git clone https://github.com/az9713/claude-agent-teams-deepdive.git
cd todo-tracker
```

Or if you're reading this guide locally, you're already in the right place.

### Directory Structure Explained

Here's what each directory and file does:

```
todo-tracker/
│
├── Cargo.toml                      # Project manifest (dependencies, metadata)
├── Cargo.lock                      # Exact dependency versions (like package-lock.json)
├── README.md                       # Project overview
├── Dockerfile                      # Docker container definition
│
├── .github/
│   └── workflows/
│       ├── ci.yml                  # Continuous Integration pipeline
│       └── release.yml             # Release automation (binary builds for GitHub)
│
├── src/                            # All Rust source code
│   ├── main.rs                     # Entry point - CLI argument parsing, command dispatch
│   ├── lib.rs                      # Library root - re-exports public API
│   ├── cli.rs                      # CLI definitions using clap (structs for arguments)
│   ├── error.rs                    # Custom error types (TodoError enum)
│   ├── model.rs                    # Core data structures (TodoItem, Priority, TodoTag)
│   ├── config.rs                   # Configuration file handling (.todo-tracker.toml)
│   ├── discovery.rs                # File discovery (respects .gitignore, follows symlinks)
│   ├── filter.rs                   # Filtering logic (by tag, author, priority, file pattern)
│   ├── progress.rs                 # Progress bar using indicatif
│   ├── policy.rs                   # CI policy engine (max-todos, require-issue, deny tags)
│   │
│   ├── scanner/                    # All scanning implementations
│   │   ├── mod.rs                  # FileScanner trait + ScanOrchestrator
│   │   ├── languages.rs            # Language database (comment syntax for 30+ languages)
│   │   ├── regex.rs                # Fast regex-based scanner (default)
│   │   ├── treesitter.rs           # AST-based scanner using tree-sitter (feature-gated)
│   │   ├── incremental.rs          # Incremental scanning (uses cache to skip unchanged files)
│   │   └── mmap.rs                 # Memory-mapped file reading (performance optimization)
│   │
│   ├── output/                     # All output formatters
│   │   ├── mod.rs                  # OutputFormatter trait + format_output() dispatcher
│   │   ├── text.rs                 # Colorized terminal output (default)
│   │   ├── json.rs                 # JSON output
│   │   ├── csv.rs                  # CSV output
│   │   ├── markdown.rs             # Markdown table output
│   │   ├── sarif.rs                # SARIF 2.1.0 (for GitHub Code Scanning)
│   │   └── github_actions.rs      # GitHub Actions workflow command format
│   │
│   ├── git/                        # Git integration
│   │   ├── mod.rs                  # Public API for git features
│   │   ├── utils.rs                # Common git utilities (is_repo, get_root_dir)
│   │   ├── blame.rs                # Git blame integration (author, date enrichment)
│   │   └── diff.rs                 # Git diff integration (compare TODOs across branches)
│   │
│   └── cache/                      # SQLite caching for incremental scans
│       ├── mod.rs                  # Public API for cache operations
│       ├── db.rs                   # SQLite connection and queries
│       └── migrations.rs           # Database schema and migrations
│
└── tests/                          # Integration tests
    ├── cli_test.rs                 # CLI integration tests using assert_cmd
    ├── integration/                # Additional integration test modules
    └── fixtures/                   # Test data (sample source files)
        ├── simple.rs               # Rust file with TODOs
        ├── todos.py                # Python file with TODOs
        ├── sample.js               # JavaScript file with TODOs
        └── ...                     # More test files for various languages
```

**Key takeaways:**
- `src/` contains all implementation code
- `tests/` contains integration tests (unit tests live inside each `.rs` file)
- `Cargo.toml` is the single source of truth for dependencies
- `.github/workflows/` contains CI/CD automation

---

## 3. Understanding the Build System

### Cargo.toml Explained

`Cargo.toml` is the project manifest. Think of it as:
- `pom.xml` in Maven
- `package.json` in npm
- `CMakeLists.txt` in CMake
- `setup.py` in Python

Let's break down each section:

```toml
[package]
name = "todo-tracker"          # Crate name (package name in crates.io)
version = "0.1.0"              # Semantic version
edition = "2021"               # Rust edition (language version)

[[bin]]
name = "todos"                 # Binary name (the executable you run)
path = "src/main.rs"           # Entry point
```

**What are editions?** Rust releases new language editions every 3 years (2015, 2018, 2021). Editions allow backwards-incompatible improvements without breaking old code. Edition 2021 is the latest stable edition.

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }   # CLI argument parsing
ignore = "0.4"                                     # .gitignore parsing
regex = "1"                                        # Regular expressions
serde = { version = "1", features = ["derive"] }   # Serialization framework
serde_json = "1"                                   # JSON support for serde
toml = "0.8"                                       # TOML parsing (for config files)
thiserror = "2"                                    # Error handling macros
anyhow = "1"                                       # Ergonomic error handling
colored = "2"                                      # Terminal colors
rayon = "1"                                        # Data parallelism
crossbeam-channel = "0.5"                          # Multi-producer multi-consumer channels
csv = "1"                                          # CSV output
unicode-width = "0.2"                              # Unicode text width calculation
tempfile = "3"                                     # Temporary file/directory creation
rusqlite = { version = "0.32", features = ["bundled"] }  # SQLite (bundled = no system dep)
indicatif = "0.17"                                 # Progress bars
memmap2 = "0.9"                                    # Memory-mapped file I/O
```

**Features explained:** The `features = ["derive"]` syntax enables optional features of a crate. For example:
- `clap` with `derive` feature enables `#[derive(Parser)]` macros
- `serde` with `derive` feature enables `#[derive(Serialize, Deserialize)]` macros
- `rusqlite` with `bundled` feature compiles SQLite from source (no system library needed)

```toml
tree-sitter = { version = "0.24", optional = true }
streaming-iterator = { version = "0.1", optional = true }
tree-sitter-rust = { version = "0.23", optional = true }
tree-sitter-javascript = { version = "0.23", optional = true }
tree-sitter-python = { version = "0.23", optional = true }
tree-sitter-go = { version = "0.23", optional = true }
tree-sitter-java = { version = "0.23", optional = true }
tree-sitter-c = { version = "0.23", optional = true }
tree-sitter-cpp = { version = "0.23", optional = true }
tree-sitter-ruby = { version = "0.23", optional = true }
```

**Optional dependencies:** These are only included if the `precise` feature is enabled. This is like `#ifdef TREE_SITTER` in C/C++.

```toml
[features]
default = []
precise = [
    "tree-sitter",
    "streaming-iterator",
    "tree-sitter-rust",
    "tree-sitter-javascript",
    "tree-sitter-python",
    "tree-sitter-go",
    "tree-sitter-java",
    "tree-sitter-c",
    "tree-sitter-cpp",
    "tree-sitter-ruby",
]
```

**Feature flags explained:**
- `default = []` means no features are enabled by default
- `precise` is a custom feature that enables all tree-sitter dependencies
- You enable it with `cargo build --features precise`
- In code, you gate features with `#[cfg(feature = "precise")]`

**Why use feature flags?** Tree-sitter dependencies are large and slow to compile. Most users just want the fast regex scanner. Power users can opt into the precise AST-based scanner.

```toml
[dev-dependencies]
assert_cmd = "2"               # Test CLI applications
predicates = "3"               # Assertions for assert_cmd
tempfile = "3"                 # Already in [dependencies], listed here for clarity
insta = { version = "1", features = ["yaml"] }  # Snapshot testing
```

**Dev dependencies:** Only used for tests, not included in the final binary.

### Building

**Debug build (fast compile, slow run, includes debug symbols):**

```bash
cargo build
```

Output: `target/debug/todos` (or `todos.exe` on Windows)

This is the default. Debug builds:
- Compile quickly (no optimizations)
- Run slowly (often 10-100x slower than release)
- Include debug symbols for debuggers
- Perform additional runtime checks (overflow checks, etc.)

**Release build (slow compile, fast run, optimized):**

```bash
cargo build --release
```

Output: `target/release/todos` (or `todos.exe` on Windows)

Release builds:
- Compile slowly (LLVM optimizations take time)
- Run fast (comparable to C/C++ `-O3`)
- Strip debug symbols (smaller binary)
- Use release builds for benchmarking or production

**Build with the `precise` feature:**

```bash
cargo build --features precise
```

This enables tree-sitter support. Compilation takes significantly longer because it builds 8 tree-sitter parsers (Rust, JavaScript, Python, Go, Java, C, C++, Ruby) from C source.

**Check without building (fastest):**

```bash
cargo check
```

This compiles your code but doesn't produce a binary. It's the fastest way to check for errors during development. rust-analyzer runs this in the background as you type.

**Where do binaries go?**

- Debug: `target/debug/todos`
- Release: `target/release/todos`
- With features: Same directories, just with extra dependencies compiled in

You can run the binary directly:
```bash
./target/debug/todos list --path ./src
```

Or on Windows:
```powershell
.\target\debug\todos.exe list --path .\src
```

### Running

**Run the debug binary with arguments:**

```bash
cargo run -- list --path ./src
```

The `--` separates Cargo's arguments from your program's arguments.

**Examples:**

```bash
# Show help
cargo run -- --help

# List all TODOs in current directory
cargo run -- list

# List TODOs with JSON output
cargo run -- list --format=json

# Run with the precise feature enabled
cargo run --features precise -- list --path ./src

# Run the release build
cargo run --release -- list --path ./src
```

**Why use `cargo run`?** It automatically rebuilds if source code changed, then runs the binary. During development, this is more convenient than manually running `cargo build` then `./target/debug/todos`.

---

## 4. Understanding the Architecture

### The Big Picture

Here's the data flow from start to finish:

```
                Files on disk
                      |
                      v
    +----------------------------------+
    |   FileDiscovery (discovery.rs)   |  <-- Finds files, respects .gitignore
    +----------------------------------+
                      |
                      v
                List of file paths
                      |
                      v
    +----------------------------------+
    |   FileScanner (scanner/*.rs)     |  <-- Scans each file for TODO patterns
    |   - regex.rs (default)           |      (parallel processing with rayon)
    |   - treesitter.rs (optional)     |
    +----------------------------------+
                      |
                      v
              Vec<TodoItem>              <-- Raw results
                      |
                      v
    +----------------------------------+
    |   FilterCriteria (filter.rs)     |  <-- Optional: filter by tag/author/priority
    +----------------------------------+
                      |
                      v
           Filtered Vec<TodoItem>
                      |
                      v
    +----------------------------------+
    |   OutputFormatter (output/*.rs)  |  <-- Format as text/json/csv/markdown/sarif
    +----------------------------------+
                      |
                      v
                   stdout                <-- Printed to terminal or piped to file
```

**Parallel processing:** FileScanner uses Rayon to scan multiple files simultaneously. On a quad-core CPU with 100 files, this means 4 files are scanned at once, dramatically reducing total scan time.

**Incremental scanning (when cache is enabled):** FileDiscovery checks SQLite cache for each file's hash. If the file hasn't changed since last scan, its cached TODOs are reused without re-scanning.

### Module-by-Module Walkthrough

For each module, we'll cover:
- What it does (in plain English)
- Key types (structs, enums, traits)
- How it connects to other modules
- File path

#### 1. `error.rs` - Error Types

**What it does:** Defines all error types used throughout the application.

**Key types:**
- `TodoError` (enum): All possible errors (file not found, regex compilation failed, git error, etc.)

**Pattern:** Uses the `thiserror` crate for automatic `Display` and `Error` trait implementations.

Example:
```rust
#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Git error: {0}")]
    GitError(String),
}
```

**How it connects:** Every module returns `Result<T, TodoError>` or `anyhow::Result<T>`.

#### 2. `model.rs` - Core Data Types

**What it does:** Defines the core domain models that represent TODOs, tags, priorities, and scan results.

**Key types:**

```rust
pub enum Priority {
    Low,      // p:low, p3
    Medium,   // p:medium, p2
    High,     // p:high, p1
    Critical, // p:critical, p0
}

pub enum TodoTag {
    Todo,            // TODO
    Fixme,           // FIXME
    Hack,            // HACK
    Bug,             // BUG
    Xxx,             // XXX
    Custom(String),  // Any other tag
}

pub struct TodoItem {
    pub tag: TodoTag,               // TODO, FIXME, etc.
    pub message: String,            // The comment text
    pub file: PathBuf,              // File path
    pub line: usize,                // Line number (1-indexed)
    pub column: usize,              // Column number (0-indexed)
    pub author: Option<String>,     // Extracted from @author annotation
    pub issue: Option<String>,      // Extracted from #123 or JIRA-456
    pub priority: Option<Priority>, // Extracted from p:high
    pub context_line: String,       // Full line of source code
    pub git_author: Option<String>, // From git blame
    pub git_date: Option<String>,   // From git blame
}

pub struct ScanStats {
    pub files_scanned: usize,
    pub files_with_todos: usize,
    pub total_todos: usize,
    pub by_tag: HashMap<String, usize>,  // Count per tag
}
```

**How it connects:** Every module works with `TodoItem`. Scanners produce them, filters filter them, formatters format them.

#### 3. `scanner/languages.rs` - Language Database

**What it does:** Maps file extensions to comment syntax for 30+ programming languages.

**Key types:**

```rust
pub struct Language {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub line_comment: Option<&'static str>,        // e.g., "//" for Rust
    pub block_comment: Option<(&'static str, &'static str)>,  // e.g., ("/*", "*/")
}

pub struct LanguageDatabase {
    languages: Vec<Language>,
}
```

**Example:**
```rust
Language {
    name: "Rust",
    extensions: &["rs"],
    line_comment: Some("//"),
    block_comment: Some(("/*", "*/")),
}
```

**How it connects:** Used by `scanner/regex.rs` to build appropriate regex patterns for each file type.

#### 4. `scanner/regex.rs` - The Regex Scanner (Heart of the Tool)

**What it does:** Scans files using regular expressions to find TODO comments. This is the default scanner—fast, simple, works for all languages.

**Key logic:**

1. Determine language from file extension (via `LanguageDatabase`)
2. Build regex pattern for that language's comment syntax
3. Read file line-by-line (or via mmap for large files)
4. Apply regex to each line
5. Extract tag, message, author, issue, priority from matched text
6. Build `TodoItem` for each match

**Pattern example:**

For Rust (line comment `//`):
```regex
//\s*(TODO|FIXME|HACK|BUG|XXX):?\s*(.*)
```

This matches:
- `// TODO: implement caching`
- `// FIXME fix the buffer overflow`
- `//TODO(alice) #123 high priority bug`

**How it connects:**
- Implements `FileScanner` trait (defined in `scanner/mod.rs`)
- Used by `ScanOrchestrator` to scan files in parallel

#### 5. `scanner/mod.rs` - FileScanner Trait + ScanOrchestrator

**What it does:** Defines the `FileScanner` trait (interface) and orchestrates parallel scanning.

**Key types:**

```rust
pub trait FileScanner: Send + Sync {
    fn scan_file(&self, path: &Path) -> Result<Vec<TodoItem>, TodoError>;
}

pub struct ScanOrchestrator {
    scanner: Arc<dyn FileScanner>,
    thread_pool: rayon::ThreadPool,
}
```

**The trait pattern:** This is like an interface in Java or a pure virtual class in C++. Any type can implement `FileScanner` by providing a `scan_file` method. This allows swapping scanners (regex vs tree-sitter) without changing calling code.

**How it connects:**
- `RegexScanner` and `TreeSitterScanner` both implement `FileScanner`
- `ScanOrchestrator` accepts any `FileScanner` and parallelizes it across files
- Called from `main.rs` to perform the actual scan

#### 6. `discovery.rs` - File Discovery

**What it does:** Finds all files to scan in a directory tree, respecting `.gitignore`, following (or skipping) symlinks, and applying size/extension filters.

**Key types:**

```rust
pub struct FileDiscovery {
    max_file_size: Option<u64>,
    extensions: Option<Vec<String>>,
    respect_gitignore: bool,
    follow_symlinks: bool,
}
```

**Builder pattern example:**

```rust
let discovery = FileDiscovery::new()
    .with_max_file_size(1024 * 1024)  // 1 MB
    .with_extensions(vec!["rs", "py", "js"])
    .with_respect_gitignore(true);

let files = discovery.discover(&Path::new("./src"))?;
```

**How it connects:**
- Uses the `ignore` crate (same library that powers `ripgrep`)
- Returns `Vec<PathBuf>` to be scanned by `ScanOrchestrator`

#### 7. `filter.rs` - Filtering Engine

**What it does:** Filters a list of `TodoItem` based on criteria (tag, author, file pattern, priority, has_issue).

**Key types:**

```rust
pub struct FilterCriteria {
    pub tags: Option<Vec<TodoTag>>,
    pub authors: Option<Vec<String>>,
    pub file_pattern: Option<glob::Pattern>,
    pub priority: Option<Priority>,
    pub has_issue: bool,
}

pub fn apply_filters(items: Vec<TodoItem>, criteria: &FilterCriteria) -> Vec<TodoItem>
```

**How it connects:**
- Called after scanning, before formatting
- Criteria comes from CLI arguments (parsed by `cli.rs`)

#### 8. `output/` - All Formatters

**What it does:** Each file in `output/` implements one output format.

**The trait:**

```rust
pub trait OutputFormatter {
    fn format(&self, items: &[TodoItem], stats: &ScanStats) -> Result<String, TodoError>;
}
```

**Implementations:**
- `text.rs`: Colorized terminal output (default) - uses `colored` crate
- `json.rs`: JSON array of objects - uses `serde_json`
- `csv.rs`: CSV with headers - uses `csv` crate
- `markdown.rs`: Markdown table
- `sarif.rs`: SARIF 2.1.0 format for GitHub Code Scanning
- `github_actions.rs`: GitHub Actions workflow command format (annotations)

**Dispatching:**

In `output/mod.rs`:
```rust
pub fn format_output(format: &str, items: &[TodoItem], stats: &ScanStats) -> Result<String, TodoError> {
    match format {
        "text" => TextFormatter::new().format(items, stats),
        "json" => JsonFormatter::new().format(items, stats),
        "csv" => CsvFormatter::new().format(items, stats),
        "markdown" => MarkdownFormatter::new().format(items, stats),
        "sarif" => SarifFormatter::new().format(items, stats),
        "github-actions" => GithubActionsFormatter::new().format(items, stats),
        _ => Err(TodoError::UnsupportedFormat(format.to_string())),
    }
}
```

**How it connects:**
- Called from `main.rs` after filtering
- Format string comes from `--format` CLI argument

#### 9. `config.rs` - Configuration

**What it does:** Loads and parses `.todo-tracker.toml` configuration files.

**Example config:**

```toml
[scan]
max_file_size_mb = 10
extensions = ["rs", "py", "js", "go"]

[policy]
max_todos = 100
require_issue = ["FIXME", "BUG"]
deny = ["NOCOMMIT", "XXX"]
```

**Key types:**

```rust
pub struct Config {
    pub scan: ScanConfig,
    pub policy: PolicyConfig,
}
```

**How it connects:**
- Loaded in `main.rs` if `.todo-tracker.toml` exists
- Merged with CLI arguments (CLI takes precedence)

#### 10. `git/` - Git Integration

**Three modules:**

**`git/utils.rs`:** Common utilities
- `is_git_repo()`: Check if directory is a git repo
- `get_repo_root()`: Find the `.git` directory
- Uses `std::process::Command` to run `git` commands

**`git/blame.rs`:** Git blame integration
- For each `TodoItem`, run `git blame -L <line>,<line> <file>`
- Parse output to extract author and date
- Populate `TodoItem.git_author` and `TodoItem.git_date`

**`git/diff.rs`:** Git diff integration
- Run `git diff <ref>` or `git diff --staged`
- Parse diff output to find added/removed/modified TODOs
- Return three lists: added, removed, unchanged

**How it connects:**
- Used by `todos blame` command to enrich TODOs with author info
- Used by `todos diff` command to compare TODOs across branches
- Used by `todos check --diff-only` to only check files in current diff

#### 11. `policy.rs` - CI Policy Engine

**What it does:** Enforces TODO policies for CI/CD pipelines. Exits with non-zero code if policies are violated.

**Key types:**

```rust
pub struct PolicyConfig {
    pub max_todos: Option<usize>,
    pub require_issue: Option<Vec<String>>,  // Tags requiring issue refs
    pub deny: Option<Vec<String>>,           // Forbidden tags
}

pub struct PolicyViolation {
    pub rule: String,
    pub message: String,
    pub item: Option<TodoItem>,
}

pub fn check_policies(items: &[TodoItem], config: &PolicyConfig) -> Vec<PolicyViolation>
```

**Example violations:**

```rust
PolicyViolation {
    rule: "max_todos",
    message: "Found 150 TODOs, but max_todos is 100",
    item: None,
}

PolicyViolation {
    rule: "require_issue",
    message: "FIXME requires an issue reference",
    item: Some(todo_item),  // The offending TODO
}

PolicyViolation {
    rule: "deny",
    message: "Tag NOCOMMIT is forbidden",
    item: Some(todo_item),
}
```

**How it connects:**
- Used by `todos check` command
- Returns violations, which are formatted and printed
- `main.rs` exits with code 1 if any violations exist

#### 12. `cache/` - SQLite Caching

**What it does:** Stores file hashes and their TODOs in SQLite. On subsequent scans, if a file's hash matches, reuse cached TODOs instead of re-scanning.

**Three modules:**

**`cache/db.rs`:** SQLite connection and queries
- `CacheDb::new()`: Open/create SQLite database
- `get_cached_scan()`: Retrieve cached TODOs for a file
- `store_scan_result()`: Store TODOs for a file
- Uses `rusqlite` crate

**`cache/migrations.rs`:** Database schema
```sql
CREATE TABLE file_scans (
    file_path TEXT PRIMARY KEY,
    hash TEXT NOT NULL,
    todos BLOB NOT NULL,  -- Serialized Vec<TodoItem>
    scanned_at INTEGER NOT NULL
);
```

**`cache/mod.rs`:** Public API
- `Cache::new()`: Initialize cache
- `Cache::get()`: Get cached items
- `Cache::set()`: Store items
- `Cache::clear()`: Delete all cache entries

**How it connects:**
- Used by `scanner/incremental.rs` (incremental scanner wrapper)
- Controlled by `--clear-cache` flag

#### 13. `scanner/incremental.rs` + `mmap.rs` - Performance Features

**`scanner/incremental.rs`:** Incremental scanning
- Wraps any `FileScanner`
- Before scanning, check cache for file hash
- If hash matches, return cached TODOs
- If hash differs or not cached, scan and update cache

**`scanner/mmap.rs`:** Memory-mapped I/O
- For files > 1 MB, use `memmap2` to memory-map the file
- This is faster than `read_to_string()` for large files
- OS handles paging, so you can "read" gigabyte files without loading into RAM

**How it connects:**
- `IncrementalScanner` wraps `RegexScanner` or `TreeSitterScanner`
- `mmap.rs` is used by `RegexScanner` for large files

#### 14. `scanner/treesitter.rs` - Tree-sitter Precision (Feature-Gated)

**What it does:** Uses tree-sitter (a parser library) to build an Abstract Syntax Tree (AST), then searches for TODO comments in comment nodes. This is more accurate than regex because it understands language syntax.

**Why it's better than regex:**
- Ignores TODOs in strings: `let s = "TODO: not a real todo";`
- Handles multi-line comments correctly
- Knows the exact node type (line comment vs block comment)

**Why it's feature-gated:**
- Tree-sitter parsers are large (adds ~10 MB to binary)
- Compilation is slow (tree-sitter compiles from C source)
- Parsing is slower than regex for most files
- Only useful for 100% precision in complex codebases

**Enabled with:**
```bash
cargo build --features precise
```

**Guarded in code:**
```rust
#[cfg(feature = "precise")]
pub mod treesitter;
```

**How it connects:**
- Implements `FileScanner` trait
- Selected when `--precise` flag is used (requires feature enabled at compile time)

#### 15. `cli.rs` - CLI Definition

**What it does:** Defines the command-line interface using the `clap` crate.

**Key types:**

```rust
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // Global flags
    pub color: ColorMode,
    pub tag: Option<String>,
    pub author: Option<String>,
    pub priority: Option<String>,
    pub has_issue: bool,
    pub path: String,
    pub format: String,
    pub clear_cache: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    List,
    Scan,
    Init,
    Stats,
    Diff { range: String, staged: bool },
    Check { max_todos: Option<usize>, require_issue: Option<String>, deny: Option<String>, diff_only: bool, staged_only: bool },
    Blame { sort: Option<String>, since: Option<String> },
}
```

**How clap works:** The `#[derive(Parser)]` macro generates argument parsing code at compile time. This is similar to Java annotations that generate code, but it's done at compile time in Rust.

**How it connects:**
- Parsed in `main.rs` via `Cli::parse()`
- Produces a `Cli` struct with all arguments populated
- Dispatched in `main.rs` match statement

#### 16. `main.rs` - Wiring Everything Together

**What it does:** Entry point. Parses CLI, loads config, dispatches to command handlers, handles errors.

**Structure:**

```rust
fn main() -> anyhow::Result<()> {
    // 1. Parse CLI arguments
    let cli = Cli::parse();

    // 2. Load config file (if exists)
    let config = Config::load(".todo-tracker.toml")?;

    // 3. Dispatch to command handler
    match cli.command {
        Some(Commands::List) | Some(Commands::Scan) => run_list(&cli, &config)?,
        Some(Commands::Init) => run_init()?,
        Some(Commands::Stats) => run_stats(&cli, &config)?,
        Some(Commands::Diff { range, staged }) => run_diff(&cli, &config, &range, staged)?,
        Some(Commands::Check { .. }) => run_check(&cli, &config)?,
        Some(Commands::Blame { .. }) => run_blame(&cli, &config)?,
        None => run_list(&cli, &config)?,  // Default command
    }

    Ok(())
}

fn run_list(cli: &Cli, config: &Config) -> anyhow::Result<()> {
    // 1. Discover files
    let files = FileDiscovery::new()
        .with_config(&config.scan)
        .discover(&Path::new(&cli.path))?;

    // 2. Scan files
    let scanner = RegexScanner::new();
    let orchestrator = ScanOrchestrator::new(Arc::new(scanner));
    let (items, stats) = orchestrator.scan_files(&files)?;

    // 3. Apply filters
    let criteria = FilterCriteria::from_cli(cli);
    let filtered = apply_filters(items, &criteria);

    // 4. Format output
    let output = format_output(&cli.format, &filtered, &stats)?;
    println!("{}", output);

    Ok(())
}
```

**Error propagation:** The `?` operator automatically returns errors. If `discover()` fails, the error propagates up to `main()`, which prints it and exits with code 1.

#### 17. `progress.rs` - Progress Bar

**What it does:** Shows a progress bar during scanning using the `indicatif` crate.

**Example:**
```
Scanning files... [#################---] 85% (850/1000 files)
```

**How it connects:**
- Used by `ScanOrchestrator` to report progress
- Automatically hidden when stdout is redirected (e.g., `todos list > output.txt`)

### Key Design Patterns

These patterns are common in Rust and throughout this codebase.

#### Trait-Based Polymorphism

**What it is:** Like interfaces in Java or abstract classes in C++. A trait defines a contract; types implement the trait.

**Example:**

```rust
pub trait FileScanner: Send + Sync {
    fn scan_file(&self, path: &Path) -> Result<Vec<TodoItem>, TodoError>;
}

pub struct RegexScanner { /* fields */ }
pub struct TreeSitterScanner { /* fields */ }

impl FileScanner for RegexScanner {
    fn scan_file(&self, path: &Path) -> Result<Vec<TodoItem>, TodoError> {
        // Regex-based implementation
    }
}

impl FileScanner for TreeSitterScanner {
    fn scan_file(&self, path: &Path) -> Result<Vec<TodoItem>, TodoError> {
        // AST-based implementation
    }
}

// Use any scanner polymorphically
fn use_scanner(scanner: &dyn FileScanner) {
    scanner.scan_file(Path::new("example.rs")).unwrap();
}
```

**Why it's useful:** You can swap implementations without changing calling code. In this project, you can switch between regex and tree-sitter scanners seamlessly.

**The `Send + Sync` bounds:** These are marker traits that indicate a type is safe to send between threads (`Send`) and safe to share references between threads (`Sync`). Required for parallel processing with Rayon.

#### Builder Pattern

**What it is:** A pattern for constructing complex objects with many optional parameters.

**Example:**

```rust
let discovery = FileDiscovery::new()
    .with_max_file_size(1024 * 1024)
    .with_extensions(vec!["rs", "py"])
    .with_respect_gitignore(true);
```

**Why it's useful:** Clearer than a constructor with 10 parameters, half of which are `None`. Each `with_*` method returns `self`, allowing chaining.

**Implementation pattern:**

```rust
impl FileDiscovery {
    pub fn new() -> Self {
        Self {
            max_file_size: None,
            extensions: None,
            respect_gitignore: true,
            follow_symlinks: false,
        }
    }

    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    pub fn with_extensions(mut self, exts: Vec<String>) -> Self {
        self.extensions = Some(exts);
        self
    }
}
```

#### Error Propagation with `?`

**What it is:** The `?` operator is shorthand for "if this is an error, return it; otherwise, unwrap the value."

**Example:**

```rust
fn process_file(path: &Path) -> Result<Vec<TodoItem>, TodoError> {
    let contents = std::fs::read_to_string(path)?;  // Return error if read fails
    let items = parse_todos(&contents)?;             // Return error if parse fails
    Ok(items)
}
```

**Equivalent without `?`:**

```rust
fn process_file(path: &Path) -> Result<Vec<TodoItem>, TodoError> {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Err(TodoError::IoError(e)),
    };
    let items = match parse_todos(&contents) {
        Ok(i) => i,
        Err(e) => return Err(e),
    };
    Ok(items)
}
```

**Why it's useful:** Much cleaner than try/catch or manual error checking. Errors propagate automatically, and the compiler enforces that you handle them.

#### Feature Flags with `#[cfg]`

**What it is:** Conditional compilation based on enabled features. Like `#ifdef` in C/C++.

**Example:**

```rust
#[cfg(feature = "precise")]
pub mod treesitter;

#[cfg(feature = "precise")]
use crate::scanner::treesitter::TreeSitterScanner;

pub fn create_scanner(precise: bool) -> Box<dyn FileScanner> {
    #[cfg(feature = "precise")]
    if precise {
        return Box::new(TreeSitterScanner::new());
    }

    Box::new(RegexScanner::new())
}
```

**Why it's useful:** You can ship a lightweight binary by default, with opt-in features for power users. The tree-sitter code isn't even compiled unless the feature is enabled.

#### Parallel Iteration with Rayon

**What it is:** Rayon provides data parallelism via `par_iter()`. It's like Java's parallel streams.

**Example:**

```rust
use rayon::prelude::*;

let results: Vec<_> = files
    .par_iter()  // Parallel iterator
    .map(|file| scan_file(file))
    .collect();
```

**How it works:**
- Rayon splits the iterator into chunks
- Each chunk is processed by a worker thread
- Results are collected in order
- No need to manage threads manually

**Why it's useful:** Scanning 1000 files sequentially takes 10 seconds. With 4 cores, parallel scanning takes ~2.5 seconds. Rayon makes this trivial.

---

## 5. Running Tests

### Test Types

Rust has two test types:

**1. Unit tests:** Inside each source file in a `#[cfg(test)]` module.

Example in `src/model.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_from_str() {
        assert_eq!(Priority::from_str_tag("high"), Some(Priority::High));
        assert_eq!(Priority::from_str_tag("p1"), Some(Priority::High));
        assert_eq!(Priority::from_str_tag("invalid"), None);
    }
}
```

**What `#[cfg(test)]` does:** This module is only compiled when running `cargo test`. It's stripped from release builds.

**2. Integration tests:** In the `tests/` directory. These test the compiled binary as a black box.

Example in `tests/cli_test.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("todos").unwrap();
    cmd.arg("list")
        .arg("--path=tests/fixtures")
        .assert()
        .success()
        .stdout(predicate::str::contains("TODO"));
}
```

**What `assert_cmd` does:** Runs the compiled binary with arguments, captures stdout/stderr, and makes assertions about output and exit code.

### Running Tests

**Run all tests:**

```bash
cargo test
```

Output shows:
- Number of tests run
- Pass/fail status for each test
- Summary at the end

**Run tests with tree-sitter feature:**

```bash
cargo test --features precise
```

**Run a specific test:**

```bash
cargo test test_priority_from_str
```

**Run tests in a specific module:**

```bash
cargo test scanner::regex
```

This runs all tests in the `scanner::regex` module (i.e., `src/scanner/regex.rs`).

**Run only unit tests (skip integration tests):**

```bash
cargo test --lib
```

**Run only integration tests:**

```bash
cargo test --test cli_test
```

**Show `println!` output during tests:**

By default, Rust captures and hides test output. To see it:

```bash
cargo test -- --nocapture
```

**Run tests in parallel (default) or serially:**

```bash
# Parallel (default, fastest)
cargo test

# Serial (one at a time, useful for debugging)
cargo test -- --test-threads=1
```

### Understanding Test Output

Example output:

```
running 173 tests
test cache::tests::test_cache_hit ... ok
test cache::tests::test_cache_miss ... ok
test filter::tests::test_filter_by_tag ... ok
test filter::tests::test_filter_by_author ... ok
test model::tests::test_priority_from_str ... ok
test scanner::regex::tests::test_scan_rust_file ... ok
test scanner::regex::tests::test_scan_python_file ... ok
...
test result: ok. 173 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.31s
```

**Interpretation:**
- `173 tests`: Total tests executed
- `ok`: Test passed
- `FAILED`: Test failed (shows assertion error and backtrace)
- `ignored`: Test marked with `#[ignore]` (skipped unless you run `cargo test -- --ignored`)
- `measured`: Benchmark tests (not used in this project)
- `filtered out`: Tests excluded by your filter (e.g., `cargo test specific_test` filters out others)

**If a test fails:**

```
test scanner::regex::tests::test_scan_rust_file ... FAILED

failures:

---- scanner::regex::tests::test_scan_rust_file stdout ----
thread 'scanner::regex::tests::test_scan_rust_file' panicked at src/scanner/regex.rs:234:9:
assertion failed: `(left == right)`
  left: `2`,
 right: `3`
note: run with `RUST_BACKTRACE=1` for a backtrace
```

**Debugging a failed test:**

1. Run with backtrace to see the call stack:
   ```bash
   RUST_BACKTRACE=1 cargo test test_scan_rust_file
   ```

2. Add `println!` statements to the test and run with `--nocapture`:
   ```rust
   #[test]
   fn test_scan_rust_file() {
       let result = scan_file("test.rs");
       println!("Result: {:?}", result);  // Debug output
       assert_eq!(result.len(), 3);
   }
   ```
   ```bash
   cargo test test_scan_rust_file -- --nocapture
   ```

3. Use a debugger (VS Code with rust-analyzer has excellent debugging support).

### Writing New Tests

#### Adding a Unit Test

1. Find the module you want to test (e.g., `src/filter.rs`)
2. Add or extend the `#[cfg(test)] mod tests` block at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;  // Import everything from parent module

    #[test]
    fn test_filter_by_tag() {
        let items = vec![
            TodoItem { tag: TodoTag::Todo, ..Default::default() },
            TodoItem { tag: TodoTag::Fixme, ..Default::default() },
        ];
        let criteria = FilterCriteria {
            tags: Some(vec![TodoTag::Todo]),
            ..Default::default()
        };
        let filtered = apply_filters(items, &criteria);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].tag, TodoTag::Todo);
    }
}
```

3. Run the test:
   ```bash
   cargo test test_filter_by_tag
   ```

#### Adding an Integration Test

1. Open `tests/cli_test.rs`
2. Add a new test function:

```rust
#[test]
fn test_json_output() {
    let mut cmd = Command::cargo_bin("todos").unwrap();
    cmd.arg("list")
        .arg("--path=tests/fixtures")
        .arg("--format=json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"tag\":"));
}
```

3. Run the test:
   ```bash
   cargo test test_json_output
   ```

#### Example: Adding a Test for a New Output Format

Suppose you added a new output format called `xml`. Here's how to test it:

**Unit test in `src/output/xml.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{TodoItem, TodoTag, ScanStats};

    #[test]
    fn test_xml_formatter() {
        let items = vec![
            TodoItem {
                tag: TodoTag::Todo,
                message: "Test message".to_string(),
                file: PathBuf::from("test.rs"),
                line: 10,
                ..Default::default()
            },
        ];
        let stats = ScanStats::new();
        let formatter = XmlFormatter::new();
        let output = formatter.format(&items, &stats).unwrap();

        assert!(output.contains("<todos>"));
        assert!(output.contains("<item>"));
        assert!(output.contains("<tag>TODO</tag>"));
        assert!(output.contains("<message>Test message</message>"));
        assert!(output.contains("</todos>"));
    }
}
```

**Integration test in `tests/cli_test.rs`:**

```rust
#[test]
fn test_xml_output_format() {
    let mut cmd = Command::cargo_bin("todos").unwrap();
    cmd.arg("list")
        .arg("--path=tests/fixtures")
        .arg("--format=xml")
        .assert()
        .success()
        .stdout(predicate::str::contains("<todos>"))
        .stdout(predicate::str::contains("</todos>"));
}
```

---

## 6. Adding New Features (Step-by-Step Guides)

### Adding a New Language

Let's add support for Elixir (`.ex`, `.exs` files).

**Step 1: Open `src/scanner/languages.rs`**

**Step 2: Add a new `Language` struct**

Find the `LanguageDatabase::new()` function and add:

```rust
Language {
    name: "Elixir",
    extensions: &["ex", "exs"],
    line_comment: Some("#"),
    block_comment: None,  // Elixir doesn't have block comments
},
```

**Step 3: Add a test fixture file**

Create `tests/fixtures/sample.ex`:

```elixir
defmodule Sample do
  # TODO: implement caching
  def hello do
    "world"  # FIXME: this is a hack
  end
end
```

**Step 4: Add a unit test**

In `src/scanner/languages.rs`, add to the `tests` module:

```rust
#[test]
fn test_elixir_language() {
    let db = LanguageDatabase::new();
    let lang = db.get_language_by_extension("ex").unwrap();
    assert_eq!(lang.name, "Elixir");
    assert_eq!(lang.line_comment, Some("#"));
    assert_eq!(lang.block_comment, None);
}
```

**Step 5: Add an integration test**

In `tests/cli_test.rs`:

```rust
#[test]
fn test_scan_elixir_file() {
    let mut cmd = Command::cargo_bin("todos").unwrap();
    cmd.arg("list")
        .arg("--path=tests/fixtures/sample.ex")
        .assert()
        .success()
        .stdout(predicate::str::contains("TODO"))
        .stdout(predicate::str::contains("FIXME"));
}
```

**Step 6: Run tests to verify**

```bash
cargo test test_elixir
cargo test test_scan_elixir_file
```

**Done!** Elixir is now supported.

### Adding a New Output Format

Let's add an XML output format.

**Step 1: Create `src/output/xml.rs`**

```rust
use crate::error::TodoError;
use crate::model::{ScanStats, TodoItem};
use crate::output::OutputFormatter;

pub struct XmlFormatter;

impl XmlFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for XmlFormatter {
    fn format(&self, items: &[TodoItem], stats: &ScanStats) -> Result<String, TodoError> {
        let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        output.push_str("<todos>\n");
        output.push_str(&format!("  <stats>\n"));
        output.push_str(&format!("    <total>{}</total>\n", stats.total_todos));
        output.push_str(&format!("    <files_scanned>{}</files_scanned>\n", stats.files_scanned));
        output.push_str(&format!("  </stats>\n"));
        output.push_str("  <items>\n");

        for item in items {
            output.push_str("    <item>\n");
            output.push_str(&format!("      <tag>{}</tag>\n", item.tag));
            output.push_str(&format!("      <message>{}</message>\n", escape_xml(&item.message)));
            output.push_str(&format!("      <file>{}</file>\n", escape_xml(&item.file.display().to_string())));
            output.push_str(&format!("      <line>{}</line>\n", item.line));
            if let Some(author) = &item.author {
                output.push_str(&format!("      <author>{}</author>\n", escape_xml(author)));
            }
            output.push_str("    </item>\n");
        }

        output.push_str("  </items>\n");
        output.push_str("</todos>\n");
        Ok(output)
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_xml_formatter() {
        let items = vec![
            TodoItem {
                tag: crate::model::TodoTag::Todo,
                message: "Test message".to_string(),
                file: PathBuf::from("test.rs"),
                line: 10,
                column: 5,
                author: Some("alice".to_string()),
                issue: None,
                priority: None,
                context_line: "// TODO: Test message".to_string(),
                git_author: None,
                git_date: None,
            },
        ];
        let mut stats = ScanStats::new();
        stats.total_todos = 1;
        stats.files_scanned = 1;

        let formatter = XmlFormatter::new();
        let output = formatter.format(&items, &stats).unwrap();

        assert!(output.contains("<todos>"));
        assert!(output.contains("<tag>TODO</tag>"));
        assert!(output.contains("<message>Test message</message>"));
        assert!(output.contains("<author>alice</author>"));
        assert!(output.contains("</todos>"));
    }
}
```

**Step 2: Add `pub mod xml` to `src/output/mod.rs`**

```rust
pub mod xml;
```

**Step 3: Add `Xml` variant to `OutputFormat` enum**

If there's an enum in `src/output/mod.rs` (there isn't currently, but if you refactor):

```rust
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Markdown,
    Sarif,
    GithubActions,
    Xml,  // Add this
}
```

**Step 4: Add dispatch in `format_output()`**

In `src/output/mod.rs`, modify the `format_output()` function:

```rust
pub fn format_output(format: &str, items: &[TodoItem], stats: &ScanStats) -> Result<String, TodoError> {
    match format {
        "text" => text::TextFormatter::new().format(items, stats),
        "json" => json::JsonFormatter::new().format(items, stats),
        "csv" => csv::CsvFormatter::new().format(items, stats),
        "markdown" => markdown::MarkdownFormatter::new().format(items, stats),
        "sarif" => sarif::SarifFormatter::new().format(items, stats),
        "github-actions" => github_actions::GithubActionsFormatter::new().format(items, stats),
        "xml" => xml::XmlFormatter::new().format(items, stats),  // Add this
        _ => Err(TodoError::UnsupportedFormat(format.to_string())),
    }
}
```

**Step 5: Add integration test**

In `tests/cli_test.rs`:

```rust
#[test]
fn test_xml_output() {
    let mut cmd = Command::cargo_bin("todos").unwrap();
    cmd.arg("list")
        .arg("--path=tests/fixtures")
        .arg("--format=xml")
        .assert()
        .success()
        .stdout(predicate::str::contains("<?xml"))
        .stdout(predicate::str::contains("<todos>"))
        .stdout(predicate::str::contains("</todos>"));
}
```

**Step 6: Run tests**

```bash
cargo test test_xml
```

**Done!** Users can now use `todos list --format=xml`.

### Adding a New CLI Command

Let's add a `todos count` command that shows just the count of TODOs (simpler than `stats`).

**Step 1: Add variant to `Commands` enum in `src/cli.rs`**

```rust
#[derive(Subcommand)]
pub enum Commands {
    List,
    Scan,
    Init,
    Stats,
    Count,  // Add this
    Diff { /* ... */ },
    Check { /* ... */ },
    Blame { /* ... */ },
}
```

**Step 2: Add handler function in `src/main.rs`**

```rust
fn run_count(cli: &Cli, config: &Config) -> anyhow::Result<()> {
    // Discover files
    let discovery = FileDiscovery::new();
    let files = discovery.discover(&Path::new(&cli.path))?;

    // Scan files
    let scanner = Arc::new(RegexScanner::new());
    let orchestrator = ScanOrchestrator::new(scanner);
    let (items, _stats) = orchestrator.scan_files(&files)?;

    // Apply filters
    let criteria = FilterCriteria::from_cli(cli);
    let filtered = apply_filters(items, &criteria);

    // Print count
    println!("{}", filtered.len());

    Ok(())
}
```

**Step 3: Add match arm in main's command dispatch**

In `main()`:

```rust
match cli.command {
    Some(Commands::List) | Some(Commands::Scan) => run_list(&cli, &config)?,
    Some(Commands::Init) => run_init()?,
    Some(Commands::Stats) => run_stats(&cli, &config)?,
    Some(Commands::Count) => run_count(&cli, &config)?,  // Add this
    Some(Commands::Diff { range, staged }) => run_diff(&cli, &config, &range, staged)?,
    Some(Commands::Check { .. }) => run_check(&cli, &config)?,
    Some(Commands::Blame { .. }) => run_blame(&cli, &config)?,
    None => run_list(&cli, &config)?,
}
```

**Step 4: Add integration test in `tests/cli_test.rs`**

```rust
#[test]
fn test_count_command() {
    let mut cmd = Command::cargo_bin("todos").unwrap();
    cmd.arg("count")
        .arg("--path=tests/fixtures")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^\d+\n$").unwrap());  // Just a number
}
```

**Step 5: Run tests**

```bash
cargo test test_count_command
```

**Done!** Users can now run `todos count --path ./src`.

### Adding a New Policy Rule

Let's add a rule that requires TODOs to have a priority tag.

**Step 1: Add field to `PolicyConfig` in `src/policy.rs`**

```rust
pub struct PolicyConfig {
    pub max_todos: Option<usize>,
    pub require_issue: Option<Vec<String>>,
    pub deny: Option<Vec<String>>,
    pub require_priority: bool,  // Add this
}
```

**Step 2: Add check logic in `check_policies()`**

In `src/policy.rs`:

```rust
pub fn check_policies(items: &[TodoItem], config: &PolicyConfig) -> Vec<PolicyViolation> {
    let mut violations = Vec::new();

    // Existing checks...

    // New: Check for required priority
    if config.require_priority {
        for item in items {
            if item.priority.is_none() {
                violations.push(PolicyViolation {
                    rule: "require_priority".to_string(),
                    message: "TODO is missing a priority tag (e.g., p:high, p:low)".to_string(),
                    item: Some(item.clone()),
                });
            }
        }
    }

    violations
}
```

**Step 3: Add CLI flag in `src/cli.rs` Check command**

```rust
Check {
    #[arg(long)]
    max_todos: Option<usize>,
    #[arg(long)]
    require_issue: Option<String>,
    #[arg(long)]
    deny: Option<String>,
    #[arg(long)]
    require_priority: bool,  // Add this
    #[arg(long)]
    diff_only: bool,
    #[arg(long)]
    staged_only: bool,
},
```

**Step 4: Wire in `run_check()` in `src/main.rs`**

```rust
fn run_check(cli: &Cli, config: &Config) -> anyhow::Result<()> {
    // ... existing code to scan and filter ...

    let policy_config = PolicyConfig {
        max_todos: max_todos_arg.or(config.policy.max_todos),
        require_issue: require_issue_arg.or(config.policy.require_issue.clone()),
        deny: deny_arg.or(config.policy.deny.clone()),
        require_priority: cli.command.require_priority,  // Add this
    };

    let violations = check_policies(&filtered, &policy_config);

    // ... print violations, exit with code 1 if any ...
}
```

**Step 5: Add unit tests in `src/policy.rs`**

```rust
#[test]
fn test_require_priority() {
    let items = vec![
        TodoItem {
            tag: TodoTag::Todo,
            priority: Some(Priority::High),
            ..Default::default()
        },
        TodoItem {
            tag: TodoTag::Todo,
            priority: None,  // Missing priority
            ..Default::default()
        },
    ];

    let config = PolicyConfig {
        require_priority: true,
        ..Default::default()
    };

    let violations = check_policies(&items, &config);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "require_priority");
}
```

**Step 6: Add integration test in `tests/cli_test.rs`**

```rust
#[test]
fn test_check_require_priority() {
    let mut cmd = Command::cargo_bin("todos").unwrap();
    cmd.arg("check")
        .arg("--require-priority")
        .arg("--path=tests/fixtures")
        .assert()
        .failure();  // Should fail because some TODOs lack priority
}
```

**Done!** Users can now enforce priority tags with `todos check --require-priority`.

---

## 7. Common Tasks

### Debugging

**Using `println!` debugging:**

Add `println!` or `dbg!` statements in your code:

```rust
fn scan_file(path: &Path) -> Result<Vec<TodoItem>, TodoError> {
    println!("Scanning file: {:?}", path);  // Debug output
    let contents = std::fs::read_to_string(path)?;
    dbg!(&contents);  // Prints variable name and value
    // ...
}
```

Run without capturing output:

```bash
cargo run -- list --path ./src
```

**Using environment variables for conditional logging:**

If you add the `env_logger` crate:

```rust
env_logger::init();
log::debug!("Scanning file: {:?}", path);
```

Run with debug logging:

```bash
RUST_LOG=debug cargo run -- list --path ./src
```

**Using VS Code debugger:**

1. Open the file you want to debug
2. Set a breakpoint by clicking in the gutter (left of line numbers)
3. Click the "Run and Debug" icon in the sidebar
4. Click "create a launch.json file" if you don't have one
5. Select "Rust" as the environment
6. Edit `launch.json` to add your program arguments:
   ```json
   {
     "version": "0.2.0",
     "configurations": [
       {
         "type": "lldb",
         "request": "launch",
         "name": "Debug todos",
         "cargo": {
           "args": ["build", "--bin=todos"]
         },
         "args": ["list", "--path", "./src"],
         "cwd": "${workspaceFolder}"
       }
     ]
   }
   ```
7. Press F5 to start debugging
8. Use F10 (step over), F11 (step into), Shift+F11 (step out)
9. Hover over variables to see their values
10. Use the Debug Console to evaluate expressions

**Test debugging:**

```bash
# See test output
cargo test -- --nocapture

# Run a single test with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run in the debugger (VS Code)
# Set a breakpoint in your test, then use the "Debug" button that rust-analyzer adds above each test
```

### Formatting Code

**Auto-format all code:**

```bash
cargo fmt
```

This runs `rustfmt` on all `.rs` files in your project. It enforces consistent style (indentation, spacing, line breaks, etc.).

**Check formatting without modifying files:**

```bash
cargo fmt -- --check
```

This is used in CI to ensure all code is formatted. It exits with code 1 if any file needs formatting.

**Configure `rustfmt`:**

Create a `rustfmt.toml` or `.rustfmt.toml` in the project root:

```toml
max_width = 100
tab_spaces = 4
edition = "2021"
```

See https://rust-lang.github.io/rustfmt/ for all options.

### Linting

**Run Clippy (Rust's linter):**

```bash
cargo clippy
```

Clippy finds common mistakes and suggests idiomatic Rust patterns.

Example warnings:
- "This expression can be simplified"
- "You don't need to clone here"
- "This loop can be replaced with an iterator"

**Treat warnings as errors (for CI):**

```bash
cargo clippy -- -D warnings
```

This causes the build to fail if Clippy finds any issues.

**Fix automatically:**

Some Clippy warnings can be auto-fixed:

```bash
cargo clippy --fix
```

**Allow specific lints:**

If you disagree with a Clippy warning, you can allow it:

```rust
#[allow(clippy::too_many_arguments)]
fn my_function(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) {
    // ...
}
```

Or at the module level:

```rust
#![allow(clippy::module_inception)]
```

### Dependency Management

**Update dependencies:**

Update `Cargo.lock` to the latest compatible versions (within semver constraints in `Cargo.toml`):

```bash
cargo update
```

**Add a new dependency:**

```bash
cargo add <crate-name>
```

Example:
```bash
cargo add serde --features derive
```

This adds to `Cargo.toml`:
```toml
serde = { version = "1", features = ["derive"] }
```

**Show dependency tree:**

```bash
cargo tree
```

Example output:
```
todo-tracker v0.1.0
├── clap v4.4.18
│   ├── clap_builder v4.4.18
│   │   ├── anstyle v1.0.4
│   │   ├── clap_lex v0.6.0
│   │   └── strsim v0.10.0
│   └── clap_derive v4.4.7
├── regex v1.10.2
│   ├── aho-corasick v1.1.2
│   └── regex-syntax v0.8.2
└── serde v1.0.193
```

**Check for outdated dependencies:**

Install `cargo-outdated`:
```bash
cargo install cargo-outdated
```

Run:
```bash
cargo outdated
```

This shows which dependencies have newer versions available.

**Audit dependencies for security vulnerabilities:**

Install `cargo-audit`:
```bash
cargo install cargo-audit
```

Run:
```bash
cargo audit
```

This checks for known security issues in your dependencies.

---

## 8. CI/CD Pipeline

The project uses GitHub Actions for CI/CD. Two workflows:

### `.github/workflows/ci.yml` - Continuous Integration

**Triggers:**
- Every push to any branch
- Every pull request

**Jobs:**

1. **test** (Linux, macOS, Windows):
   - Install Rust
   - Run `cargo test`
   - Run `cargo test --features precise`
   - Runs on all three platforms to catch platform-specific bugs

2. **lint**:
   - Run `cargo fmt -- --check` (ensure code is formatted)
   - Run `cargo clippy -- -D warnings` (no linting warnings allowed)

3. **coverage** (optional, if enabled):
   - Generate code coverage report using `tarpaulin`
   - Upload to Codecov

**How to read the workflow file:**

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo test --features precise

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
```

**What each part does:**
- `on: [push, pull_request]`: When to run
- `matrix.os`: Run the job on all three OSes
- `actions/checkout@v3`: Clone the repo
- `dtolnay/rust-toolchain@stable`: Install the latest stable Rust
- `run: cargo test`: Execute shell commands

### `.github/workflows/release.yml` - Release Automation

**Triggers:**
- Push a git tag starting with `v` (e.g., `v0.1.0`, `v1.2.3`)

**Jobs:**

1. **build** (Linux, macOS, Windows):
   - Build release binary: `cargo build --release`
   - Strip debug symbols (Linux/macOS): `strip target/release/todos`
   - Package as `.tar.gz` (Linux/macOS) or `.zip` (Windows)
   - Upload as GitHub release artifact

2. **docker** (optional, if enabled):
   - Build Docker image
   - Push to Docker Hub or GitHub Container Registry

**How to trigger a release:**

1. Update version in `Cargo.toml`:
   ```toml
   version = "0.2.0"
   ```

2. Commit and tag:
   ```bash
   git add Cargo.toml
   git commit -m "Release v0.2.0"
   git tag v0.2.0
   git push origin main --tags
   ```

3. GitHub Actions builds binaries for all platforms and creates a GitHub Release with:
   - `todos-v0.2.0-x86_64-unknown-linux-gnu.tar.gz`
   - `todos-v0.2.0-x86_64-apple-darwin.tar.gz`
   - `todos-v0.2.0-x86_64-pc-windows-msvc.zip`

Users can download and run the binary without installing Rust.

---

## 9. Docker

### Dockerfile Explained

The `Dockerfile` uses a multi-stage build to create a minimal Alpine-based image.

```dockerfile
# Stage 1: Build stage
FROM rust:1.75-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Stage 2: Runtime stage
FROM alpine:latest
RUN apk add --no-cache git
COPY --from=builder /app/target/release/todos /usr/local/bin/todos
ENTRYPOINT ["todos"]
CMD ["list", "--path", "/app"]
```

**What each part does:**

- **Stage 1 (builder):**
  - `FROM rust:1.75-alpine`: Start with Rust on Alpine Linux (small base image)
  - `apk add musl-dev`: Install musl libc (required for static linking on Alpine)
  - `COPY Cargo.* ./`: Copy dependency files first (Docker caches this layer)
  - `COPY src ./src`: Copy source code
  - `cargo build --release`: Build optimized binary

- **Stage 2 (runtime):**
  - `FROM alpine:latest`: Fresh Alpine image (no Rust, much smaller)
  - `apk add git`: Install git (needed for blame/diff features)
  - `COPY --from=builder`: Copy only the binary from the build stage
  - `ENTRYPOINT`: The binary to run
  - `CMD`: Default arguments (can be overridden)

**Why multi-stage builds?** The builder stage is huge (~1 GB with Rust toolchain). The runtime stage is tiny (~50 MB) because it only includes the binary and git.

### Building the Docker Image

```bash
docker build -t todos .
```

This takes several minutes the first time (compiles all dependencies). Subsequent builds are faster due to Docker layer caching.

### Running the Docker Container

**Scan files in the container:**

```bash
docker run todos list --path /app
```

**Scan files from your host machine:**

Mount a volume to make your files accessible inside the container:

```bash
docker run -v $(pwd):/code todos list --path /code
```

On Windows (PowerShell):
```powershell
docker run -v ${PWD}:/code todos list --path /code
```

**Run with custom arguments:**

```bash
docker run todos list --path /code --format=json --tag=FIXME
```

**Access the shell inside the container:**

```bash
docker run -it --entrypoint /bin/sh todos
# Now you're inside the container
/app # todos --help
/app # exit
```

### Publishing the Docker Image

**Tag the image:**

```bash
docker tag todos myusername/todos:latest
docker tag todos myusername/todos:v0.1.0
```

**Push to Docker Hub:**

```bash
docker login
docker push myusername/todos:latest
docker push myusername/todos:v0.1.0
```

**Pull and run from Docker Hub:**

```bash
docker pull myusername/todos:latest
docker run myusername/todos list --path /app
```

---

## 10. Troubleshooting

### Common Cargo Errors

#### "unresolved import"

**Error:**
```
error[E0432]: unresolved import `crate::scanner::treesitter`
  --> src/scanner/mod.rs:5:9
   |
5  | pub use treesitter::TreeSitterScanner;
   |         ^^^^^^^^^^ maybe a missing crate `treesitter`?
```

**Cause:** You referenced a module but didn't declare it with `mod` or `pub mod`.

**Solution:** Add `pub mod treesitter;` to `src/scanner/mod.rs`:
```rust
pub mod regex;
pub mod treesitter;  // Add this
```

#### "trait bound not satisfied"

**Error:**
```
error[E0277]: the trait bound `MyStruct: Clone` is not satisfied
  --> src/main.rs:10:5
   |
10 |     my_function(my_struct);
   |     ^^^^^^^^^^^ the trait `Clone` is not implemented for `MyStruct`
```

**Cause:** A function requires `Clone` but your type doesn't implement it.

**Solution:** Add `#[derive(Clone)]`:
```rust
#[derive(Clone)]  // Add this
pub struct MyStruct {
    // ...
}
```

#### "borrow checker" errors

**Error:**
```
error[E0502]: cannot borrow `x` as mutable because it is also borrowed as immutable
  --> src/main.rs:5:5
   |
4  |     let y = &x;
   |             -- immutable borrow occurs here
5  |     x.push(1);
   |     ^^^^^^^^^ mutable borrow occurs here
6  |     println!("{}", y);
   |                    - immutable borrow later used here
```

**Cause:** You tried to mutate `x` while an immutable reference `y` exists. Rust enforces "aliasing XOR mutation"—you can have many immutable references OR one mutable reference, but not both.

**Solution:** Ensure references don't overlap:
```rust
let y = &x;
println!("{}", y);  // Use y
// Now y is out of scope
x.push(1);  // Now we can mutate x
```

Or clone:
```rust
let y = x.clone();  // y owns its own data
x.push(1);  // No conflict
println!("{}", y);
```

**Understanding the borrow checker:**
- Rust enforces memory safety at compile time
- No null pointers, no use-after-free, no data races
- The trade-off: you must satisfy the borrow checker's rules
- This is the biggest learning curve for new Rust developers
- Resources:
  - The Rust Book: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
  - Rust by Example: https://doc.rust-lang.org/rust-by-example/scope/borrow.html

#### "linking with `link.exe` failed" (Windows)

**Error:**
```
error: linking with `link.exe` failed: exit code: 1181
```

**Cause:** Missing Visual Studio Build Tools (Windows-specific).

**Solution:**

1. Download "Visual Studio Build Tools" from https://visualstudio.microsoft.com/downloads/
2. Run the installer
3. Select "Desktop development with C++"
4. Install
5. Restart your terminal
6. Retry `cargo build`

Alternatively, install Visual Studio (full IDE) which includes the build tools.

### Platform-Specific Issues

#### macOS: "xcrun: error: invalid active developer path"

**Cause:** Xcode Command Line Tools not installed or outdated.

**Solution:**
```bash
xcode-select --install
```

#### Linux: "cannot find -lpthread" or "cannot find -ldl"

**Cause:** Missing development libraries.

**Solution (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install build-essential
```

**Solution (Fedora/RHEL):**
```bash
sudo dnf groupinstall "Development Tools"
```

#### Windows: "failed to run custom build command for `openssl-sys`"

**Cause:** Some crates (like `openssl-sys`) require C dependencies.

**Solution:** Use the `-vendored` feature to compile from source:

In `Cargo.toml`:
```toml
openssl = { version = "0.10", features = ["vendored"] }
```

Or use a pure-Rust alternative (e.g., `rustls` instead of `openssl`).

### Performance Issues

#### "Cargo is very slow to compile"

**Causes and solutions:**

1. **Debug builds compile faster than release:**
   - Use `cargo build` (debug) during development
   - Use `cargo check` (even faster—no binary produced)
   - Only use `cargo build --release` for benchmarking or production

2. **Incremental compilation (should be enabled by default):**
   - Verify in `~/.cargo/config.toml` or `.cargo/config.toml`:
     ```toml
     [build]
     incremental = true
     ```

3. **Parallel compilation:**
   - Cargo compiles crates in parallel by default
   - You can increase thread count: `CARGO_BUILD_JOBS=8 cargo build`

4. **Use a faster linker:**
   - Linux: Use `mold` (fastest) or `lld` (fast)
   - Install: `sudo apt-get install mold` or `sudo apt-get install lld`
   - Configure in `.cargo/config.toml`:
     ```toml
     [target.x86_64-unknown-linux-gnu]
     linker = "clang"
     rustflags = ["-C", "link-arg=-fuse-ld=mold"]
     ```

5. **Reduce dependencies:**
   - Fewer dependencies = faster compile times
   - Check `cargo tree` to see what you're pulling in

#### "Runtime is slow"

**Possible causes:**

1. **Running a debug build:**
   - Debug builds are 10-100x slower than release builds
   - Always benchmark with `cargo build --release`

2. **Missing parallelism:**
   - Use Rayon for parallel iteration
   - Example: `.par_iter()` instead of `.iter()`

3. **Unnecessary allocations:**
   - Reuse buffers instead of allocating new ones
   - Use `&str` instead of `String` when possible
   - Use iterators instead of collecting into `Vec`

4. **Profile your code:**
   - Install `cargo-flamegraph`: `cargo install flamegraph`
   - Run: `cargo flamegraph --bin=todos -- list --path ./large_codebase`
   - Open `flamegraph.svg` in a browser to see where time is spent

### Getting Help

**Official resources:**

- The Rust Book: https://doc.rust-lang.org/book/
- Rust by Example: https://doc.rust-lang.org/rust-by-example/
- Rust Standard Library docs: https://doc.rust-lang.org/std/
- Cargo Book: https://doc.rust-lang.org/cargo/

**Community:**

- Rust Users Forum: https://users.rust-lang.org/
- Rust Discord: https://discord.gg/rust-lang
- Rust Subreddit: https://www.reddit.com/r/rust/
- Stack Overflow: Tag your question with `rust`

**Project-specific:**

- Open an issue: https://github.com/az9713/claude-agent-teams-deepdive/issues
- Read the case study: `AGENT_TEAM_CASE_STUDY.md` (how this project was built)
- Read the architecture exploration: `exploration-architecture.md`

---

## Conclusion

You now have a comprehensive understanding of the `todo-tracker` project:

- How to set up Rust and the development environment
- How the build system (Cargo) works
- The architecture and data flow
- How to run and write tests
- How to add new features (languages, output formats, commands, policies)
- Common development tasks (formatting, linting, dependency management)
- CI/CD pipelines
- Docker containerization
- Troubleshooting common issues

**Next steps:**

1. Clone the repo and run `cargo build`
2. Run the tests: `cargo test`
3. Pick a feature to add (see GitHub issues or create your own)
4. Make your changes
5. Run tests to verify
6. Submit a pull request

Welcome to the project, and happy coding!
