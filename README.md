# iMessage Analysis

[![CI](https://github.com/DecisionNerd/imessage-analysis/actions/workflows/ci.yml/badge.svg)](https://github.com/DecisionNerd/imessage-analysis/actions/workflows/ci.yml)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![skills.sh](https://skills.sh/b/DecisionNerd/imessage-analysis)](https://skills.sh/DecisionNerd/imessage-analysis)
[![PyPI](https://img.shields.io/pypi/v/imessage-analysis)](https://pypi.org/project/imessage-analysis/)
[![Homebrew](https://img.shields.io/badge/homebrew-DecisionNerd%2Ftap-orange)](https://github.com/DecisionNerd/homebrew-tap)
[![macOS](https://img.shields.io/badge/platform-macOS-lightgrey)](https://www.apple.com/macos/)

Ask questions about your entire iMessage history — from an AI agent, the terminal, or a Python notebook. Works on Mac.

---

## What it does

- **Ask your AI** — connect to Claude, Codex, Cursor, or ChatGPT and ask things like *"Who do I text most?"* or *"What did Alice and I talk about last month?"*
- **Search and query** — run any question against your messages from the terminal
- **Python notebooks** — load your message data into pandas for custom analysis
- **Always current** — syncs only new messages each run, so it stays fast
- **Real names** — reads your Contacts to show names instead of phone numbers

---

## Quickstart

### With Claude Code, Codex, or Cursor

**1.** Install the skills package:

```sh
npx skills add DecisionNerd/imessage-analysis
```

**2.** Run the setup command inside your AI tool:

```
/imessage-analysis-install
```

This installs the binary, connects it to your AI tool, and walks you through the first sync — including a note about using Apple Terminal the first time so macOS can ask for Contacts permission.

**3.** Ask anything:

> *"Who have I texted most this year?"*
> *"Give me a deep dive on Alice"*
> *"Who's been waiting on a reply from me?"*

### With Claude Desktop, ChatGPT, or any other AI tool

```sh
curl -fsSL https://raw.githubusercontent.com/DecisionNerd/imessage-analysis/main/scripts/install.sh | bash
```

Detects and connects to Claude Desktop, Cursor, Claude Code, and Codex automatically. See [MCP setup](docs/mcp.md) for manual steps.

### From the terminal only

**1.** Give Terminal permission to read your messages:

> System Settings → Privacy & Security → Full Disk Access → enable Terminal

**2.** Install:

```sh
brew tap DecisionNerd/tap
brew install imessage-analysis
```

**3.** Run your first sync from **Apple Terminal.app** (not iTerm2 or other terminals — macOS needs this to ask for Contacts permission the first time):

```sh
imessage-analysis sync
```

**4.** Start exploring:

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

> **Note:** `imessage_analysis.sync()` picks up new messages but won't resolve contact names — that requires running `imessage-analysis sync` from the terminal at least once.

---

## Installation

| Method | Command |
|---|---|
| Homebrew | `brew tap DecisionNerd/tap && brew install imessage-analysis` |
| Cargo | `cargo install --git https://github.com/DecisionNerd/imessage-analysis` |
| PyPI | `pip install imessage-analysis` |
| Claude Code / Codex / Cursor | `npx skills add DecisionNerd/imessage-analysis` |

Requires macOS. Rust 1.70+ for source builds. Python 3.11+ for the Python package.

---

## Documentation

| | |
|---|---|
| [Installation](docs/installation.md) | Full Disk Access, Homebrew, source, Python |
| [CLI reference](docs/cli.md) | All commands and flags |
| [AI agent setup](docs/mcp.md) | Connecting to Claude, Codex, Cursor, ChatGPT |
| [Python package](docs/python.md) | API reference, notebook examples |
| [Data model](docs/data-model.md) | All 22 output columns |
| [Contact resolution](docs/contacts.md) | Contacts.app + TOML overrides |
| [Architecture](docs/architecture.md) | How it works under the hood |
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

This is a separate, ground-up Rust rewrite that extends the concept with a native CLI, AI agent integration, and Python bindings.

---

## License

Copyright (C) 2026 David Spencer. Released under the [GNU General Public License v3.0](LICENSE).
