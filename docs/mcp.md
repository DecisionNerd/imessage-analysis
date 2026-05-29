# MCP Server

`imessage-mcp` is an MCP (Model Context Protocol) server that exposes your iMessage data to AI agents. It speaks JSON-RPC 2.0 over stdio, the standard transport for local MCP servers.

## One-command setup (Claude Code)

```sh
npx skills add DecisionNerd/imessage-analysis
```

Then run `/imessage-analysis` inside Claude Code. The skill installs the binary, guides you through granting Full Disk Access, registers the MCP server, and runs the initial ETL — all in one session.

## Setup with Claude Desktop

1. Run the ETL at least once so a dataset exists:
   ```sh
   imessage-analysis etl
   ```

2. Edit `~/Library/Application Support/Claude/claude_desktop_config.json` and add the server:
   ```json
   {
     "mcpServers": {
       "imessage": {
         "command": "imessage-mcp"
       }
     }
   }
   ```
   If `imessage-mcp` is not on your `$PATH`, use the absolute path (e.g. `/usr/local/bin/imessage-mcp` for a Homebrew install, or `~/.cargo/bin/imessage-mcp` for a source build).

3. Restart Claude Desktop.

You can now ask Claude things like:
- *"Who do I text the most?"*
- *"How many messages did I send in 2024?"*
- *"What are my most-used message reactions?"*
- *"Show me my messaging trends over the last year."*

## Available tools

| Tool | Description |
|---|---|
| `run_etl` | Run the full ETL pipeline (chat.db → Parquet) |
| `refresh` | Incremental update — only new messages since last run |
| `query` | Execute arbitrary SQL against the `messages` table |
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

## Keeping data fresh

The server holds the dataset in memory for the lifetime of the process. After calling `run_etl` or `refresh`, the in-memory index is automatically re-initialised — you do not need to restart the server.

For day-to-day use, prefer `refresh` over `run_etl`. It only processes messages since the last run and completes in milliseconds.
