---
name: sync
description: Sync your iMessage history — builds the dataset on first run, updates incrementally after that. Use this to make sure your data is current before analysing.
---

# Sync iMessage History

Build or update your message dataset so it's ready to query.

## When to Use

- "Sync my messages"
- "Update my message history"
- "Make sure my messages are up to date"
- "Refresh my data"
- Any time `status` shows stale or missing data

## Steps

### 1 — Check current state

```
status()
```

Report what you find:
- No dataset → "No dataset found, building for the first time…"
- Dataset exists, last sync < 1 hour ago → "Already up to date (last sync: X). Nothing to do."
- Dataset exists, last sync > 1 hour ago → "Updating since last sync at X…"

### 2 — Sync if needed

```
sync()
```

This is safe to call at any time — it does nothing if already current, and only processes new messages if stale.

### 3 — Confirm

Call `status()` again and report the result:
- Total messages indexed
- Last sync timestamp
- Dataset size

## If sync fails with a permissions error

The MCP server doesn't have access to `~/Library/Messages/chat.db`. This requires Full Disk Access for the app running the MCP server (Claude Desktop, Claude Code, or your terminal).

Guide the user:

> To fix this, grant Full Disk Access to the app you're using:
> **System Settings → Privacy & Security → Full Disk Access**
> Add Claude Desktop, Claude Code, or your terminal application, then try again.

Alternatively, the user can run `imessage-analysis sync` once from a terminal that already has Full Disk Access — after that, the dataset exists and queries will work without special permissions.
