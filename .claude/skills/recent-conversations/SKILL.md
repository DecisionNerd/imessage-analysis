---
name: recent-conversations
description: Summarise your most recent message exchanges — who you've been talking to, what topics came up, and the overall tone of recent activity.
---

# Recent Conversations

Summarise what's been happening in your messages lately.

## When to Use

- "What have I been talking about recently?"
- "Who have I been messaging this week?"
- "Catch me up on my recent conversations"
- "What's been going on in my messages lately?"

## Steps

### 1 — Check and sync

```
status()
```

If `last_sync` is more than a few hours old, call `sync()` first.

### 2 — Get recent activity window

Ask the user how far back to look if not specified. Default to 7 days.

```sql
-- Most active contacts in the last 7 days
SELECT
  name,
  COUNT(*) AS messages,
  MAX(CAST(timestamp AS VARCHAR)) AS last_message,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received
FROM messages
WHERE timestamp >= CAST(date_trunc('day', now() - INTERVAL '7 days') AS TIMESTAMP)
  AND name IS NOT NULL
  AND chat_size = 1
GROUP BY name
ORDER BY last_message DESC
LIMIT 15
```

```sql
-- Recent group chat activity
SELECT
  chat_members_contact_info,
  COUNT(*) AS messages,
  MAX(CAST(timestamp AS VARCHAR)) AS last_message
FROM messages
WHERE timestamp >= CAST(date_trunc('day', now() - INTERVAL '7 days') AS TIMESTAMP)
  AND chat_size > 1
GROUP BY chat_members_contact_info
ORDER BY last_message DESC
LIMIT 5
```

```sql
-- Recent message volume by day
SELECT
  CAST(date AS VARCHAR) AS day,
  COUNT(*) AS total,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received
FROM messages
WHERE date >= date_trunc('day', now() - INTERVAL '7 days')
GROUP BY date
ORDER BY date DESC
```

### 3 — Summarise

Give a brief narrative: who's been most active, any notable conversations (high volume days, new contacts), and your overall send/receive balance for the period. Keep it concise — this is a catch-up, not a deep analysis.

Offer to do a `contact-deep-dive` on anyone who looks interesting in the results.

## Note on message content

This skill works entirely from metadata (names, counts, timestamps). It does not read the text of your messages.
