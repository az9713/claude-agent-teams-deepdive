# Todo-Tracker User Guide

A comprehensive guide to using the `todos` command-line tool to track technical debt in your codebase.

---

## 1. What is todo-tracker?

**todo-tracker** is a command-line tool that finds and manages technical debt markers in your source code. It scans your codebase for special comments like `TODO`, `FIXME`, `HACK`, `BUG`, and `XXX`, then presents them in useful formats for tracking and action.

### Why Use It?

- **Track Technical Debt**: Get a complete view of all deferred work across your project
- **Enforce Team Standards**: Ensure TODOs include author names, issue references, or priority levels
- **CI/CD Integration**: Fail builds if critical issues are left unaddressed or if technical debt exceeds thresholds
- **Generate Reports**: Export to JSON, CSV, Markdown, or SARIF for documentation or integration with other tools
- **Git Integration**: See who wrote each TODO and track what changed between branches
- **Team Communication**: Share a common view of what needs attention

### What Makes It Different from `grep TODO`?

While you could use `grep -r "TODO" .`, todo-tracker offers several advantages:

- **Language-Aware Parsing**: Understands comment syntax for 10+ languages, avoiding false matches in strings or variable names
- **Metadata Extraction**: Automatically parses author names, issue references (like `#123`), and priority levels from TODO comments
- **Multiple Output Formats**: JSON for scripting, CSV for spreadsheets, SARIF for GitHub Code Scanning, and more
- **Smart Filtering**: Filter by tag type, author, file pattern, priority, or issue references
- **Performance**: Built-in caching makes repeat scans nearly instant
- **Git Integration**: Blame TODOs to see who wrote them, diff between branches to see changes
- **CI/CD Ready**: Policy checking with configurable rules and exit codes for automation

---

## 2. Installation

### From Source (Rust Required)

You'll need Rust installed on your system.

**Install Rust:**

