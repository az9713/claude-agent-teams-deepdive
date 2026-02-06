# Quick Start Guide

Welcome! This guide will get you productive with the `todos` CLI tool in minutes. Each example takes under 2 minutes and gives you immediate, satisfying results.

---

## Getting Started (30 seconds)

First, verify the tool is installed:

```bash
todos --version
```

If you see a version number, you're ready to go! If not, check the [installation instructions](#installation) below.

### Try It Out Safely

All the examples below work with two modes:

- **Safe mode**: Use `--path tests/fixtures` to try commands on the sample files that ship with the project
- **Real mode**: Use `--path .` (or omit it) to scan your actual project

We recommend starting with the test fixtures to get a feel for the tool before running it on your codebase.

---

## 10 Quick Wins

### Use Case 1: Find All TODOs in Your Project

**What you'll learn**: Basic scanning

```bash
todos list --path tests/fixtures
```

**What you'll see**: A colorful, grouped-by-file list of all TODO comments. Each entry shows:
- The file path
- The line number
- The tag (TODO, FIXME, HACK, etc.)
- The full comment text
- Author and issue references (if present)

This is your first view into the hidden backlog lurking in your code. It's often eye-opening!

---

### Use Case 2: How Many TODOs Do I Have?

**What you'll learn**: Count format

```bash
todos list --path tests/fixtures --format=count
```

**What you'll see**: A single number. That's it. Clean and simple.

This is perfect for:
- Quick health checks ("Did the count go up or down since last week?")
- CI pipelines (compare before/after PR merge)
- Dashboard metrics

---

### Use Case 3: Find Only the Urgent Stuff

**What you'll learn**: Tag filtering

```bash
todos list --path tests/fixtures --tag=FIXME,HACK
```

**What you'll see**: Only FIXME and HACK items. TODOs are hidden.

**Why this matters**: Different tags mean different urgency levels:
- `FIXME`: Something is broken or wrong
- `HACK`: Temporary workaround that needs replacement
- `TODO`: Nice-to-have improvement
- `XXX`: Dangerous code, tread carefully

Filtering lets you triage ruthlessly. Before a release, scan for `FIXME` and `HACK` items. Leave `TODO` for the backlog.

---

### Use Case 4: What Did Alice Leave Behind?

**What you'll learn**: Author filtering

```bash
todos list --path tests/fixtures --author=alice
```

**What you'll see**: Only TODOs authored by alice (from comments like `// TODO(alice): ...`).

**Why this matters**:
- Code review prep: "What did Alice flag as incomplete in this PR?"
- Onboarding: "What areas did the previous dev identify as needing work?"
- Accountability: "Who owns this technical debt?"

---

### Use Case 5: Export to a Spreadsheet

**What you'll learn**: CSV output

```bash
todos list --path tests/fixtures --format=csv > my-todos.csv
```

**What you'll see**: A CSV file you can open in Excel, Google Sheets, or Numbers.

**Columns included**:
- File path
- Line number
- Tag
- Message
- Author (if present)
- Issue reference (if present)
- Priority (if present)

Perfect for sharing with non-technical stakeholders or doing pivot table analysis.

---

### Use Case 6: Generate a TODO Report for Your Team

**What you'll learn**: Markdown output

```bash
todos list --path tests/fixtures --format=markdown > TODO_REPORT.md
```

**What you'll see**: A nicely formatted Markdown file with:
- Grouped by file
- Clickable line numbers (if you add repo URLs)
- Ready to paste into wikis, Notion, Confluence, or PR descriptions

Share this in your weekly standup Slack thread or pin it to your team's project board.

---

### Use Case 7: See the Big Picture with Statistics

**What you'll learn**: Stats command with bar charts

```bash
todos stats --path tests/fixtures
```

**What you'll see**: Unicode bar charts showing:
- **Distribution by tag**: How many TODO vs FIXME vs HACK items?
- **Distribution by file**: Which files are the most TODO-dense?
- **Distribution by author**: Who left the most TODOs?

This is your "executive dashboard" view. Great for retrospectives and planning sessions.

---

### Use Case 8: Who Wrote This TODO and When?

**What you'll learn**: Git blame integration

```bash
todos blame --path tests/fixtures --sort=date
```

**What you'll see**: Each TODO annotated with:
- Git author (the person who added the comment)
- Commit date
- Commit hash

**Important**: This requires your project to be in a git repository. It won't work on the test fixtures unless you initialize a git repo there first.

**Why this matters**: Find stale TODOs. If a TODO was added 3 years ago and never addressed, maybe it's not important anymore.

---

### Use Case 9: What TODOs Were Added in This Branch?

**What you'll learn**: Git diff

```bash
todos diff main..HEAD
```

**What you'll see**:
- Green `+` lines for newly added TODOs
- Red `-` lines for removed TODOs
- The delta between two git refs

**Use cases**:
- Pre-merge review: "Did this PR add a bunch of TODOs?"
- Release notes: "We cleared 12 FIXMEs this sprint!"
- CI gate: "Fail the build if new FIXMEs are introduced"

**Note**: Requires a git repository. You can also use `--staged` to see TODOs in staged files.

---

### Use Case 10: Set Up a CI Quality Gate

**What you'll learn**: Policy checks

```bash
todos check --path tests/fixtures --max-todos=100 --deny=NOCOMMIT
```

**What you'll see**:
- Exit code `0` if all checks pass (green light for CI)
- Exit code `1` if violations found (red light, fail the build)
- Clear error messages explaining violations

