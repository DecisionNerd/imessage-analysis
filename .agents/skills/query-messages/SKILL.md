---
name: query-messages
description: Use the imessage-analysis MCP server to answer questions about the user's iMessage history.
---

# Query iMessage History

Use the `imessage-analysis` MCP tools to answer questions about the user's messages.

## When to Use

When the user asks anything about their message history:
- "Who do I text the most?"
- "How many messages did I send last year?"
- "What reactions do I use?"
- "Show me my texting trends"
- "How often do I text [person]?"
- "What links have I shared?"

## Workflow

### 1 — Check freshness first

Always call `status` before analysing. If `synced` is false or `last_sync` is stale (more than a day old), call `sync` first.

```
status → { synced: true, total_messages: 374804, last_sync: "2026-05-29T..." }
```

### 2 — Find the right contact name

When the user mentions a person, use `search_contacts` to find the exact name string before filtering by it. Never guess.

```
search_contacts({ query: "alice" })
→ [{ name: "Alice Smith", contact_info: "+14155550001", message_count: 4821 }]
```

Use the `name` value exactly as returned in subsequent tool calls.

### 3 — Answer with the right tool

| Question | Tool |
|---|---|
| Who do I text most? | `top_contacts` |
| How often do I text someone? | `time_series` with `contact` |
| How have my messaging habits changed? | `time_series` |
| What reactions do I use/receive? | `reactions` |
| What message effects have I sent? | `effects` |
| What links have I shared? | `links` |
| Messages by day of week / time of year | `seasonality` |
| Stats for a specific person | `contact_stats` with `contact` |
| Anything else | `query` with SQL |

### 4 — Be specific with filters

Use `sent: true` or `received: true` when the question is directional.
Use `year` to scope to a calendar year.
Use `direct_only: true` to exclude group chats.

### 5 — Present results clearly

- Summarise the key insight in one sentence before showing numbers
- For time series, describe the trend (increasing, seasonal, etc.)
- For top contacts, put the most interesting finding first
- Offer to drill deeper: "Want me to show the trend for [top contact]?"

## Available Tools

- `status` — check if data is current
- `sync` — update the dataset
- `search_contacts` — find exact contact names
- `top_contacts` — most-messaged people
- `time_series` — daily message counts over time
- `reactions` — reaction type breakdown
- `effects` — message effect breakdown
- `links` — top shared domains
- `seasonality` — patterns by day-of-week or month
- `contact_stats` — per-contact totals, dates, frequency
- `query` — arbitrary SQL for anything else

## Example SQL Patterns

Use `body_text` for message-body analysis, search, topic summaries, and NLP. `text` is the raw SQLite `message.text` value, `inferred_text` is the decoded `attributedBody` fallback, and `text_combined` is kept as a legacy compatibility alias.

```sql
-- Messages per year
SELECT year, COUNT(*) AS n FROM messages GROUP BY year ORDER BY year

-- Search message body text
SELECT timestamp, name, is_from_me, body_text
FROM messages
WHERE body_text ILIKE '%dinner%'
ORDER BY timestamp DESC
LIMIT 20

-- Most active months
SELECT year, month, COUNT(*) AS n FROM messages
GROUP BY year, month ORDER BY n DESC LIMIT 10

-- Group chats only
SELECT name, COUNT(*) AS n FROM messages
WHERE chat_size > 1 AND name IS NOT NULL
GROUP BY name ORDER BY n DESC LIMIT 10

-- First message with someone
SELECT name, MIN(CAST(date AS VARCHAR)) AS first_message
FROM messages WHERE name IS NOT NULL
GROUP BY name ORDER BY first_message LIMIT 20
```