- **Unix/Linux/macOS**:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Windows**: Download and run the installer from [rustup.rs](https://rustup.rs)

After installing Rust, restart your terminal and verify:
```bash
rustc --version
```

**Clone, Build, and Install:**

```bash
git clone https://github.com/yourusername/todo-tracker.git
cd todo-tracker
cargo install --path .
```

This compiles the tool and installs the `todos` binary to your Cargo bin directory (usually `~/.cargo/bin` or `%USERPROFILE%\.cargo\bin`).

**Verify Installation:**

```bash
todos --version
```

Expected output:
```
todos 0.1.0
```

### From Docker

If you have Docker installed, you can run todo-tracker without installing Rust:

```bash
docker build -t todos .
docker run -v $(pwd):/workspace todos list --path /workspace
```

**Windows (PowerShell):**
```powershell
docker run -v ${PWD}:/workspace todos list --path /workspace
```

This mounts your current directory into the container and scans it.

### From Pre-built Binaries (GitHub Releases)

1. Go to the [Releases page](https://github.com/yourusername/todo-tracker/releases)
2. Download the binary for your platform:
   - `todos-linux-x86_64.tar.gz` for Linux
   - `todos-macos-x86_64.tar.gz` for macOS (Intel)
   - `todos-macos-aarch64.tar.gz` for macOS (Apple Silicon)
   - `todos-windows-x86_64.zip` for Windows
3. Extract the archive
4. Add the binary to your PATH:
   - **Unix/Linux/macOS**: Move to `/usr/local/bin` or add the directory to your `$PATH`
   - **Windows**: Add the directory to your `%PATH%` environment variable

---

## 3. Basic Usage

### Your First Scan

Navigate to any code project and run:

```bash
todos list
```

**Example Output:**

```
src/main.rs
  L4    [TODO] Implement proper error handling (alice, #42, p:high)
  L15   [FIXME] Memory leak in this function (bob)
  L89   [HACK] Temporary workaround for parser bug

src/utils.rs
  L23   [BUG] Off-by-one error in edge cases (#156, p:critical)
  L45   [TODO] Add unit tests

Summary: 5 items found (2 TODO, 1 FIXME, 1 HACK, 1 BUG)
```

The output uses colors when displayed in a terminal:
- File paths appear in **cyan/blue**
- Line numbers appear in **green**
- Tags are color-coded: `TODO` (cyan), `FIXME` (yellow), `HACK` (magenta), `BUG` (red), `XXX` (bright red)
- Messages appear in default text color
- Metadata (author, issue, priority) appears in **gray/dim**

### Scanning a Specific Directory

By default, `todos list` scans the current directory. To scan a different path:

```bash
todos list --path ./src
todos list --path /home/alice/projects/myapp
```

**Windows:**
```cmd
todos list --path C:\Users\alice\projects\myapp
```

### Understanding the Output

Let's break down a single TODO item:

```
src/main.rs
  L4    [TODO] Implement proper error handling (alice, #42, p:high)
```

- **`src/main.rs`**: The file containing the TODO
- **`L4`**: Line number (line 4)
- **`[TODO]`**: The tag type (TODO, FIXME, HACK, BUG, or XXX)
- **`Implement proper error handling`**: The message describing what needs to be done
- **`(alice, #42, p:high)`**: Metadata extracted from the comment:
  - `alice`: Author who should handle this
  - `#42`: Issue tracker reference (GitHub issue, JIRA ticket, etc.)
  - `p:high`: Priority level (low, medium, high, or critical)

**Summary Footer:**
```
Summary: 5 items found (2 TODO, 1 FIXME, 1 HACK, 1 BUG)
```

Shows the total count and breakdown by tag type.

---

## 4. Output Formats

### Text (default)

The human-readable format shown above.

```bash
todos list --format=text
```

### JSON (for scripting/piping)

Output structured JSON for processing with tools like `jq`, Python scripts, or other automation:

```bash
todos list --format=json
```

**Example Output:**

```json
{
  "items": [
    {
      "file": "src/main.rs",
      "line": 4,
      "tag": "TODO",
      "message": "Implement proper error handling",
      "author": "alice",
      "issue": "42",
      "priority": "high"
    },
    {
      "file": "src/main.rs",
      "line": 15,
      "tag": "FIXME",
      "message": "Memory leak in this function",
      "author": "bob",
      "issue": null,
      "priority": null
    }
  ],
  "summary": {
    "total": 2,
    "by_tag": {
      "TODO": 1,
      "FIXME": 1
    }
  }
}
```

**Piping to jq for Filtering:**

```bash
# Find all FIXME items
todos list --format=json | jq '.items[] | select(.tag == "FIXME")'

# Find items without issue references
todos list --format=json | jq '.items[] | select(.issue == null)'

# Count items by author
todos list --format=json | jq '.items | group_by(.author) | map({author: .[0].author, count: length})'
```

### CSV (for spreadsheets)

Export to CSV for viewing in Excel, Google Sheets, or other spreadsheet software:

```bash
todos list --format=csv > todos.csv
```

**Example Output:**

```csv
file,line,tag,message,author,issue,priority
src/main.rs,4,TODO,Implement proper error handling,alice,42,high
src/main.rs,15,FIXME,Memory leak in this function,bob,,
src/utils.rs,23,BUG,Off-by-one error in edge cases,,156,critical
```

**Opening in Spreadsheets:**
- **Excel**: File > Open > Select `todos.csv`
- **Google Sheets**: File > Import > Upload `todos.csv`
- **LibreOffice Calc**: File > Open > Select `todos.csv`

### Markdown (for documentation)

Generate a formatted Markdown report for README files or documentation:

```bash
todos list --format=markdown > TODO_REPORT.md
```

**Example Output:**

```markdown
# TODO Report

## src/main.rs

- **L4** `[TODO]` Implement proper error handling *(alice, #42, p:high)*
- **L15** `[FIXME]` Memory leak in this function *(bob)*

## src/utils.rs

- **L23** `[BUG]` Off-by-one error in edge cases *(#156, p:critical)*

---

**Summary:** 3 items found (1 TODO, 1 FIXME, 1 BUG)
```

### Count (just the number)

When you only need the total count:

```bash
todos list --format=count
```

**Example Output:**

```
5
```

Useful in scripts:

```bash
TODO_COUNT=$(todos list --format=count)
if [ $TODO_COUNT -gt 100 ]; then
  echo "Too many TODOs! Please clean up technical debt."
  exit 1
fi
```

### SARIF (for GitHub Code Scanning)

SARIF (Static Analysis Results Interchange Format) is a standard format for static analysis tools. GitHub Code Scanning can display SARIF results as annotations in pull requests.

```bash
todos list --format=sarif > results.sarif
```

**Upload to GitHub:**

1. Generate SARIF results
2. In your GitHub Actions workflow, use the `github/codeql-action/upload-sarif` action
3. Results appear in the "Security" tab under "Code scanning alerts"

See the CI/CD Integration section for a complete example.

### GitHub Actions Annotations

Output annotations that GitHub Actions can display inline in workflow logs and pull request diffs:

```bash
todos list --format=github-actions
```

**Example Output:**

```
::warning file=src/main.rs,line=4::TODO: Implement proper error handling (alice, #42, p:high)
::error file=src/utils.rs,line=23::BUG: Off-by-one error in edge cases (#156, p:critical)
```

When run in GitHub Actions, these appear as:
- Yellow warning annotations for TODO, FIXME, HACK, XXX
- Red error annotations for BUG tags or items with `p:critical`

---

## 5. Filtering

Narrow down results using filters. All filters can be combined.

### By Tag

Show only specific types of markers:

```bash
# Show only FIXME items
todos list --tag=FIXME

# Show multiple tag types (comma-separated, no spaces)
todos list --tag=FIXME,HACK,BUG
```

**Available Tags:**
- `TODO`: General tasks to complete
- `FIXME`: Known issues that need fixing
- `HACK`: Temporary workarounds or non-ideal solutions
- `BUG`: Confirmed bugs
- `XXX`: Warnings or serious concerns

### By Author

Show only TODOs assigned to specific people:

```bash
# Single author
todos list --author=alice

# Multiple authors
todos list --author=alice,bob
```

Author names are extracted from comment metadata like `TODO(alice): message`.

### By File Pattern

Filter by file paths using glob patterns:

```bash
# Only files in src directory (recursive)
todos list --file="src/**"

# Only Rust files
todos list --file="*.rs"

# Only files in specific subdirectories
todos list --file="src/api/**" --file="src/db/**"

# Exclude test files
todos list --file="!**/*test*"
```

**Glob Pattern Syntax:**
- `*` matches any characters except `/`
- `**` matches any characters including `/` (recursive)
- `?` matches exactly one character
- `[abc]` matches any character in the set
- `!pattern` excludes matching files

### By Priority

Show only high-priority items:

```bash
# Single priority
todos list --priority=high

# Multiple priorities
todos list --priority=high,critical
```

**Priority Levels:**
- `low`
- `medium`
- `high`
- `critical`

Priority is extracted from comments like `TODO(p:high): message`.

### By Issue Reference

Show only TODOs that reference an issue tracker:

```bash
# Has any issue reference
todos list --has-issue

# Specific issue number
todos list --issue=42
todos list --issue=JIRA-123
```

Issue references are extracted from comments like `TODO(#42): message` or `TODO(JIRA-123): message`.

### Combining Filters

Filters are combined with AND logic (all conditions must match):

```bash
# High-priority FIXMEs by alice
todos list --tag=FIXME --author=alice --priority=high

# TODOs in src/ that reference issues
todos list --tag=TODO --file="src/**" --has-issue

# Critical bugs in production code (not tests)
todos list --tag=BUG --priority=critical --file="!**/*test*"
```

---

## 6. Configuration

### Creating a Config File

Generate a configuration file with default settings:

```bash
todos init
```

This creates `.todo-tracker.toml` in your current directory.

**Output:**

```
Created configuration file: .todo-tracker.toml
```

### Config File Format

The `.todo-tracker.toml` file uses TOML format. Here's the complete template with explanations:

```toml
# Default path to scan (can override with --path)
path = "."

# Tags to search for
tags = ["TODO", "FIXME", "HACK", "BUG", "XXX"]

# File extensions to scan
# todo-tracker auto-detects these, but you can limit or expand here
extensions = [".rs", ".go", ".py", ".js", ".ts", ".java", ".c", ".cpp", ".cs", ".rb"]

# Files/directories to exclude (in addition to .gitignore)
exclude = [
  "target/",
  "node_modules/",
  "*.log",
  "*.tmp"
]

# Maximum file size to scan (in bytes)
# Files larger than this are skipped
max_file_size = 1048576  # 1MB

# Enable caching for faster repeat scans
cache_enabled = true

# Cache location
cache_path = ".todo-tracker/cache.db"

# Output format (text, json, csv, markdown, count, sarif, github-actions)
format = "text"

# Color output (auto, always, never)
# - auto: colors in terminal, plain when piped
# - always: force colors
# - never: no colors
color = "auto"

# Policy rules for `todos check` command
[policy]
# Maximum number of TODOs allowed (fail build if exceeded)
max_total = 100

# Maximum per tag type
max_by_tag = { TODO = 50, FIXME = 20, HACK = 10, BUG = 5 }

# Tags that require issue references
require_issue = ["FIXME", "BUG"]

# Tags that require author names
require_author = ["FIXME"]

# Tags that are completely forbidden (fail on any occurrence)
deny = ["NOCOMMIT", "HACK"]

# Minimum priority for certain tags (e.g., all BUGs must be p:high or p:critical)
min_priority = { BUG = "high" }

# Git integration settings
[git]
# Enable git blame to find authors
blame_enabled = true

# Date format for blame output
blame_date_format = "%Y-%m-%d"

# Tree-sitter precision mode (requires building with --features precise)
[treesitter]
enabled = false
```

**Example Customizations:**

Only scan specific directories:
```toml
path = "src"
exclude = ["src/generated/", "src/vendor/"]
```

Strict policy for production code:
```toml
[policy]
max_total = 0  # No TODOs allowed
deny = ["TODO", "FIXME", "HACK"]
```

Require metadata on all TODOs:
```toml
[policy]
require_issue = ["TODO", "FIXME", "BUG"]
require_author = ["TODO", "FIXME", "BUG"]
```

### Config Resolution Order

todo-tracker searches for configuration in this order (first match wins):

1. **Explicit config** via `--config` flag:
   ```bash
   todos list --config path/to/config.toml
   ```

2. **Project config** in current directory (walks up to git root):
   ```
   ./.todo-tracker.toml
   ../.todo-tracker.toml
   ../../.todo-tracker.toml
   # ... continues up to git root or filesystem root
   ```

3. **User default config**:
   - **Unix/Linux/macOS**: `~/.config/todo-tracker/config.toml`
   - **Windows**: `%APPDATA%\todo-tracker\config.toml`

4. **Built-in defaults** if no config file is found

**Tip:** Place `.todo-tracker.toml` in your repository root and commit it to share configuration across your team.

---

## 7. Git Integration

### Blame: Who Wrote Each TODO?

Use `git blame` integration to see who authored each TODO and when:

```bash
todos blame
```

**Example Output:**

```
src/main.rs
  L4    [TODO] Implement proper error handling (alice, #42, p:high)
        Author: Bob Smith <bob@example.com>
        Date:   2024-11-15
        Commit: a1b2c3d

  L15   [FIXME] Memory leak in this function (bob)
        Author: Bob Smith <bob@example.com>
        Date:   2024-12-01
        Commit: e4f5g6h

src/utils.rs
  L23   [BUG] Off-by-one error in edge cases (#156, p:critical)
        Author: Alice Johnson <alice@example.com>
        Date:   2024-10-22
        Commit: i7j8k9l

Summary: 3 items found
```

**Sort by Date:**

```bash
# Oldest first
todos blame --sort=date

# Newest first
todos blame --sort=date --reverse
```

**Filter by Date Range:**

```bash
# TODOs added since January 1, 2024
todos blame --since=2024-01-01

# TODOs added in the last 30 days
todos blame --since="30 days ago"

# TODOs added between two dates
todos blame --since=2024-01-01 --until=2024-12-31
```

**JSON Output for Scripting:**

```bash
todos blame --format=json
```

**Example JSON:**

```json
{
  "items": [
    {
      "file": "src/main.rs",
      "line": 4,
      "tag": "TODO",
      "message": "Implement proper error handling",
      "blame": {
        "author": "Bob Smith",
        "email": "bob@example.com",
        "date": "2024-11-15",
        "commit": "a1b2c3d"
      }
    }
  ]
}
```

### Diff: What Changed Between Branches?

See what TODOs were added or removed between git references:

```bash
# Compare current branch to main
todos diff main..HEAD

# Compare two branches
todos diff main..feature-branch

# Compare staged changes
todos diff --staged

# Compare working directory (unstaged changes)
todos diff
```

**Example Output:**

```
Comparing: main..feature-branch

Added (2):
  src/api.rs
    L12   [TODO] Add rate limiting (alice, #78, p:high)

  src/db.rs
    L45   [FIXME] Connection pool exhaustion (bob)

Removed (1):
  src/main.rs
    L8    [HACK] Temporary workaround for parser bug

Modified (1):
  src/utils.rs
    L23   [BUG] Off-by-one error in edge cases (#156, p:critical)
    Was:  [BUG] Edge case bug (#156)

Summary: +2 added, -1 removed, 1 modified
```

**Use Cases:**

- **Pull Request Reviews**: See what technical debt a PR introduces
  ```bash
  todos diff main..pr-branch
  ```

- **Release Planning**: Check what TODOs accumulated since last release
  ```bash
  todos diff v1.0.0..HEAD
  ```

- **Pre-commit Hook**: Ensure no new critical TODOs
  ```bash
  todos diff --staged --tag=BUG --priority=critical
  ```

---

## 8. CI/CD Integration

### Policy Checks

The `todos check` command validates your codebase against configurable policies and exits with appropriate codes for CI/CD:

```bash
todos check
```

**Exit Codes:**
- `0`: All checks passed
- `1`: Policy violations found
- `2`: Error occurred (file not found, git failure, etc.)

**Example Output (passing):**

```
Running policy checks...

✓ Total TODOs: 23 (limit: 100)
✓ TODO count: 15 (limit: 50)
✓ FIXME count: 5 (limit: 20)
✓ HACK count: 2 (limit: 10)
✓ BUG count: 1 (limit: 5)
✓ All FIXMEs have issue references
✓ No denied tags found

All checks passed!
```

**Example Output (failing):**

```
Running policy checks...

✓ Total TODOs: 78 (limit: 100)
✗ FIXME count: 25 (limit: 20) - 5 over limit
✗ HACK count: 15 (limit: 10) - 5 over limit
✗ Found 3 FIXMEs without issue references:
    src/api.rs:12
    src/db.rs:45
    src/utils.rs:67
✗ Found denied tag: NOCOMMIT
    src/temp.rs:8

Policy check failed!
```

**Policy Options:**

```bash
# Set maximum total TODOs
todos check --max-todos=50

# Require issue references for specific tags
todos check --require-issue=FIXME,BUG

# Deny specific tags completely
todos check --deny=NOCOMMIT,HACK

# Require author names
todos check --require-author=FIXME,BUG

# Set minimum priority
todos check --min-priority=BUG:high
```

### GitHub Actions Example

Create `.github/workflows/todo-check.yml`:

```yaml
name: TODO Policy Check

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  todo-check:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for git blame/diff

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install todo-tracker
        run: |
          cargo install --git https://github.com/yourusername/todo-tracker.git

      - name: Run TODO policy check
        run: todos check --max-todos=100 --require-issue=FIXME,BUG --deny=NOCOMMIT

      - name: Generate SARIF report
        if: always()
        run: todos list --format=sarif > results.sarif

      - name: Upload SARIF to GitHub
        if: always()
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: results.sarif

      - name: Generate TODO report
        if: always()
        run: |
          todos list --format=markdown > TODO_REPORT.md
          echo "## TODO Summary" >> $GITHUB_STEP_SUMMARY
          cat TODO_REPORT.md >> $GITHUB_STEP_SUMMARY
```

This workflow:
1. Runs on every pull request and push to main
2. Installs todo-tracker
3. Checks policies (fails if violated)
4. Generates SARIF report for Code Scanning
5. Adds TODO summary to the workflow summary page

**Annotate Pull Requests:**

For inline annotations in PR diffs:

```yaml
- name: Annotate pull request
  run: todos list --format=github-actions
```

### Pre-commit Hook

Use with [pre-commit](https://pre-commit.com/) to check TODOs before committing.

Create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/yourusername/todo-tracker
    rev: v0.1.0
    hooks:
      # Check policy on all TODOs
      - id: todo-tracker-check
        name: Check TODO policy
        entry: todos check --max-todos=100
        language: system
        pass_filenames: false

      # Fail if NOCOMMIT tags are found
      - id: todo-tracker-nocommit
        name: Check for NOCOMMIT tags
        entry: todos check --deny=NOCOMMIT
        language: system
        pass_filenames: false
```

Install the hook:

```bash
pre-commit install
```

Now policy checks run automatically before every commit.

---

## 9. Statistics

Get a statistical overview of your technical debt:

```bash
todos stats
```

**Example Output:**

```
TODO Statistics
===============

Total Items: 45

By Tag:
  TODO:   25 ██████████████████████████ 55.6%
  FIXME:  12 ████████████ 26.7%
  HACK:    5 █████ 11.1%
  BUG:     2 ██ 4.4%
  XXX:     1 █ 2.2%

By Priority:
  critical:  2 ██ 4.4%
  high:      8 ████████ 17.8%
  medium:    5 █████ 11.1%
  low:       3 ███ 6.7%
  (none):   27 ██████████████████████████ 60.0%

By Author:
  alice:    15 ███████████████ 33.3%
  bob:      10 ██████████ 22.2%
  carol:     5 █████ 11.1%
  (none):   15 ███████████████ 33.3%

With Issue References: 18 (40.0%)

Top Files:
  src/main.rs:       8 items
  src/api.rs:        6 items
  src/db.rs:         5 items
  src/utils.rs:      4 items
  src/parser.rs:     3 items
```

**JSON Output:**

```bash
todos stats --format=json
```

**Example JSON:**

```json
{
  "total": 45,
  "by_tag": {
    "TODO": 25,
    "FIXME": 12,
    "HACK": 5,
    "BUG": 2,
    "XXX": 1
  },
  "by_priority": {
    "critical": 2,
    "high": 8,
    "medium": 5,
    "low": 3,
    "none": 27
  },
  "by_author": {
    "alice": 15,
    "bob": 10,
    "carol": 5,
    "none": 15
  },
  "with_issue": 18,
  "top_files": [
    {"file": "src/main.rs", "count": 8},
    {"file": "src/api.rs", "count": 6},
    {"file": "src/db.rs", "count": 5}
  ]
}
```

**Filter Statistics:**

```bash
# Stats for only FIXME items
todos stats --tag=FIXME

# Stats for high-priority items
todos stats --priority=high

# Stats for items by alice
todos stats --author=alice
```

---

## 10. Performance

### Caching

todo-tracker automatically caches scan results to make repeat scans nearly instant.

**How It Works:**

1. **First Scan**: All files are scanned, results are stored in SQLite cache
   ```
   Scanning 1,234 files...
   Found 45 TODOs
   Time: 2.3s
   ```

2. **Second Scan**: Unchanged files load from cache
   ```
   Scanning 1,234 files (1,180 from cache)...
   Found 45 TODOs
   Time: 0.2s
   ```

The cache tracks:
- File path
- Modification time
- File size
- Scan results (TODOs found)

**Cache Location:**

By default, the cache is stored at `.todo-tracker/cache.db` in your project root. You can customize this in `.todo-tracker.toml`:

```toml
cache_path = ".cache/todos.db"
```

**Disable Caching:**

```toml
cache_enabled = false
```

Or via command line:

```bash
todos list --no-cache
```

**Clear Cache:**

Force a fresh scan by clearing the cache:

```bash
todos list --clear-cache
```

This deletes the cache database and rescans all files.

### How Caching Works

**File Fingerprinting:**

Each file is fingerprinted using:
- Modification timestamp (mtime)
- File size (bytes)

When scanning:
1. Check if file exists in cache
2. Compare fingerprint (mtime + size)
3. If match, load cached results
4. If no match, rescan file and update cache

**Cache Invalidation:**

The cache is automatically invalidated when:
- File is modified (mtime changes)
- File size changes
- File is deleted
- File is renamed
- Cache schema version changes

**Performance Characteristics:**

| Files | First Scan | Cached Scan | Speedup |
|-------|-----------|-------------|---------|
| 100   | 0.3s      | 0.05s       | 6x      |
| 1,000 | 2.5s      | 0.2s        | 12x     |
| 10,000| 25s       | 1.5s        | 17x     |

*Benchmarks on MacBook Pro M1, SSD, typical source code*

**Cache Storage:**

The SQLite database is lightweight:
- ~100 bytes per cached file
- 10,000 files ≈ 1MB cache database

---

## 11. Advanced: Tree-sitter Precision Mode

### What It Does

By default, todo-tracker uses regex-based scanning, which is fast but can produce false positives:

```rust
// This is a TODO
let var = "TODO: not actually a todo";  // False positive!
```

**Tree-sitter Precision Mode** uses Abstract Syntax Tree (AST) parsing to verify that TODOs are actually in comments, not strings, variable names, or other code elements.

### When to Use It

Enable precision mode if you experience:
- False positives from TODO keywords in strings
- TODOs in variable/function names being detected
- Need for 100% accuracy in code scanning tools

**Trade-offs:**
- **Pros**: No false positives, accurate parsing
- **Cons**: Slower scanning (2-3x), larger binary size, requires tree-sitter grammars

### Installation

Build with the `precise` feature:

```bash
cargo install --path . --features precise
```

This compiles tree-sitter grammar parsers into the binary.

### Usage

Enable in `.todo-tracker.toml`:

```toml
[treesitter]
enabled = true
```

Or via command line:

```bash
todos list --precise
```

### Supported Languages

Tree-sitter precision mode supports:

- **Rust** (.rs)
- **JavaScript** (.js, .jsx)
- **TypeScript** (.ts, .tsx)
- **Python** (.py)
- **Go** (.go)
- **Java** (.java)
- **C** (.c, .h)
- **C++** (.cpp, .hpp, .cc, .cxx, .hxx)
- **Ruby** (.rb)

Files in unsupported languages fall back to regex scanning.

### How It Works

1. Parse file into AST using tree-sitter
2. Find all comment nodes in the AST
3. Extract text from comment nodes
4. Search for TODO tags only in comment text

This guarantees that only comments are matched, eliminating false positives.

---

## 12. Supported Languages

todo-tracker recognizes TODO comments in 10 programming languages:

| Language   | Extensions                      | Line Comments | Block Comments |
|------------|---------------------------------|---------------|----------------|
| Rust       | .rs                             | `//`          | `/* */`        |
| Go         | .go                             | `//`          | `/* */`        |
| Python     | .py                             | `#`           | (none)         |
| JavaScript | .js, .jsx                       | `//`          | `/* */`        |
| TypeScript | .ts, .tsx                       | `//`          | `/* */`        |
| Java       | .java                           | `//`          | `/* */`        |
| C          | .c, .h                          | `//`          | `/* */`        |
| C++        | .cpp, .hpp, .cc, .cxx, .hxx     | `//`          | `/* */`        |
| C#         | .cs                             | `//`          | `/* */`        |
| Ruby       | .rb                             | `#`           | (none)         |

**Note:** While Python and Ruby don't have traditional block comments, todo-tracker also recognizes TODOs in multi-line strings/docstrings when they appear to be used as comments.

### Adding Custom Languages

You can extend language support by editing your `.todo-tracker.toml`:

```toml
# Add .ex files for Elixir
extensions = [".rs", ".go", ".py", ".js", ".ex"]

# Or in the config file, add custom comment patterns
[[languages]]
name = "Elixir"
extensions = [".ex", ".exs"]
line_comment = "#"
```

---

## 13. TODO Comment Syntax Guide

todo-tracker recognizes these patterns in comments:

### Basic TODO

```rust
// TODO: basic message
```

### TODO with Author

```rust
// TODO(alice): message
```

The author name appears in parentheses immediately after the tag.

### TODO with Author and Issue

```rust
// TODO(alice, #123): message
```

Issue references start with `#` followed by a number or alphanumeric string.

**Examples:**
- `#42` (GitHub issue)
- `#JIRA-123` (JIRA ticket)
- `#PROJ-456` (custom tracker)

### TODO with Author, Issue, and Priority

```rust
// TODO(alice, #123, p:high): message
```

Priority is specified with `p:` followed by the level.

**Priority Levels:**
- `p:low`
- `p:medium`
- `p:high`
- `p:critical`

### TODO with Priority Only

```rust
// FIXME(p:critical): message
```

### TODO with Issue Only

```rust
// BUG(#456): message
```

### All Recognized Tags

```rust
// TODO: general task
// FIXME: known issue to fix
// HACK: temporary workaround
// BUG: confirmed bug
// XXX: serious concern or warning
```

### Metadata Order

Metadata can appear in any order:

```rust
// TODO(#123, alice): message
// TODO(p:high, #123): message
// TODO(alice, p:high, #123): message
```

All of these are parsed correctly.

### Whitespace

Whitespace is flexible:

```rust
// TODO: message
// TODO : message
//TODO: message
//   TODO:   message
```

All are recognized.

### Block Comments

```rust
/*
 * TODO: multi-line message
 * that continues here
 */

/* FIXME(alice): single-line block comment */
```

### Multiple TODOs

```rust
// TODO: first task
// TODO: second task
// FIXME: unrelated issue
```

Each is tracked separately.

---

## 14. Ignoring Files

### `.gitignore` Integration

todo-tracker automatically respects your `.gitignore` file. Files and directories ignored by git are skipped:

```
# .gitignore
target/
node_modules/
*.log
.env
```

These files won't be scanned for TODOs.

### `.todoignore`

For additional exclusions specific to TODO scanning (without modifying `.gitignore`), create a `.todoignore` file:

```
# .todoignore
docs/archive/
vendor/
*.generated.js
third-party/
```

`.todoignore` uses the same syntax as `.gitignore`.

**Combining Rules:**

Files are excluded if they match EITHER `.gitignore` OR `.todoignore`.

### Configuration File Exclusions

You can also exclude files in `.todo-tracker.toml`:

```toml
exclude = [
  "target/",
  "node_modules/",
  "*.log",
  "docs/old/**"
]
```

### Max File Size

Large files are automatically skipped to avoid performance issues.

**Default:** 1 MB (1,048,576 bytes)

**Customize:**

```toml
max_file_size = 5242880  # 5MB
```

Files larger than this limit are skipped with a warning.

### Binary Files

Binary files are automatically detected and skipped. Detection uses:
1. File extension (e.g., `.exe`, `.dll`, `.so`, `.png`, `.jpg`)
2. Content heuristics (presence of null bytes)

---

## 15. Color Control

todo-tracker uses colors in terminal output to improve readability.

### Auto Mode (default)

```bash
todos list --color=auto
```

Automatically detects the environment:
- **Terminal (TTY)**: Colors enabled
- **Pipe or redirect**: Colors disabled

**Examples:**

```bash
# Colors shown
todos list

# No colors (output redirected)
todos list > output.txt

# No colors (piped to another command)
todos list | less
```

### Always Mode

Force colors even when piping or redirecting:

```bash
todos list --color=always
```

**Use case:** When piping to a tool that supports ANSI colors (like `less -R`):

```bash
todos list --color=always | less -R
```

### Never Mode

Disable colors completely:

```bash
todos list --color=never
```

**Use case:** When terminal doesn't support colors, or for plain text output:

```bash
todos list --color=never > plain_output.txt
```

### Color Configuration

Set default color behavior in `.todo-tracker.toml`:

```toml
color = "auto"  # or "always" or "never"
```

### Colors Used

When colors are enabled:

- **File paths**: Cyan/Blue
- **Line numbers**: Green
- **Tags**:
  - `TODO`: Cyan
  - `FIXME`: Yellow
  - `HACK`: Magenta
  - `BUG`: Red
  - `XXX`: Bright Red
- **Metadata** (author, issue, priority): Gray/Dim
- **Messages**: Default text color
- **Summary**: Bold

---

## 16. Command Reference

Quick reference for all commands and flags.

### Commands

| Command       | Description                                    |
|---------------|------------------------------------------------|
| `list`        | List all TODOs (default command)               |
| `blame`       | List TODOs with git blame information          |
| `diff`        | Show TODO changes between git references       |
| `check`       | Validate TODOs against policy rules            |
| `stats`       | Show statistical summary of TODOs              |
| `init`        | Create a `.todo-tracker.toml` config file      |
| `help`        | Show help information                          |
| `version`     | Show version information                       |

### Global Flags

Available for all commands:

| Flag                 | Description                                    |
|----------------------|------------------------------------------------|
| `--path <PATH>`      | Directory to scan (default: current directory) |
| `--config <FILE>`    | Path to config file                            |
| `--color <MODE>`     | Color output: auto, always, never              |
| `--help`             | Show help for command                          |
| `--version`          | Show version information                       |

### `list` Command Flags

| Flag                     | Description                                    |
|--------------------------|------------------------------------------------|
| `--format <FORMAT>`      | Output format: text, json, csv, markdown, count, sarif, github-actions |
| `--tag <TAGS>`           | Filter by tag (comma-separated)                |
| `--author <AUTHORS>`     | Filter by author (comma-separated)             |
| `--file <PATTERN>`       | Filter by file pattern (glob)                  |
| `--priority <PRIORITY>`  | Filter by priority (comma-separated)           |
| `--issue <ISSUE>`        | Filter by specific issue reference             |
| `--has-issue`            | Filter to items with any issue reference       |
| `--no-cache`             | Disable cache for this scan                    |
| `--clear-cache`          | Clear cache before scanning                    |
| `--precise`              | Use tree-sitter precision mode                 |

### `blame` Command Flags

| Flag                | Description                                    |
|---------------------|------------------------------------------------|
| `--format <FORMAT>` | Output format: text, json                      |
| `--sort <FIELD>`    | Sort by: file, line, date, author              |
| `--reverse`         | Reverse sort order                             |
| `--since <DATE>`    | Show only TODOs added since date               |
| `--until <DATE>`    | Show only TODOs added before date              |
| All `list` filters  | (--tag, --author, --file, --priority, etc.)    |

### `diff` Command Flags

| Flag                  | Description                                    |
|-----------------------|------------------------------------------------|
| `<REF>..<REF>`        | Git references to compare (e.g., main..HEAD)   |
| `--staged`            | Compare staged changes                         |
| `--format <FORMAT>`   | Output format: text, json                      |
| All `list` filters    | (--tag, --author, --file, --priority, etc.)    |

### `check` Command Flags

| Flag                           | Description                                    |
|--------------------------------|------------------------------------------------|
| `--max-todos <N>`              | Maximum total TODOs allowed                    |
| `--max-by-tag <TAG:N>`         | Maximum per tag (e.g., FIXME:20)               |
| `--require-issue <TAGS>`       | Tags that require issue references             |
| `--require-author <TAGS>`      | Tags that require author names                 |
| `--deny <TAGS>`                | Tags that are forbidden                        |
| `--min-priority <TAG:LEVEL>`   | Minimum priority for tag (e.g., BUG:high)      |

### `stats` Command Flags

| Flag                | Description                                    |
|---------------------|------------------------------------------------|
| `--format <FORMAT>` | Output format: text, json                      |
| All `list` filters  | (--tag, --author, --file, --priority, etc.)    |

### Examples by Use Case

**Daily workflow:**
```bash
# Quick scan
todos list

# See what I need to work on
todos list --author=$(git config user.name)

# Check before committing
todos diff --staged --tag=BUG
```

**Code review:**
```bash
# What TODOs does this PR add?
todos diff main..pr-branch

# Any critical issues?
todos list --priority=critical --format=markdown
```

**CI/CD pipeline:**
```bash
# Enforce policy
todos check --max-todos=100 --deny=NOCOMMIT

# Generate report
todos list --format=sarif > results.sarif
```

**Technical debt audit:**
```bash
# Full statistics
todos stats

# Who's responsible for the most TODOs?
todos blame --sort=author

# What files have the most debt?
todos stats --format=json | jq '.top_files'
```

**Integration with other tools:**
```bash
# Export to CSV for tracking in spreadsheet
todos list --format=csv > todos.csv

# Find high-priority items without issue references
todos list --priority=high --format=json | jq '.items[] | select(.issue == null)'

# Count FIXMEs for metrics dashboard
FIXME_COUNT=$(todos list --tag=FIXME --format=count)
```

---

## Getting Help

### Command-line Help

```bash
# General help
todos --help

# Help for specific command
todos list --help
todos check --help
```

### Documentation

- **README**: Project overview and quick start
- **USER_GUIDE.md**: This comprehensive guide
- **ARCHITECTURE.md**: Technical implementation details
- **CONTRIBUTING.md**: How to contribute

### Reporting Issues

Found a bug or have a feature request?

1. Check existing issues: https://github.com/yourusername/todo-tracker/issues
2. Create a new issue with:
   - Todo-tracker version (`todos --version`)
   - Operating system
   - Expected behavior
   - Actual behavior
   - Example code/commands to reproduce

### Community

- **GitHub Discussions**: Ask questions, share workflows
- **Discord**: Real-time chat with maintainers and users
- **Twitter**: Follow @todotracker for updates

---

## FAQ

### Why doesn't todo-tracker find my TODOs?

Check that:
1. Your file extension is supported (see Supported Languages)
2. The TODO is in a comment, not a string
3. The file isn't excluded by `.gitignore` or `.todoignore`
4. The file is under the max file size (default 1MB)

Run with `--verbose` to see which files are being scanned:
```bash
todos list --verbose
```

### How do I scan only specific file types?

Use the `--file` flag with glob patterns:
```bash
# Only JavaScript files
todos list --file="**/*.js"

# Only Rust and Go files
todos list --file="**/*.rs" --file="**/*.go"
```

Or configure in `.todo-tracker.toml`:
```toml
extensions = [".rs", ".go"]
```

### Can I use custom TODO tags?

Yes! Edit `.todo-tracker.toml`:
```toml
tags = ["TODO", "FIXME", "HACK", "BUG", "XXX", "NOTE", "OPTIMIZE"]
```

### How do I integrate with JIRA/Linear/etc.?

todo-tracker extracts issue references but doesn't integrate directly with issue trackers. You can:

1. **Export to JSON** and write a script to sync with your tracker:
   ```bash
   todos list --format=json | your-sync-script.py
   ```

2. **Use issue references** in your TODOs:
   ```rust
   // TODO(alice, JIRA-123): implement feature
   ```

3. **Filter by issue** to find TODOs for a specific ticket:
   ```bash
   todos list --issue=JIRA-123
   ```

### Does todo-tracker modify my code?

No. todo-tracker only reads files and never writes to them. It's safe to run at any time.

### How do I exclude generated files?

Add them to `.gitignore` or `.todoignore`:
```
# .todoignore
*.generated.js
dist/
build/
```

Or in `.todo-tracker.toml`:
```toml
exclude = ["*.generated.js", "dist/", "build/"]
```

### Can I use todo-tracker in a monorepo?

Yes! Place `.todo-tracker.toml` at the monorepo root, or create separate configs in each project directory.

To scan the entire monorepo:
```bash
todos list --path .
```

To scan a specific project:
```bash
todos list --path packages/my-project
```

### What's the performance impact of caching?

Caching makes repeat scans 10-20x faster with minimal overhead:
- Initial scan: Normal speed
- Cache writes: <10ms per file
- Cached scans: Only modified files are rescanned
- Storage: ~100 bytes per file (~1MB for 10,000 files)

### How do I disable colors in output?

```bash
todos list --color=never
```

Or set in config:
```toml
color = "never"
```

### Can I run todo-tracker offline?

Yes. todo-tracker is a standalone binary with no network dependencies. Git integration uses local repository data only.

---

## Conclusion

You now have a complete understanding of todo-tracker! Here are the key takeaways:

1. **Start simple**: Run `todos list` to see all TODOs
2. **Configure once**: Create `.todo-tracker.toml` and commit it
3. **Integrate early**: Add `todos check` to CI/CD
4. **Establish conventions**: Require authors and issues for important TODOs
5. **Track over time**: Use `todos blame` and `todos diff` to understand technical debt trends
6. **Export data**: Use JSON/CSV formats to integrate with other tools

Happy TODO tracking!