**Example policy checks**:
- `--max-todos=100`: Fail if more than 100 TODOs exist
- `--deny=NOCOMMIT`: Fail if any NOCOMMIT tags are found (these should never be committed)
- `--require-issue=FIXME`: Fail if any FIXME lacks an issue reference like `#123`

Add this to your `.github/workflows/ci.yml` or `.gitlab-ci.yml` to prevent technical debt from sneaking in.

---

## Bonus Use Cases (Level Up)

### Use Case 11: Pipe JSON to jq for Custom Queries

```bash
# How many TODOs total?
todos list --path tests/fixtures --format=json | jq '.items | length'

# How many FIXMEs?
todos list --path tests/fixtures --format=json | jq '[.items[] | select(.tag == "FIXME")] | length'

# List all TODOs with high priority
todos list --path tests/fixtures --format=json | jq '.items[] | select(.priority == "high")'

# Group by author
todos list --path tests/fixtures --format=json | jq 'group_by(.author) | map({author: .[0].author, count: length})'
```

JSON output makes `todos` composable with any tool in your pipeline.

---

### Use Case 12: Scan Only Your Source Directory

```bash
todos list --path=src --file="*.rs"
```

Narrow your scan to specific directories and file types. Great for monorepos or large codebases where you only care about certain parts.

---

### Use Case 13: Find High-Priority Items With Issue Tracking

```bash
todos list --path tests/fixtures --priority=high --has-issue
```

Show only high-priority TODOs that have associated issue references (like `#123`). Perfect for release planning.

---

### Use Case 14: Create a .todoignore File

```bash
echo "vendor/" > .todoignore
echo "node_modules/" >> .todoignore
echo "*.generated.go" >> .todoignore
todos list
```

Exclude directories or files from scanning. The `.todoignore` file uses the same syntax as `.gitignore`.

This speeds up scans and reduces noise from third-party code.

---

### Use Case 15: Initialize a Config File

```bash
todos init
```

Creates a `.todo-tracker.toml` file in the current directory with default settings. Edit this file to set project-wide defaults like:

```toml
[scan]
path = "src"
tags = ["TODO", "FIXME", "HACK", "BUG"]

[policy]
max_todos = 50
deny = ["NOCOMMIT"]
require_issue = ["FIXME"]

[output]
format = "text"
color = "auto"
```

Now you don't have to pass the same flags every time!

---

## What's Next?

- **Comprehensive reference**: See `USER_GUIDE.md` for all commands, flags, and configuration options
- **Contributing**: See `DEVELOPER_GUIDE.md` if you want to add features or fix bugs
- **Behind the scenes**: Read `AGENT_TEAM_CASE_STUDY.md` for the fascinating story of how this tool was built by 11 AI agents working in parallel

---

## Cheat Sheet

Quick reference for the most common commands:

| Task | Command |
|------|---------|
| Scan current dir | `todos list` |
| Scan specific dir | `todos list --path=src` |
| Count TODOs | `todos list --format=count` |
| Filter by tag | `todos list --tag=FIXME` |
| Filter by author | `todos list --author=alice` |
| JSON output | `todos list --format=json` |
| CSV export | `todos list --format=csv > out.csv` |
| Markdown report | `todos list --format=markdown > report.md` |
| Statistics | `todos stats` |
| Git blame | `todos blame` |
| Git diff | `todos diff main..HEAD` |
| CI check | `todos check --max-todos=50` |
| Init config | `todos init` |
| Clear cache | `todos list --clear-cache` |

---

## Installation

### Pre-built binaries

Download the latest release from the GitHub releases page and extract it to your `$PATH`.

### From source

```bash
cargo install --path .
```

### With tree-sitter precision mode

For more accurate parsing (especially for multi-line comments):

```bash
cargo install --path . --features precise
```

### Docker

```bash
docker build -t todos .
docker run -v $(pwd):/workspace todos list --path=/workspace
```

---

## Troubleshooting

**Q: I get "command not found: todos"**

A: The binary isn't in your PATH. Either:
- Add `~/.cargo/bin` to your PATH (if installed via cargo)
- Run `./target/release/todos` directly
- Install with `cargo install --path .` to put it in `~/.cargo/bin`

**Q: Git commands (blame, diff) don't work**

A: These features require:
1. Your project to be in a git repository
2. The `git` command to be installed and in your PATH

**Q: Scanning is slow on large repos**

A: Try:
- Using `--path` to scan only specific directories
- Creating a `.todoignore` file to exclude large directories
- Using `--file` to filter by extension (e.g., `--file="*.rs"`)
- The tool caches results, so subsequent runs are faster

**Q: I want to see more/fewer TODO tags**

A: The tool detects common tags by default (TODO, FIXME, HACK, XXX, BUG, NOTE). To customize, create a `.todo-tracker.toml` file with:

```toml
[scan]
tags = ["TODO", "FIXME", "CUSTOM_TAG"]
```

---

## Tips for Success

1. **Start small**: Try on test fixtures first, then a single directory, then your whole project
2. **Use filters**: Don't get overwhelmed by hundreds of TODOs. Filter by tag or author to focus
3. **Set up CI**: Add `todos check` to your CI pipeline early. Prevent TODOs from piling up
4. **Track trends**: Run `todos list --format=count` weekly and chart the results
5. **Make it a habit**: Add TODO scanning to your code review checklist

---

Happy tracking! Remember: every TODO you find is a chance to make your codebase better.
