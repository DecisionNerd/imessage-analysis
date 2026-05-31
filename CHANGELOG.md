# Changelog

All notable changes to this project will be documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.4] — 2026-05-30

### Changed
- Homebrew formula now downloads a pre-built binary tarball — `brew install` no longer requires Rust and completes in seconds

---

## [0.1.3] — 2026-05-30

### Added
- `imessage_analysis.__version__` — exposes the installed package version

---

## [0.1.2] — 2026-05-30

### Fixed
- `contact_stats()` now accepts a `limit` parameter (previously missing from Python binding)
- `contact_stats()` date span calculation now works across all DataFusion versions — replaced `DATEDIFF()` with portable epoch integer arithmetic

---

## [0.1.1] — 2026-05-30

### Fixed
- Python wheel now uses abi3 stable ABI (`cp311-abi3`), compatible with Python 3.11+ instead of only the specific Python version used to build it

---

## [0.1.0] — 2026-05-30

First public release. Ground-up Rust rewrite of the original Python analysis notebooks.

### Added

**Core**
- Extract iMessage history from `~/Library/Messages/chat.db` into Parquet via Apache DataFusion
- Incremental sync — only new messages are processed on subsequent runs, using a ROWID watermark
- Contact name resolution from macOS Contacts.app with automatic permission prompting
- TOML override file for contacts not in Contacts.app (`--contacts`)
- Dual-format Apple timestamp handling (nanosecond and legacy second formats)
- Duplicate-row prevention for messages appearing in multiple chats

**CLI** (`imessage-analysis`)
- `sync` — full build on first run, incremental update after that; shows row-count progress during initial build
- `status` — dataset info including message count, last sync time, size, and contacts resolution status
- `query` — arbitrary SQL against the messages table
- `search-contacts` — find contacts by name, phone, or email
- `top-contacts` — most-messaged contacts with sent/received filtering
- `time-series` — daily message counts with rolling average
- `reactions` — reaction type breakdown
- `effects` — message effect breakdown
- `links` — top shared link domains
- `seasonality` — messages by day-of-week or month-of-year
- `contact-stats` — per-contact totals, first/last date, active days
- `completions` — shell completions for zsh, bash, fish
- Table, JSON, and CSV output formats

**MCP server** (`imessage-mcp`)
- All CLI analyses exposed as MCP tools for AI agents
- Compatible with Claude Code, Codex, Cursor, Claude Desktop, and any MCP-capable client
- `size_formatted` field in status response
- `sync --force` to rebuild from scratch (e.g. after granting Contacts access)

**Skills** (via `npx skills add DecisionNerd/imessage-analysis`)
- `imessage-analysis-install` — guided setup for any AI agent
- `imessage-analysis-status` — check dataset freshness
- `contact-deep-dive` — full relationship analysis for a specific contact
- `compare-contacts` — side-by-side comparison of two contacts
- `needs-reply` — conversations waiting on a reply from you
- `period-in-review` — messaging summary for today, this week, or this month
- `query-messages` — general-purpose message querying guidance
- `group-chats` — group chat activity analysis
- `compare-periods` — messaging comparison between two time windows
- `sync` — sync guidance and freshness check

**Python package** (`pip install imessage-analysis`)
- Query-only: all analysis functions return `pyarrow.Table`
- `sync()`, `run_etl()`, `refresh()` raise a clear error directing users to the CLI

**Distribution**
- Homebrew tap: `brew tap DecisionNerd/tap && brew install imessage-analysis`
- Homebrew formula signs the binary with Contacts entitlement automatically
- `justfile` with `just install`, `just setup`, `just register` recipes for source builds
- `scripts/install.sh` — curl-pipe installer that auto-detects and registers with all supported AI clients
- `.cursor/mcp.json` — project-level Cursor MCP config

### Security
- LIKE injection prevention in `search_contacts`: `%`, `_`, and `\` are escaped with `ESCAPE '\\'`
- `direction` parameter uses a typed `Direction` enum rather than raw SQL fragments

---

[0.1.0]: https://github.com/DecisionNerd/imessage-analysis/releases/tag/v0.1.0
