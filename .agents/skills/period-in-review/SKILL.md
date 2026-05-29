---
name: period-in-review
description: Summarise your iMessage activity for a recent period — today, this week, or this month. Defaults to the current week.
---

# Period in Review

Give the user a concise summary of their messaging activity for a recent period.

## When to Use

- "How was my messaging today?"
- "Give me a summary of this week's messages"
- "What did my messaging look like this month?"
- "Who have I been talking to lately?"
- "Messaging recap"
- Any question about recent activity without specifying a person

## Determine the period

If the user specifies: use it.
If not: default to **this week** (Monday through today).

| User says | Period |
|---|---|
| "today" | current calendar day |
| "yesterday" | previous calendar day |
| "this week" / default | Monday of current week through today |
| "last week" | previous Monday–Sunday |
| "this month" | 1st of current month through today |
| "last month" | previous calendar month |

## Steps

### 1 — Sync check

```
status()
```

If `last_sync` is more than 6 hours old, offer to sync first (or sync automatically if it seems stale).

### 2 — Run queries for the period

Replace `START_DATE` and `END_DATE` with `YYYY-MM-DD` strings for the period.

**Volume and balance**
```sql
SELECT
  COUNT(*) AS total_messages,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received
FROM messages
WHERE date BETWEEN 'START_DATE' AND 'END_DATE'
```

**Active contacts (1-on-1)**
```sql
SELECT
  name,
  COUNT(*) AS messages,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received,
  MAX(CAST(timestamp AS VARCHAR)) AS last_message
FROM messages
WHERE date BETWEEN 'START_DATE' AND 'END_DATE'
  AND name IS NOT NULL
  AND chat_size = 1
GROUP BY name
ORDER BY messages DESC
LIMIT 10
```

**Day-by-day breakdown** (week and month only — skip for single day)
```sql
SELECT
  CAST(date AS VARCHAR) AS day,
  COUNT(*) AS total,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received
FROM messages
WHERE date BETWEEN 'START_DATE' AND 'END_DATE'
GROUP BY date
ORDER BY date
```

**Reactions exchanged**
```sql
SELECT reaction, COUNT(*) AS n
FROM messages
WHERE date BETWEEN 'START_DATE' AND 'END_DATE'
  AND reaction != 'no-reaction'
GROUP BY reaction
ORDER BY n DESC
LIMIT 5
```

**Links shared**
```sql
SELECT link_domain, COUNT(*) AS n
FROM messages
WHERE date BETWEEN 'START_DATE' AND 'END_DATE'
  AND link_domain IS NOT NULL
GROUP BY link_domain
ORDER BY n DESC
LIMIT 5
```

**Group chat activity** (week and month only)
```sql
SELECT
  COUNT(*) AS messages,
  COUNT(DISTINCT chat_id) AS group_chats
FROM messages
WHERE date BETWEEN 'START_DATE' AND 'END_DATE'
  AND chat_size > 1
```

### 3 — Synthesise

Tailor the length to the period — a day recap should be a few sentences; a month can be a paragraph or two.

**For a day:**
- Total messages sent/received
- Who you talked to (top 3)
- Any reactions or links worth noting
- One-line summary: "A quiet day" / "Active — mostly with [name]"

**For a week:**
- Total volume and sent/received balance
- Top 3 contacts with brief characterisation ("mostly back-and-forth with Alice")
- Busiest day
- Any standout content (reactions, links)
- How it compares to a typical week (use a quick query if needed: 7-day rolling avg from `time_series`)

**For a month:**
- Total volume, YoY or MoM if interesting
- Top 5 contacts
- Weekly rhythm — were some weeks much heavier?
- Reactions and links
- One insight that stands out

Always end with an offer to drill deeper: "Want a full breakdown of [top contact] or a look at a specific day?"
