---
name: imessage-analysis-status
description: Show the current status of the iMessage analysis dataset — last sync time, total messages, and whether it's up to date.
---

# iMessage Analysis Status

Report the current state of the local iMessage dataset.

## When to Use

- "What's the status of my iMessage data?"
- "Is my data up to date?"
- "When was the last sync?"
- "How many messages do I have?"
- "Check the dataset"

## Steps

### 1 — Call status

```
status()
```

### 2 — Report the result

Present the key fields clearly:

| Field | What to show |
|---|---|
| `synced` | Whether the dataset is in sync with the Messages database |
| `last_sync` | Human-readable: "just now", "2 hours ago", "3 days ago" |
| `total_messages` | Formatted with thousands separator |
| `size_bytes` | Human-readable: KB, MB |

### 3 — Advise on freshness

- **Up to date** (synced + last_sync < 1 hour ago): "Your data is current."
- **Slightly stale** (last_sync 1–24 hours ago): "Last synced X hours ago — run `/sync` to refresh."
- **Stale** (last_sync > 24 hours ago or `synced: false`): Offer to sync now, or tell the user to run `sync`.

### 4 — Offer next steps

Suggest what they might want to do:
- "Want a summary of this week's messages? Try `/period-in-review`."
- "Looking for someone specific? Try `/contact-deep-dive`."
