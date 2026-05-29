# Installation

## Requirements

- macOS (the iMessage database only exists on Mac)
- For Homebrew / binary installs: no other dependencies
- For source builds: Rust 1.70+
- For the Python package: Python 3.11+

## Homebrew (recommended)

```sh
brew tap DecisionNerd/tap
brew install imessage-analysis
```

This installs two binaries:

| Binary | Purpose |
|---|---|
| `imessage-analysis` | CLI for ETL and analysis |
| `imessage-mcp` | MCP server for AI agents |

To upgrade later:

```sh
brew upgrade imessage-analysis
```

## From source

```sh
git clone https://github.com/DecisionNerd/imessage-analysis
cd imessage-analysis
cargo build --release --locked
```

Binaries are written to `target/release/`. You can copy them to any directory on your `$PATH`:

```sh
cp target/release/imessage-analysis target/release/imessage-mcp /usr/local/bin/
```

## Python package

```sh
pip install imessage-analysis
```

The Python package bundles the same Rust core. ETL functions require macOS; query and analysis functions work on any platform if a Parquet dataset is already present (e.g. copied from a Mac).

## Permissions

macOS requires **Full Disk Access** for any process that reads `~/Library/Messages/chat.db`.

1. Open **System Settings → Privacy & Security → Full Disk Access**
2. Click **+** and add your terminal application (Terminal, iTerm2, etc.)
3. Re-launch the terminal

Without this, `imessage-analysis sync` will fail with a permissions error.
