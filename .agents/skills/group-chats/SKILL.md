---
name: group-chats
description: Analyse your group chat activity — most active groups, top talkers per group, and activity over time.
---

# Group Chats

Explore your group messaging activity: which groups are most active, who dominates each chat, and how group messaging trends over time.

## When to Use

- "What are my most active group chats?"
- "Who talks most in [group chat name]?"
- "Show me my group chat activity"
- "How much of my messaging is in group chats?"
- "Which group chats have I been in lately?"

## Steps

### 1 — Sync check

```
status()
```

Sync if stale (last sync more than 6 hours ago).

### 2 — Top group chats by message count

```sql
SELECT
  chat_id,
  chat_members_contact_info,
  COUNT(*) AS total_messages,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received,
  MIN(CAST(date AS VARCHAR)) AS first_message,
  MAX(CAST(date AS VARCHAR)) AS last_message,
  chat_size
FROM messages
WHERE chat_size > 1
GROUP BY chat_id, chat_members_contact_info, chat_size
ORDER BY total_messages DESC
LIMIT 10
```

This gives you the top 10 group chats. Use `chat_id` and `chat_members_contact_info` to identify each group — iMessage doesn't always surface a group name.

### 3 — Most active participants per chat

To find who sends the most messages in a specific group chat (replace `CHAT_ID` with the value from step 2):

```sql
SELECT
  name,
  contact_info,
  COUNT(*) AS messages,
  ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER (), 1) AS pct_of_chat
FROM messages
WHERE chat_id = CHAT_ID
  AND is_from_me = 0
  AND (name IS NOT NULL OR contact_info IS NOT NULL)
GROUP BY name, contact_info
ORDER BY messages DESC
LIMIT 15
```

Run this for each top group to understand who drives each conversation. If the user asks about a specific group by name or participant, use `search_contacts` first to resolve the contact identifier, then cross-reference with `chat_members_contact_info`.

### 4 — Your participation in each group

How much you contribute vs. receive across all group chats:

```sql
SELECT
  chat_id,
  chat_size,
  COUNT(*) AS total_messages,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS i_sent,
  ROUND(100.0 * SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) / COUNT(*), 1) AS pct_i_sent
FROM messages
WHERE chat_size > 1
GROUP BY chat_id, chat_size
ORDER BY total_messages DESC
LIMIT 10
```

A low `pct_i_sent` means you're mostly a reader; high means you're a driver.

### 5 — Group chat activity over time

How group messaging volume has changed month-by-month:

```sql
SELECT
  year,
  month,
  COUNT(*) AS messages,
  COUNT(DISTINCT chat_id) AS active_groups
FROM messages
WHERE chat_size > 1
GROUP BY year, month
ORDER BY year, month
```

For a single group's activity over time (replace `CHAT_ID`):

```sql
SELECT
  CAST(date AS VARCHAR) AS day,
  COUNT(*) AS messages
FROM messages
WHERE chat_id = CHAT_ID
GROUP BY date
ORDER BY date
```

### 6 — Overall group vs. direct split

Quick summary to anchor the narrative:

```sql
SELECT
  CASE WHEN chat_size > 1 THEN 'group' ELSE 'direct' END AS type,
  COUNT(*) AS messages,
  ROUND(100.0 * COUNT(*) / SUM(COUNT(*)) OVER (), 1) AS pct
FROM messages
GROUP BY type
ORDER BY messages DESC
```

## Synthesise

Lead with the top group by volume and how it compares to the others. Then cover:

- **Top group** — total messages, how long it has been active, who talks most
- **Your role** — are you a talker or a lurker in each group?
- **Trend** — has group chat activity grown or shrunk over the years?
- **Split** — what fraction of all your messaging is in groups vs. 1-on-1?

If the user asked about a specific group, focus entirely on that group: participant breakdown, your contribution, activity trend, and the most recent activity.

End by offering to drill into a specific group or compare it with a 1-on-1 relationship.
