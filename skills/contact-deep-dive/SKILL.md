---
name: contact-deep-dive
description: Comprehensive analysis of your messaging relationship with a specific person — frequency, patterns, reactions, shared content, and conversation highlights.
---

# Contact Deep Dive

Build a full picture of your messaging relationship with one person.

## When to Use

When the user asks about a specific person:
- "Tell me about my texting relationship with Alice"
- "How much do I talk to [person]?"
- "Give me stats on [contact]"
- "What's my messaging history with [name]?"

## Steps

### 1 — Identify the contact

```
search_contacts({ query: "<name the user provided>" })
```

If multiple matches, ask the user to confirm which one. Use the exact `name` value returned.

### 2 — Pull all stats in parallel

Run these tools together with `contact` set to the resolved name:

- `contact_stats({ contact: NAME })` — totals, first/last date, active days, avg per day
- `time_series({ contact: NAME, window: 28 })` — trend over time
- `reactions({ contact: NAME })` — reactions exchanged
- `top_contacts({ limit: 1, direct_only: true })` — confirm rank among all contacts

Also run:
```sql
-- Sent vs received breakdown
SELECT
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received,
  ROUND(100.0 * SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) / COUNT(*), 1) AS pct_sent
FROM messages WHERE name = '<NAME>'
```

```sql
-- Most active years
SELECT year, COUNT(*) AS n FROM messages
WHERE name = '<NAME>'
GROUP BY year ORDER BY year
```

```sql
-- Day of week pattern
SELECT
  CASE EXTRACT(DOW FROM CAST(timestamp AS TIMESTAMP))
    WHEN 0 THEN 'Sunday' WHEN 1 THEN 'Monday' WHEN 2 THEN 'Tuesday'
    WHEN 3 THEN 'Wednesday' WHEN 4 THEN 'Thursday' WHEN 5 THEN 'Friday'
    WHEN 6 THEN 'Saturday'
  END AS day,
  COUNT(*) AS n
FROM messages WHERE name = '<NAME>' AND timestamp IS NOT NULL
GROUP BY EXTRACT(DOW FROM CAST(timestamp AS TIMESTAMP)), day
ORDER BY EXTRACT(DOW FROM CAST(timestamp AS TIMESTAMP))
```

### 3 — Synthesise into a narrative

Present the results as a coherent story, not a list of numbers. Structure:

**Relationship overview** — when the relationship started, total volume, how they rank among your contacts, whether you text more or they text more.

**Trajectory** — is messaging increasing, decreasing, or steady? Call out any notable spikes or gaps.

**Patterns** — most active day of week, whether it's a morning/evening relationship (if visible from data).

**Reactions** — what reactions you exchange most; who sends more Loved vs Liked etc.

**One interesting finding** — surface the most surprising or meaningful data point.

End by offering to drill into a specific year or compare to the overall average.
