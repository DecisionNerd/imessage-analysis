---
name: imessage-analysis
description: Install and configure the imessage-analysis MCP server so AI agents can query your Mac iMessage history.
---

# imessage-analysis

Install the `imessage-mcp` server so Claude and other agents can query your iMessage history directly.

## When to Use

Invoke this skill when the user wants to:
- Install the imessage-analysis MCP server for the first time
- Re-configure or repair the MCP server connection
- Understand what the server provides

## Steps

### 1 — Install the binary

Check whether `imessage-mcp` is already installed:

```sh
which imessage-mcp
```

If not found, install via Homebrew:

```sh
brew tap DecisionNerd/tap
brew install imessage-analysis
```

If Homebrew is not available, build from source:

```sh
git clone https://github.com/DecisionNerd/imessage-analysis
cd imessage-analysis
cargo build --release --locked
cp target/release/imessage-mcp /usr/local/bin/
cp target/release/imessage-analysis /usr/local/bin/
```

### 2 — Grant Full Disk Access

The ETL pipeline reads `~/Library/Messages/chat.db`, which requires Full Disk Access.

Prompt the user:
> Open **System Settings → Privacy & Security → Full Disk Access** and add your terminal application (Terminal, iTerm2, Ghostty, etc.). Re-launch the terminal afterwards.

### 3 — Register the MCP server

```sh
claude mcp add imessage-analysis -- imessage-mcp
```

This registers `imessage-mcp` as a stdio MCP server named `imessage-analysis` in Claude Code.

### 4 — Build the dataset

```sh
imessage-analysis sync
```

This reads `chat.db` and writes `~/.imessage-analysis/messages.parquet`. It takes a few seconds for a typical chat history.

### 5 — Verify

Tell the user the server is ready. They can now ask things like:
- *"Who do I text the most?"*
- *"How many messages did I send in 2024?"*
- *"Show me my messaging trends over the last year."*
- *"What links have I shared most often?"*

To refresh the dataset with new messages at any time:

```sh
imessage-analysis refresh
```

## Notes

- The MCP server is registered at the **user scope** (`-s user`) by default, making it available across all projects. Add `-s local` to limit it to the current project only.
- The binary name is `imessage-mcp`; the registered server name is `imessage-analysis`.
- Re-running this skill is safe — `claude mcp add` will update an existing registration.
