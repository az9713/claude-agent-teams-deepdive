# Claude Opus 4.6 Agent Teams: A Real-World Case Study

## Building a 7,500-Line Rust CLI in One Session with 11 Parallel Agents

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [What Are Agent Teams?](#what-are-agent-teams)
3. [The Project: todo-tracker](#the-project)
4. [Team Architecture](#team-architecture)
5. [Phase-by-Phase Execution](#phase-by-phase-execution)
6. [Visual Timeline](#visual-timeline)
7. [Agent Roster](#agent-roster)
8. [Coordination Patterns](#coordination-patterns)
9. [Conflict Resolution](#conflict-resolution)
10. [Final Statistics](#final-statistics)
11. [Key Takeaways](#key-takeaways)

---

## Executive Summary

This document captures a real-world session where Claude Opus 4.6 orchestrated a team
of 11 specialized builder agents to implement a complete Rust CLI application from a
design plan. The project -- `todo-tracker` (binary: `todos`) -- is a fast, cross-language
TODO linter with 7 implementation phases, 30 source files, 5,759 lines of Rust, and
173 passing tests.

**The entire build took approximately 75 minutes**, from plan to committed code.

The session demonstrates how Agent Teams enable:
- **Parallelism**: 3-4 agents building independent modules simultaneously
- **Specialization**: Each agent owns specific files with clear boundaries
- **Coordination**: A team lead orchestrates phases, resolves conflicts, and integrates work
- **Scalability**: The same pattern works for 3 agents or 11 agents

---

## What Are Agent Teams?

Agent Teams are a feature in Claude Code where the primary Claude instance (the **team lead**)
can spawn specialized sub-agents (**teammates**) that work in parallel on independent tasks.
Each teammate:

- Runs as a separate Claude instance with its own context window
- Has access to file read/write, bash, and search tools
- Communicates with the team lead via structured messages
- Shares a task list for coordination
- Can be shut down gracefully when work is complete

The team lead is responsible for:
- Creating the team and task list
- Spawning teammates with clear, detailed prompts
- Creating scaffolding/stubs so the project compiles before agents write code
- Integrating agent output (resolving conflicts, wiring modules together)
- Running tests and verifying the combined output
- Shutting down agents and cleaning up

### Key Concepts

| Concept | Description |
|---------|-------------|
| **Team** | A named group of agents sharing a task list (`todo-tracker-build`) |
| **Team Lead** | The primary Claude instance that orchestrates everything |
| **Teammate** | A spawned sub-agent working on a specific task |
| **Task List** | Shared todo list at `~/.claude/tasks/{team-name}/` |
| **Inbox** | Message queue for agent-to-agent communication |
| **Idle State** | Normal state when a teammate finishes a turn and waits for input |
| **Shutdown Request** | Graceful termination signal sent by the team lead |

---

## The Project

**todo-tracker** (`todos`) is a fast, cross-language TODO linter in Rust.

### Capabilities
- Scans codebases for `TODO`, `FIXME`, `HACK`, `BUG`, `XXX` comments
- Supports 10 programming languages with comment-syntax awareness
- Extracts metadata: author, issue reference, priority level
- Multiple output formats: colored text, JSON, CSV, Markdown, SARIF, GitHub Actions
- Git integration: blame enrichment, diff between refs
- CI policy engine: max TODOs, require issue refs, deny tags
- SQLite caching for incremental scanning
- Optional tree-sitter AST verification for false positive elimination

### Architecture (30 source files)
```
src/
  main.rs          (461 lines)  CLI entry point and command wiring
  lib.rs           (12 lines)   Module declarations
  cli.rs           (99 lines)   Clap derive CLI definition
  config.rs        (250 lines)  TOML config with directory walk-up
  error.rs         (24 lines)   Error types (thiserror)
  model.rs         (116 lines)  Core data model (TodoItem, ScanResult, etc.)
  filter.rs        (380 lines)  Filter engine with glob matching
  policy.rs        (322 lines)  CI policy engine
  progress.rs      (37 lines)   Progress bar (indicatif)
  discovery.rs     (209 lines)  File discovery with .gitignore support
  scanner/
    mod.rs         (287 lines)  FileScanner trait + ScanOrchestrator
    languages.rs   (234 lines)  10-language comment syntax database
    regex.rs       (419 lines)  Regex-based scanner with metadata extraction
    treesitter.rs  (516 lines)  Tree-sitter AST verification (feature-gated)
    incremental.rs (42 lines)   Cache-backed incremental scanner
    mmap.rs        (20 lines)   Memory-mapped file reading
  output/
    mod.rs         (69 lines)   OutputFormatter trait + dispatch
    text.rs        (421 lines)  Colored grouped-by-file text output
    json.rs        (140 lines)  JSON formatter
    csv.rs         (217 lines)  CSV formatter
    markdown.rs    (282 lines)  Markdown formatter
    sarif.rs       (229 lines)  SARIF v2.1.0 for GitHub Code Scanning
    github_actions.rs (166 lines) ::warning/::error annotations
  git/
    mod.rs         (3 lines)    Module declarations
    utils.rs       (35 lines)   Git CLI helpers
    blame.rs       (267 lines)  Git blame porcelain parser
    diff.rs        (176 lines)  TODO diff between refs
  cache/
    mod.rs         (4 lines)    Module declarations
    db.rs          (287 lines)  SQLite WAL-mode cache
    migrations.rs  (35 lines)   Schema creation
```

---

## Team Architecture

### The Strategy: Phased Parallel Execution

The 7 phases of the plan have sequential dependencies (Phase 2 builds on Phase 1's
data model, etc.), but **within each phase**, modules are independent. The team lead
exploited this by:

1. Completing scaffolding (stubs, interfaces, Cargo.toml) for a phase
2. Spawning 2-4 agents in parallel to implement independent modules
3. Waiting for agents to complete
4. Integrating their work (wiring modules together in main.rs/cli.rs)
5. Running `cargo check` and `cargo test` to verify
6. Moving to the next phase

### Team Composition Across Phases

```
Phase 1 (MVP):          [scanner-agent] [discovery-agent] [output-agent]
Phase 2 (Core):         [config-agent]  [formatters-agent] [filter-agent]
Phase 3 (Git):          [blame-agent]   [diff-agent]
Phase 4 (CI):           [policy-agent]
Phase 5 (Performance):  [cache-agent]
Phase 6 (Precision):    [tree-sitter background task]
Phase 7 (Distribution): [distro-agent]
```

Note: Phases 3+4 overlapped. Phases 5+7 overlapped. Phase 6 ran as a background
task agent while the lead did integration work.

---

## Phase-by-Phase Execution

### Phase 1: MVP -- Minimum Viable Scanner

**Goal**: `todos list` works with grouped output, metadata extraction, color coding.

#### Step 1: Team Lead Creates Scaffolding (~01:38-01:42 UTC)

Before spawning any agents, the team lead created the project skeleton so that
agents would write code into a compilable project:

| File | Purpose |
|------|---------|
| `Cargo.toml` | All dependencies declared (clap, ignore, regex, serde, rayon, etc.) |
| `src/lib.rs` | Module declarations for all planned modules |
| `src/error.rs` | `TodoError` enum with thiserror, `Result<T>` type alias |
| `src/model.rs` | `TodoItem`, `ScanResult`, `ScanStats`, `ScanMetadata`, `Priority`, `TodoTag` |
| `src/scanner/mod.rs` | `FileScanner` trait definition (stub) |
| `src/output/mod.rs` | `OutputFormatter` trait definition (stub) |

**Why stubs first?** Agents write code that imports from other modules. Without stubs
declaring the traits and types, agents' code wouldn't compile, and they couldn't
run `cargo check` to verify their work.

#### Step 2: Spawn 3 Agents in Parallel (~01:42 UTC)

The team lead spawned three agents simultaneously using the `Task` tool with
`team_name: "todo-tracker-build"`:

| Agent | subagent_type | Files Assigned | Prompt Summary |
|-------|--------------|----------------|----------------|
| **scanner-agent** | `general-purpose` | `scanner/languages.rs`, `scanner/regex.rs` | Build 10-language comment DB + regex scanner with metadata extraction |
| **discovery-agent** | `general-purpose` | `discovery.rs`, `scanner/mod.rs` | File discovery with .gitignore + scan orchestrator with rayon |
| **output-agent** | `general-purpose` | `output/text.rs`, `output/mod.rs` | Colored text formatter with grouping + output dispatch |

Each agent received a detailed prompt containing:
- The exact file paths to create
- The trait/struct interfaces they must implement
- The data types they'd work with (copied from model.rs)
- Specific requirements (e.g., "10 languages: Rust, Go, Python, JS, TS, Java, C, C++, C#, Ruby")
- Testing requirements

#### Step 3: Agents Work Independently (~01:42-01:50 UTC)

All three agents worked simultaneously, each in their own context:

**scanner-agent** (completed ~01:50):
- Read `model.rs` to understand `TodoItem`, `TodoTag`, `Priority`
- Created `languages.rs` with 10 `Language` structs, `LanguageDatabase` with HashMap lookup
- Created `regex.rs` with `RegexScanner` implementing `FileScanner`
- Two regex patterns: bare tag + tag with parenthesized metadata
- Language-aware block comment depth tracking
- Wrote 29 unit tests (12 language + 17 regex)

**discovery-agent** (completed ~01:48):
- Created `discovery.rs` with `FileDiscovery` using `ignore::WalkBuilder`
- Added `.todoignore` support, binary file filtering, max file size
- Completed `scanner/mod.rs` with `ScanOrchestrator` using `rayon::par_iter`
- Wrote 11 unit tests (7 discovery + 4 orchestrator)

**output-agent** (completed ~01:47):
- Created `output/text.rs` with `TextFormatter` -- colored, grouped by file
- Completed `output/mod.rs` with `OutputFormat` enum and `format_output()` dispatch
- Wrote 12 unit tests for text formatting

#### Step 4: Team Lead Integrates (~01:50-02:00 UTC)

After all three agents reported completion, the team lead:
1. Created `src/cli.rs` with clap derive (List, Scan subcommands, global flags)
2. Created `src/main.rs` wiring scanner -> discovery -> orchestrator -> formatter
3. Created 12 test fixture files (sample.rs, .go, .py, .js, .ts, .java, .c, .cpp, .cs, .rb, false_positive.rs, block_comments.js)
4. Created `tests/cli_test.rs` with 9 integration tests
5. Ran `cargo test` -- **52 tests pass**
6. Fixed a bug: `scan_duration_ms` was being stored into `files_scanned` after filtering

**Test count after Phase 1: 52 unit + 9 integration = 61 tests**

---

### Phase 2: Core Features -- JSON, Config, Filtering, Stats

**Goal**: Machine-readable output, TOML config, rich filtering, `todos stats`.

#### Setup (~02:00 UTC)
Team lead added `csv` and `unicode-width` to Cargo.toml, created config.rs and filter.rs
stubs, updated lib.rs.

#### Spawn 3 Agents (~02:00 UTC)

| Agent | Files Assigned | Output |
|-------|----------------|--------|
| **config-agent** | `config.rs`, CLI `Init` cmd, main.rs wiring | 250 lines, 9 tests |
| **formatters-agent** | `output/json.rs`, `output/csv.rs`, `output/markdown.rs`, `output/mod.rs` updates | 639 lines, 17 tests |
| **filter-agent** | `filter.rs`, CLI filter flags, `Stats` cmd, main.rs stats wiring | 380 lines, 18 tests |

#### Agents Complete (~02:04-02:08 UTC)

- **config-agent** (~02:04): TOML config with 3-tier resolution (explicit > walk-up > user home > default), `todos init` command, commented template
- **formatters-agent** (~02:04): JSON (serde_json), CSV (csv crate with namespace alias), Markdown (grouped with headings), all wired into output dispatch
- **filter-agent** (~02:08): AND-combined filters for tag/author/file-glob/priority/has-issue, `todos stats` with Unicode bar charts, refactored main.rs with `build_filter()`/`apply_filter()` helpers

#### Integration (~02:08-02:10 UTC)
Team lead ran `cargo test` -- **96 unit + 9 integration = 105 tests pass**

---

### Phase 3: Git Integration + Phase 4: CI Mode (Overlapped)

**Innovation**: The team lead started Phase 4's agent before Phase 3's agents finished,
because Phase 4 only depends on the Phase 1/2 code that was already stable.

#### Spawn 3 Agents (~02:10 UTC)

| Agent | Phase | Files Assigned |
|-------|-------|----------------|
| **blame-agent** | 3 | `git/blame.rs`, `git/utils.rs`, CLI `Blame` cmd |
| **diff-agent** | 3 | `git/diff.rs`, CLI `Diff` cmd |
| **policy-agent** | 4 | `policy.rs`, `output/sarif.rs`, `output/github_actions.rs`, CLI `Check` cmd |

#### Agent Interactions (~02:10-02:28 UTC)

This phase revealed the most interesting coordination challenge. Three agents were
modifying overlapping files (`cli.rs` and `main.rs`):

- **diff-agent** added the `Diff` command and `run_diff()` function
- **blame-agent** added the `Blame` command and `run_blame()` function
- **policy-agent** added the `Check` command, `run_check()`, SARIF/GA formatters

The **diff-agent** noticed borrow-checker issues from the other agents' match arms
and proactively fixed them by adding `ref` keywords. It also noted that blame-agent's
`enrich_with_blame` had a return type mismatch (returning `()` but being called with
`.map_err()`).

#### Completion Times
- **diff-agent**: ~02:26 (176 lines git/diff.rs + CLI wiring)
- **blame-agent**: ~02:27 (267 lines git/blame.rs with custom timestamp formatter -- no chrono dep, 8 tests)
- **policy-agent**: ~02:27 (322 lines policy.rs with 13 tests, 229 lines sarif.rs with 9 tests, 166 lines github_actions.rs with 7 tests)

#### Integration (~02:28-02:35 UTC)
Team lead resolved the `enrich_with_blame` return type issue, verified all CLI commands
worked: `todos check --deny=NOCOMMIT`, SARIF output, GitHub Actions annotations.

**Test count: 133 unit + 9 integration = 142 tests**

---

### Phase 5: Performance + Phase 7: Distribution (Overlapped)

**Another overlap**: Phase 7 (distribution files) has zero code dependencies on
Phase 5 (caching), so both ran in parallel.

#### Setup (~02:35 UTC)
Team lead added `rusqlite` (bundled), `indicatif`, `memmap2` to Cargo.toml, created
stub modules for cache/, scanner/incremental.rs, scanner/mmap.rs, progress.rs.

#### Spawn 2 Agents (~02:35 UTC)

| Agent | Phase | Files Assigned |
|-------|-------|----------------|
| **cache-agent** | 5 | `cache/db.rs`, `cache/migrations.rs`, `cache/mod.rs`, `scanner/incremental.rs`, `scanner/mmap.rs`, `progress.rs` |
| **distro-agent** | 7 | `.github/workflows/ci.yml`, `release.yml`, `Dockerfile`, `.pre-commit-hooks.yaml`, Cargo.toml binstall metadata |

#### Completion
- **distro-agent**: ~02:41 (CI matrix for 3 platforms, release pipeline for 6 targets, Alpine Dockerfile, pre-commit hooks)
- **cache-agent**: ~02:42 (SQLite WAL-mode cache with 8 tests, incremental scanner, mmap reader, progress bar)

#### Integration (~02:43-02:48 UTC)
Team lead wired caching into `ScanOrchestrator`:
- Added `scan_with_cache()` method using `IncrementalScanner`
- Added `open_cache()` helper in main.rs
- Added `--clear-cache` CLI flag
- Updated all commands (scan, stats, check, blame) to use cache
- Verified: "Scanned 12 files (12 from cache) in 18ms"

**Test count: 141 unit + 9 integration = 150 tests**

---

### Phase 6: Tree-sitter Precision

**Goal**: AST-verified comment detection behind `--features precise`.

#### Setup (~02:43 UTC)
Team lead added tree-sitter + 8 language grammar crates as optional deps behind
a `precise` feature flag in Cargo.toml. Added conditional module declaration
in scanner/mod.rs.

#### Background Task Agent (~02:43-02:48 UTC)
Instead of a teammate, the lead used a **background Task agent** (subagent_type: `builder`)
since this was a single-file task with no coordination needed:

| Task | File | Output |
|------|------|--------|
| tree-sitter scanner | `scanner/treesitter.rs` | 516 lines, 23 tests |

The agent created `TreeSitterScanner` wrapping `RegexScanner`, with:
- Language-to-grammar mapping for 8 languages
- Tree-sitter query `(comment) @comment` to extract comment byte ranges
- Line-to-byte-offset conversion for candidate verification
- `PrecisionStats` accuracy reporting
- Fallback to regex results for unknown languages or parse failures

#### Fix Required
The agent used the standard `Iterator` trait for `QueryMatches`, but tree-sitter v0.24
uses `StreamingIterator` from the `streaming-iterator` crate. Team lead fixed:
1. Added `streaming-iterator` as optional dep
2. Changed `for match_ in matches` to `while let Some(match_) = matches.next()`
3. Added the `use streaming_iterator::StreamingIterator` import

**Final test count: 164 unit + 9 integration = 173 tests**

---

## Visual Timeline

```
Time (UTC)  Team Lead Activity                    Agents Running
02/06
01:38  |  Create team "todo-tracker-build"
01:40  |  Write scaffolding (Cargo.toml,
       |  model.rs, error.rs, lib.rs, stubs)
01:42  |  Spawn Phase 1 agents .................. [scanner-agent] [discovery-agent] [output-agent]
       |  (waiting for agents)                     |                |                |
01:47  |                                           |                |                * output done (12 tests)
01:48  |                                           |                * discovery done (11 tests)
01:50  |                                           * scanner done (29 tests)
       |  Integrate: cli.rs, main.rs,
       |  test fixtures, integration tests
01:55  |  Fix bug: scan_duration_ms -> files_scanned
02:00  |  `cargo test` = 61 pass
       |  Spawn Phase 2 agents .................. [config-agent] [formatters-agent] [filter-agent]
       |  (waiting for agents)                     |               |                 |
02:04  |                                           * config done   * formatters done |
       |                                           | (9 tests)     | (17 tests)      |
02:08  |                                                                             * filter done (18 tests)
       |  Integrate: format dispatch,
       |  CLI wiring, verify
02:10  |  `cargo test` = 105 pass
       |  Spawn Phase 3+4 agents ................ [blame-agent] [diff-agent] [policy-agent]
       |  (waiting for agents)                     |              |            |
       |                                           |              |            |
02:26  |                                           |              * diff done  |
02:27  |                                           * blame done   | (0 tests*) * policy done (29 tests)
       |  Integrate: fix borrow issues,            |              |
       |  verify all commands
       |  `cargo test` = 142 pass
02:35  |  Setup Phase 5 stubs
       |  Spawn Phase 5+7 agents ................ [cache-agent]  [distro-agent]
       |  (waiting for agents)                     |               |
02:41  |                                           |               * distro done (0 tests**)
02:42  |                                           * cache done (8 tests)
02:43  |  Wire caching into orchestrator
       |  Add --clear-cache flag
       |  Spawn Phase 6 background task ......... [tree-sitter task]
02:45  |  `cargo test` = 150 pass                  |
02:48  |                                           * tree-sitter done (23 tests)
       |  Fix streaming-iterator API
       |  `cargo test --features precise`
       |  = 173 pass
02:55  |  Send shutdown to all 11 agents           x x x x x x x x x x x
02:56  |  All agents terminated
       |  Team cleanup
02:57  |  `git commit` (50 files, 7521 lines)

*  diff-agent: tests are integration-level, tested via CLI
** distro-agent: YAML/Dockerfile files, no unit tests applicable
```

### Gantt-Style View of Agent Lifespans

```
Agent              01:42    01:50    02:00    02:10    02:20    02:30    02:40    02:50    02:56
                     |        |        |        |        |        |        |        |        |
scanner-agent      [========]idle.....................................................................X
discovery-agent    [======]idle.......................................................................X
output-agent       [=====]idle........................................................................X
config-agent                          [====]idle..................................................X
formatters-agent                      [====]idle..................................................X
filter-agent                          [========]idle..............................................X
blame-agent                                      [=================]idle......................X
diff-agent                                       [================]idle.......................X
policy-agent                                     [=================]idle......................X
cache-agent                                                              [=======]idle........X
distro-agent                                                             [======]idle.........X

Legend: [====] = actively working    idle = waiting    X = shutdown
```

---

## Agent Roster

### Complete Agent Directory

| # | Agent Name | Phase | Role | Model | Lines Written | Tests Written | Created (UTC) | Completed | Shut Down |
|---|-----------|-------|------|-------|--------------|--------------|---------------|-----------|-----------|
| 1 | scanner-agent | 1 | Regex scanner + language DB | Sonnet 4.5 | 653 | 29 | ~01:42 | ~01:50 | 02:56:07 |
| 2 | discovery-agent | 1 | File discovery + orchestrator | Sonnet 4.5 | 496 | 11 | ~01:42 | ~01:48 | 02:56:02 |
| 3 | output-agent | 1 | Text formatter + output dispatch | Sonnet 4.5 | 490 | 12 | ~01:42 | ~01:47 | 02:56:02 |
| 4 | config-agent | 2 | TOML config + init command | Sonnet 4.5 | 250 | 9 | ~02:00 | ~02:04 | 02:55:13 |
| 5 | formatters-agent | 2 | JSON, CSV, Markdown formatters | Sonnet 4.5 | 639 | 17 | ~02:00 | ~02:04 | 02:55:22 |
| 6 | filter-agent | 2 | Filter engine + stats command | Sonnet 4.5 | 380 | 18 | ~02:00 | ~02:08 | 02:55:24 |
| 7 | blame-agent | 3 | Git blame parser + blame command | Sonnet 4.5 | 267 | 8 | ~02:10 | ~02:27 | 02:56:08 |
| 8 | diff-agent | 3 | Git diff + diff command | Sonnet 4.5 | 211 | 0* | ~02:10 | ~02:26 | 02:55:51 |
| 9 | policy-agent | 4 | Policy engine + SARIF + GA output | Sonnet 4.5 | 717 | 29 | ~02:10 | ~02:27 | 02:55:54 |
| 10 | cache-agent | 5 | SQLite cache + incremental scanner | Sonnet 4.5 | 425 | 8 | ~02:35 | ~02:42 | 02:56:02 |
| 11 | distro-agent | 7 | CI/CD, Docker, pre-commit | Sonnet 4.5 | ~150 | 0** | ~02:35 | ~02:41 | 02:55:48 |
| -- | tree-sitter (bg task) | 6 | Tree-sitter AST scanner | Sonnet 4.5 | 516 | 23 | ~02:43 | ~02:48 | (auto) |
| -- | **Team Lead** | All | Orchestration + integration | **Opus 4.6** | ~1,565 | 9 | session start | session end | -- |

\* diff-agent: functionality tested via integration tests
\*\* distro-agent: YAML/Dockerfile files have no unit test framework

### Agent Type Breakdown

**Team Lead (Opus 4.6)**: The orchestrator. Does NOT write most code. Instead:
- Creates project scaffolding and interface stubs
- Writes detailed prompts for each agent
- Integrates agent output into cli.rs and main.rs
- Fixes cross-agent conflicts (borrow checker, API mismatches)
- Runs verification (`cargo check`, `cargo test`, smoke tests)
- Manages agent lifecycle (spawn, monitor, shutdown)

**Builder Teammates (Sonnet 4.5)**: The implementers. Each one:
- Receives a focused prompt with exact file paths and requirements
- Reads existing code to understand interfaces
- Writes implementation code with unit tests
- Runs `cargo check` to verify compilation
- Reports back with a summary of what was built and test results

---

## Coordination Patterns

### Pattern 1: Stub-First Scaffolding

The team lead always creates interface stubs before spawning agents:

```
Lead creates:                  Agents implement:
  trait FileScanner {...}  -->   struct RegexScanner (implements FileScanner)
  trait OutputFormatter {...} -> struct TextFormatter (implements OutputFormatter)
  struct TodoItem {...}    -->   All agents use TodoItem
  struct ScanResult {...}  -->   Orchestrator returns ScanResult
```

This ensures:
- Agents can `use crate::model::TodoItem` without compilation errors
- Agents agree on shared interfaces without communicating with each other
- The project compiles at every stage (stubs -> partial -> complete)

### Pattern 2: File Ownership Boundaries

Each agent owns specific files. No two agents write to the same file simultaneously:

```
Phase 1:
  scanner-agent  owns: scanner/languages.rs, scanner/regex.rs
  discovery-agent owns: discovery.rs, scanner/mod.rs
  output-agent   owns: output/text.rs, output/mod.rs
  lead           owns: cli.rs, main.rs, error.rs, model.rs, lib.rs
```

When agents need to modify shared files (cli.rs, main.rs), the team lead does
the integration work after agents complete.

### Pattern 3: Phase Overlap

The lead identifies phases with no code dependencies and runs them in parallel:

```
Phase 3 (Git) + Phase 4 (CI):
  Both depend on Phase 2 output, but NOT on each other.
  policy-agent doesn't import from git/, blame-agent doesn't import from policy.rs.
  --> Run all 3 agents simultaneously.

Phase 5 (Cache) + Phase 7 (Distribution):
  Cache is Rust code, distribution is YAML/Docker files.
  Zero overlap in file space.
  --> Run both agents simultaneously.
```

### Pattern 4: Background Task for Single-File Work

For Phase 6 (tree-sitter), instead of a full teammate, the lead used a lightweight
background task agent:
- No team coordination overhead
- Runs in background while lead does other integration work
- Reports result via task completion notification
- Lead checks output file when ready

### Pattern 5: Graceful Lifecycle Management

```
1. spawnTeam("todo-tracker-build")     -- Creates team + task list
2. Task(team_name, name: "agent-x")    -- Spawns agent into team
3. Agent works... sends idle notification when done
4. SendMessage(type: "shutdown_request") -- Request graceful shutdown
5. Agent responds with shutdown_approved
6. System sends teammate_terminated
7. Teammate.cleanup()                   -- Remove team directories
```

---

## Conflict Resolution

### Conflict 1: Multiple Agents Editing cli.rs and main.rs

**Problem**: In Phase 3+4, three agents (blame-agent, diff-agent, policy-agent) all
needed to add CLI commands and handler functions. They each wrote their additions
to cli.rs and main.rs, but could only see their own version.

**Resolution**: The team lead provided each agent with the current state of cli.rs
in their prompt. The diff-agent (completing last in its Phase 3 batch) noticed
borrow-checker issues from other agents' match arms and proactively fixed them
by adding `ref` keywords. The team lead verified the final integrated state.

### Conflict 2: enrich_with_blame Return Type Mismatch

**Problem**: blame-agent wrote `enrich_with_blame()` returning `()` (void), but
the policy-agent's main.rs code called it with `.map_err(...)` expecting a `Result`.

**Resolution**: The blame-agent auto-fixed this when it noticed the compilation
error, removing the `.map_err()` call and using the function as a void operation.

### Conflict 3: File Modified Since Read

**Problem**: Team lead tried to write to main.rs but got a "file modified since read"
error because an agent had just modified it.

**Resolution**: Re-read the file, found the agent had already applied the needed fix.
No additional changes required.

### Conflict 4: tree-sitter API Mismatch

**Problem**: Background task agent used `for match_ in matches` (standard Iterator),
but tree-sitter v0.24 uses StreamingIterator.

**Resolution**: Team lead fixed the import and loop pattern after the agent completed.
This is a common pattern -- agents work with familiar APIs, and the lead handles
version-specific quirks.

---

## Final Statistics

### Code Output

| Metric | Value |
|--------|-------|
| Total Rust source lines | 5,759 |
| Total files (source + config + tests) | 50 |
| Test fixture files | 12 |
| CI/CD workflow files | 2 |
| Cargo dependencies | 17 (+ 9 optional for tree-sitter) |

### Test Coverage

| Category | Count |
|----------|-------|
| Unit tests (default features) | 141 |
| Unit tests (with `precise` feature) | 164 |
| Integration tests (CLI) | 9 |
| **Total (with all features)** | **173** |

### Agent Productivity

| Metric | Value |
|--------|-------|
| Total agents spawned | 12 (11 teammates + 1 background task) |
| Total agent-written lines | ~4,194 |
| Team lead-written lines | ~1,565 |
| Average lines per agent | ~350 |
| Most productive agent | policy-agent (717 lines, 29 tests) |
| Fastest agent | output-agent (~5 min) |
| Longest-running agent | blame-agent (~17 min) |

### Timing

| Milestone | Time (UTC) | Elapsed |
|-----------|-----------|---------|
| Session start | ~01:38 | 0:00 |
| Phase 1 agents done | ~01:50 | 0:12 |
| Phase 1 integrated | ~02:00 | 0:22 |
| Phase 2 agents done | ~02:08 | 0:30 |
| Phase 2 integrated | ~02:10 | 0:32 |
| Phase 3+4 agents done | ~02:28 | 0:50 |
| Phase 3+4 integrated | ~02:35 | 0:57 |
| Phase 5+7 agents done | ~02:42 | 1:04 |
| Phase 5+6+7 integrated | ~02:48 | 1:10 |
| All tests pass | ~02:50 | 1:12 |
| Git commit | ~02:57 | 1:19 |

**Total wall-clock time: ~79 minutes**

### Parallel Speedup Estimate

If done sequentially (one agent at a time, same speed):
- Phase 1: 3 agents x 8 min avg = 24 min -> done in 8 min (3x speedup)
- Phase 2: 3 agents x 6 min avg = 18 min -> done in 8 min (2.25x speedup)
- Phase 3+4: 3 agents x 17 min avg = 51 min -> done in 17 min (3x speedup)
- Phase 5+7: 2 agents x 7 min avg = 14 min -> done in 7 min (2x speedup)

**Estimated sequential time: ~150 min. Actual: ~79 min. Parallel speedup: ~1.9x**

The speedup is less than theoretical maximum because:
- Integration/wiring time between phases is sequential
- Some agents finish faster than others (idle time waste)
- Phase overlap (3+4, 5+7) partially compensates

---

## Key Takeaways

### For Users/Developers

1. **Agent Teams shine for modular projects**. If your codebase has clear module
   boundaries (separate files, well-defined interfaces), agents can work in parallel
   with minimal conflicts.

2. **The team lead does less coding, more orchestrating**. The lead wrote ~27% of
   the code but spent most time on scaffolding, integration, and verification.
   Think of it as a senior engineer doing code review and merge resolution while
   juniors implement features.

3. **Stubs are essential**. Always create interfaces/types before spawning agents.
   Without shared type definitions, agents would produce incompatible code.

4. **Detailed prompts pay off**. Each agent prompt included exact file paths,
   interface definitions, data types, and testing requirements. Vague prompts
   lead to agents making conflicting assumptions.

5. **Phase overlap is free parallelism**. Look for phases that don't depend on
   each other and run them simultaneously.

6. **Expect minor conflicts**. Multiple agents editing the same file (even at
   different times) creates integration work. Plan for 10-15 min of wiring per phase.

7. **Background tasks for simple work**. Not everything needs a full teammate.
   Single-file tasks can use lightweight background task agents.

### Architecture Principles for Agent-Friendly Code

- **One module, one owner**: Each file has exactly one agent responsible for it
- **Shared types in a central model**: All agents import from the same `model.rs`
- **Trait-based interfaces**: `FileScanner`, `OutputFormatter` define contracts
  agents implement independently
- **Dependency injection**: `ScanOrchestrator::new(Box<dyn FileScanner>, FileDiscovery)`
  lets any scanner plug in without the orchestrator knowing the implementation
- **Integration at the edges**: main.rs and cli.rs are the only files that touch
  every module -- the team lead owns these

---

*Document generated from a real Claude Code session on 2026-02-06.*
*Team: todo-tracker-build | Lead: Claude Opus 4.6 | Agents: 11 x Claude Sonnet 4.5*
*Total output: 50 files, 7,521 lines, 173 tests, 79 minutes wall-clock time.*
