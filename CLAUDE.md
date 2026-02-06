# todo-tracker

## Project Overview

- **Name**: todo-tracker
- **Binary**: `todos`
- **Language**: Rust (edition 2021)
- **Purpose**: Fast cross-language TODO/FIXME/HACK/BUG/XXX comment linter
- **Case Study**: This project demonstrates Claude Code Agent Teams (see AGENT_TEAM_CASE_STUDY.md)

## Build Commands

```bash
cargo build                        # Debug build
cargo build --release              # Optimized build
cargo test                         # Run all tests (141 unit + 9 integration)
cargo test --features precise      # Run with tree-sitter tests (164 unit + 9 integration)
cargo check                        # Fast compilation check (no binary)
cargo run -- <args>                # Run in dev mode
cargo test test_name               # Run single test
cargo test --lib module::tests     # Run module tests
```

## Project Structure

```
src/
├── main.rs                        # CLI entry point, command wiring
├── lib.rs                         # Module declarations
├── cli.rs                         # clap derive CLI definition
├── model.rs                       # TodoItem, TodoTag, Priority, ScanResult, ScanStats, ScanMetadata
├── error.rs                       # TodoError enum (thiserror), Result type alias
├── config.rs                      # TOML config loader with 3-tier resolution
├── discovery.rs                   # File discovery with .gitignore/.todoignore support
├── filter.rs                      # AND-combined filter engine with glob matching
├── policy.rs                      # CI policy engine (max_todos, require_issue, deny_tags)
├── progress.rs                    # indicatif progress bar
├── scanner/
│   ├── mod.rs                     # FileScanner trait, ScanOrchestrator
│   ├── languages.rs               # 10-language comment syntax database
│   ├── regex.rs                   # Regex-based scanner with metadata extraction
│   ├── treesitter.rs              # Tree-sitter AST verification (feature: precise)
│   ├── incremental.rs             # Cache-backed incremental scanner
│   └── mmap.rs                    # Memory-mapped file reading for large files
├── output/
│   ├── mod.rs                     # OutputFormatter trait, format dispatch
│   ├── text.rs                    # Colored text formatter
│   ├── json.rs                    # JSON formatter
│   ├── csv.rs                     # CSV formatter
│   ├── markdown.rs                # Markdown formatter
│   ├── sarif.rs                   # SARIF v2.1.0 for GitHub Code Scanning
│   └── github_actions.rs          # GitHub Actions annotations
├── git/
│   ├── mod.rs                     # Module declarations
│   ├── utils.rs                   # Git CLI helpers
│   ├── blame.rs                   # Git blame porcelain parser
│   └── diff.rs                    # TODO diff between git refs
└── cache/
    ├── mod.rs                     # Module declarations
    ├── db.rs                      # SQLite WAL-mode cache
    └── migrations.rs              # Schema creation

tests/
├── cli_test.rs                    # Integration tests
└── fixtures/                      # Sample files in 10 languages
```

## Architecture Patterns

- **FileScanner trait**: All scanners implement `scan_file(&self, path: &Path) -> Result<Vec<TodoItem>>`
- **OutputFormatter trait**: All formatters implement `format(&self, result: &ScanResult) -> Result<String>`
- **ScanOrchestrator**: Parallel scanning with rayon, optional SQLite caching
- **Filter pipeline**: scan → filter → format (applied in main.rs)
- **Error handling**: thiserror for library, anyhow at main level

## Key Dependencies

- **CLI & Parallelism**: clap 4, rayon
- **Scanning**: regex, tree-sitter + 8 language grammars (optional, `precise` feature)
- **Storage**: rusqlite bundled (SQLite caching)
- **Output**: serde, serde_json, colored
- **File Discovery**: ignore (gitignore support)

## Conventions

- Tests live alongside code in `#[cfg(test)] mod tests` blocks
- Integration tests in `tests/cli_test.rs` use assert_cmd
- Use `cargo test` before committing
- Keep FileScanner implementations in `src/scanner/`
- Keep OutputFormatter implementations in `src/output/`
- All CLI commands wired in `src/main.rs`

## Documentation

- **README.md** - Project overview and goals
- **AGENT_TEAM_CASE_STUDY.md** - Detailed Agent Team behavior study
- **docs/DEVELOPER_GUIDE.md** - Developer documentation
- **docs/USER_GUIDE.md** - User documentation
- **docs/QUICK_START.md** - Quick start with 10 use cases
