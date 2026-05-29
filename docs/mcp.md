# MCP Server

`imessage-mcp` is an MCP (Model Context Protocol) server that exposes your iMessage data to AI agents. It speaks JSON-RPC 2.0 over stdio, the standard transport for local MCP servers.

---

## Setup

### One-command (Claude Code / Codex)

```sh
npx skills add DecisionNerd/imessage-analysis
```

Then run `/imessage-analysis-install` in your agent — it handles binary install, signing, MCP registration, and first sync instructions.

### Shell script (any client)

```sh
curl -fsSL https://raw.githubusercontent.com/DecisionNerd/imessage-analysis/main/scripts/install.sh | bash
```

Detects and registers with Claude Code, Codex, Claude Desktop, and Cursor automatically.

### Manual setup by client

#### Claude Code

```sh
claude mcp add imessage-analysis $(which imessage-mcp)
```

#### Codex

```sh
codex mcp add imessage-analysis -- $(which imessage-mcp)
```

#### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "imessage-analysis": {
      "command": "imessage-mcp"
    }
  }
}
```

Restart Claude Desktop after saving.

#### Cursor

Edit `~/.cursor/mcp.json` (global) or `.cursor/mcp.json` in your project:

```json
{
  "mcpServers": {
    "imessage-analysis": {
      "command": "imessage-mcp"
    }
  }
}
```

Restart Cursor after saving. If you opened this repo in Cursor, `.cursor/mcp.json` is already included.

#### ChatGPT desktop

MCP configuration path varies by app version. Use the same `mcpServers` JSON format above and point `command` at the absolute path to `imessage-mcp`:

```sh
which imessage-mcp   # get the absolute path
```

---

## First sync

Run the first sync from **Apple Terminal.app** — not iTerm2, tmux, cmux, or other multiplexers. macOS requires a direct window-server connection to show the Contacts permission dialog.

```sh
imessage-analysis sync
```

Grant Contacts access when prompted. After that, `sync` works from any terminal. If you have an existing dataset with no contact names, rebuild it:

```sh
imessage-analysis sync --force
```

---

## Available tools

| Tool | Description |
|---|---|
| `sync` | Build dataset on first run, update incrementally after that |
| `status` | Dataset freshness, message count, size |
| `query` | Execute arbitrary SQL against the `messages` table |
| `search_contacts` | Find contacts by name, phone, or email |
| `top_contacts` | Most-messaged contacts |
| `time_series` | Daily message counts with rolling average |
| `reactions` | Reaction type breakdown |
| `effects` | Message effect breakdown |
| `links` | Top shared link domains |
| `seasonality` | Messages by day-of-week or month-of-year |
| `contact_stats` | Per-contact statistics |
| `schema` | Return the dataset schema |

## Tool parameters

All parameters are optional unless noted.

### `query`

| Parameter | Type | Description |
|---|---|---|
| `sql` *(required)* | string | SQL to execute. Table name is `messages`. |
| `limit` | integer | Max rows to return (default 100) |

### `search_contacts`

| Parameter | Type | Description |
|---|---|---|
| `query` *(required)* | string | Substring to search for in name, phone, or email (case-insensitive) |
| `limit` | integer | Max results (default 20) |

### `top_contacts`

| Parameter | Type | Description |
|---|---|---|
| `limit` | integer | Number of contacts (default 10) |
| `year` | integer | Filter to a specific year |
| `direct_only` | boolean | Only 1-on-1 chats |

### `time_series`

| Parameter | Type | Description |
|---|---|---|
| `contact` | string | Filter to a specific contact name |
| `window` | integer | Rolling average window in days (default 28) |
| `start` | string | Start date `YYYY-MM-DD` |
| `end` | string | End date `YYYY-MM-DD` |

### `reactions` / `effects`

| Parameter | Type | Description |
|---|---|---|
| `contact` | string | Filter to a specific contact |
| `year` | integer | Filter to a specific year |

### `seasonality`

| Parameter | Type | Description |
|---|---|---|
| `kind` | `"dow"` or `"month"` | Day-of-week or month-of-year (default `"dow"`) |

### `contact_stats`

| Parameter | Type | Description |
|---|---|---|
| `contact` | string | Filter to a specific contact |
| `limit` | integer | Max contacts to return (default 100) |

---

## Keeping data fresh

The server holds the dataset in memory for the lifetime of the process. After calling `sync`, the in-memory index is automatically re-initialised — you do not need to restart the server.
