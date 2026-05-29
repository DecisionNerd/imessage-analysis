# iMessage Analysis

[![CI](https://github.com/DecisionNerd/imessage-analysis/actions/workflows/ci.yml/badge.svg)](https://github.com/DecisionNerd/imessage-analysis/actions/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![skills.sh](https://skills.sh/b/DecisionNerd/imessage-analysis)](https://skills.sh/DecisionNerd/imessage-analysis)
[![PyPI](https://img.shields.io/pypi/v/imessage-analysis)](https://pypi.org/project/imessage-analysis/)
[![Homebrew](https://img.shields.io/badge/homebrew-DecisionNerd%2Ftap-orange)](https://github.com/DecisionNerd/homebrew-tap)
[![macOS](https://img.shields.io/badge/platform-macOS-lightgrey)](https://www.apple.com/macos/)

Query and analyse your entire Mac iMessage history — from the terminal, an AI agent, or a Python notebook. Extracts Apple's SQLite database into Parquet, then runs fast columnar queries via Apache DataFusion. Built in Rust.

---

## Features

- **CLI** — ETL, ad-hoc SQL, and built-in analyses (top contacts, reactions, time series, seasonality)
- **MCP server** — expose your message data as tools for Claude and other AI agents
- **Python package** — returns `pyarrow.Table` for seamless pandas / notebook integration
- **Incremental refresh** — only processes new messages since the last run
- **Contact resolution** — auto-resolves names from macOS Contacts.app, with TOML overrides
- **Fast** — handles 500K+ messages in seconds via vectorised columnar execution

---

## Quickstart

### With Claude Code, Codex, or Cursor

**1.** Add the skills:

```sh
npx skills add DecisionNerd/imessage-analysis
```

**2.** Run the install skill:

```
/imessage-analysis-install
```

Checks for the binary, registers the MCP server with the right command for your agent, and walks you through the first sync.

**3.** Ask anything about your messages:

> *"Who have I texted most this year?"*
> *"Give me a deep dive on Alice"*
> *"Who's been waiting on a reply from me?"*

### With any other client (Claude Desktop, ChatGPT, Windsurf…)

```sh
curl -fsSL https://raw.githubusercontent.com/DecisionNerd/imessage-analysis/main/scripts/install.sh | bash
```

Auto-detects and registers with Claude Desktop, Cursor, Claude Code, and Codex. See [MCP setup](docs/mcp.md) for manual config.

### With the CLI

**1.** Grant **Terminal Full Disk Access**:

> System Settings → Privacy & Security → Full Disk Access → enable Terminal

**2.** Install:

```sh
brew tap DecisionNerd/tap
brew install imessage-analysis
```

**3.** First sync from Apple Terminal.app (see note above):

```sh
imessage-analysis sync
```

**4.** Explore:

```sh
imessage-analysis status
imessage-analysis top-contacts --limit 10
imessage-analysis time-series --year 2024
imessage-analysis reactions --received
imessage-analysis search-contacts alice
imessage-analysis query "SELECT year, COUNT(*) AS n FROM messages GROUP BY year ORDER BY year"
```

---

## Use with Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "imessage": { "command": "imessage-mcp" }
  }
}
```

---

## Use with Python

```sh
pip install imessage-analysis
```

```python
import imessage_analysis

df = imessage_analysis.top_contacts().to_pandas()
df = imessage_analysis.query("SELECT * FROM messages WHERE year = 2024").to_pandas()
```

All query functions return `pyarrow.Table`. Call `.to_pandas()` to convert.

> **Contact names:** `imessage_analysis.sync()` picks up new messages but cannot resolve contact names — the Python interpreter doesn't have Contacts access. Run `imessage-analysis sync` from the CLI to keep names up to date.

---

## Installation

| Method | Command |
|---|---|
| Homebrew | `brew tap DecisionNerd/tap && brew install imessage-analysis` |
| Cargo | `cargo install --git https://github.com/DecisionNerd/imessage-analysis` |
| PyPI | `pip install imessage-analysis` |
| Claude Code | `npx skills add DecisionNerd/imessage-analysis` |

Requires macOS. Rust 1.70+ for source builds. Python 3.11+ for the Python package.

---

## Documentation

| | |
|---|---|
| [Installation](docs/installation.md) | Full Disk Access, Homebrew, source, Python |
| [CLI reference](docs/cli.md) | All commands and flags |
| [MCP server](docs/mcp.md) | Tool list, Claude Desktop setup |
| [Python package](docs/python.md) | API reference, notebook examples |
| [Data model](docs/data-model.md) | All 22 output columns |
| [Contact resolution](docs/contacts.md) | Contacts.app + TOML overrides |
| [Architecture](docs/architecture.md) | Crate layout, ETL vs query layers |
| [Releasing](docs/releasing.md) | Tagging, Homebrew formula update |

---

## Contributing

1. Fork the repo and create a branch
2. `cargo test --all` — all tests must pass
3. `cargo clippy -- -D warnings` and `cargo fmt`
4. Open a pull request — CI runs automatically

This project uses [Conventional Commits](https://www.conventionalcommits.org/).

---

## Attribution

Inspired by the foundational work of [Yorgos Askalidis](https://medium.com/@yaskalidis), who first documented how to access and analyse the macOS iMessage database. See his [original Python implementation](https://github.com/yoasaaa/imessage-analysis) and write-ups:

- [Accessing your iMessage history on Mac](https://medium.com/@yaskalidis/heres-how-you-can-access-your-entire-imessage-history-on-your-mac-f8878276c6e9)
- [Fun analysis ideas](https://medium.com/@yaskalidis/fun-things-you-can-learn-about-yourself-and-from-your-messages-5101631a8e20)

This is a separate, ground-up Rust rewrite that extends the concept with a native CLI, MCP server, and Python bindings.

---

## License

Copyright (C) 2024 David Spencer. Released under the [GNU General Public License v3.0](LICENSE).
