# iMessage Analysis

[![skills.sh](https://skills.sh/b/DecisionNerd/imessage-analysis)](https://skills.sh/DecisionNerd/imessage-analysis)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

Extract, query, and analyse your Mac iMessage history from the command line or from Python. Built in Rust with Apache DataFusion — fast enough to handle years of chat history in seconds.

## Installation

### Claude Code / AI agents (one command)

```sh
npx skills add DecisionNerd/imessage-analysis
```

This installs the `imessage-analysis` skill into Claude Code. Running `/imessage-analysis` will walk you through installing the binary, granting permissions, registering the MCP server, and building the initial dataset.

### Homebrew (recommended)

```sh
brew tap DecisionNerd/tap
brew install imessage-analysis
```

This installs two binaries:
- `imessage-analysis` — the CLI
- `imessage-mcp` — the MCP server for AI agents

### From source

```sh
git clone https://github.com/DecisionNerd/imessage-analysis
cd imessage-analysis
cargo build --release --locked
# binaries at target/release/imessage-analysis and target/release/imessage-mcp
```

### Python package (PyPI)

```sh
pip install imessage-analysis
```

## Quick start

Grant Terminal **Full Disk Access** first (System Settings → Privacy & Security → Full Disk Access), then:

```sh
# Extract your messages to ~/.imessage-analysis/messages.parquet
imessage-analysis etl

# Analyse
imessage-analysis top-contacts
imessage-analysis time-series --window 28
imessage-analysis reactions
imessage-analysis query "SELECT year, COUNT(*) FROM messages GROUP BY year ORDER BY year"
```

## Requirements

- macOS (the iMessage database is Mac-only)
- For source builds: Rust 1.70+
- For the Python package: Python 3.11+

## Documentation

Full documentation is in the [`docs/`](docs/) directory:

- [Installation](docs/installation.md)
- [CLI reference](docs/cli.md)
- [MCP server](docs/mcp.md)
- [Python package](docs/python.md)
- [Data model](docs/data-model.md)
- [Architecture](docs/architecture.md)

## Attribution

This project was inspired by and builds upon the foundational work of
[Yorgos Askalidis](https://medium.com/@yaskalidis), who first documented how to access and
analyse the macOS iMessage database:

- [Here's How You Can Access Your Entire iMessage History on Your Mac](https://medium.com/@yaskalidis/heres-how-you-can-access-your-entire-imessage-history-on-your-mac-f8878276c6e9)
- [Fun Things You Can Learn About Yourself From Your Messages](https://medium.com/@yaskalidis/fun-things-you-can-learn-about-yourself-and-from-your-messages-5101631a8e20)

The original Python implementation is at [github.com/yoasaaa/imessage-analysis](https://github.com/yoasaaa/imessage-analysis).
This Rust rewrite is a separate, ground-up implementation that extends the original concept
with a native CLI, MCP server, and Python bindings.

## License

Copyright (C) 2024 David Spencer

This program is free software: you can redistribute it and/or modify it under the terms of
the [GNU General Public License v3.0](LICENSE) as published by the Free Software Foundation.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
