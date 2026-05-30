---
name: compare-periods
description: Compare your messaging activity between two time periods — volume, contacts, and trends side by side.
---

# Compare Periods

Compare your messaging activity across two time windows to understand how your communication habits have changed.

## When to Use

- "How does this month compare to last month?"
- "Was I more active in 2024 or 2023?"
- "Compare my messaging this year vs. last year"
- "How has my texting changed since [date]?"
- "Am I messaging more or less lately?"

## Determine the two periods

If the user specifies both periods: use them.
If they name one: infer the natural comparison period (e.g. "this month" implies vs. last month).

| User says | Period A | Period B |
|---|---|---|
| "this month vs last month" | 1st of current month → today | 1st of previous month → last day of previous month |
| "this year vs last year" | Jan 1 current year → today | Jan 1–Dec 31 previous year (or same span) |
| "2024 vs 2023" | full calendar year 2024 | full calendar year 2023 |
| "last 30 days vs 30 before that" | today-30d → today | today-60d → today-31d |

Always anchor the comparison to equal-length windows when possible — e.g. if Period A is a partial month, compare the same number of days in Period B.

## Steps

### 1 — Sync check

```
status()
```

Sync if stale.

### 2 — Volume and balance for each period

Run once per period, replacing `START_A`/`END_A` and `START_B`/`END_B`:

```sql
-- Period A
SELECT
  'A' AS period,
  COUNT(*) AS total_messages,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received,
  COUNT(DISTINCT CAST(date AS VARCHAR)) AS active_days,
  COUNT(DISTINCT name) AS distinct_contacts
FROM messages
WHERE date BETWEEN 'START_A' AND 'END_A'

UNION ALL

-- Period B
SELECT
  'B' AS period,
  COUNT(*) AS total_messages,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received,
  COUNT(DISTINCT CAST(date AS VARCHAR)) AS active_days,
  COUNT(DISTINCT name) AS distinct_contacts
FROM messages
WHERE date BETWEEN 'START_B' AND 'END_B'
```

### 3 — Top contacts for each period

```sql
-- Period A top contacts
SELECT 'A' AS period, name, COUNT(*) AS messages
FROM messages
WHERE date BETWEEN 'START_A' AND 'END_A'
  AND name IS NOT NULL
  AND chat_size = 1
GROUP BY name
ORDER BY messages DESC
LIMIT 10
```

```sql
-- Period B top contacts
SELECT 'B' AS period, name, COUNT(*) AS messages
FROM messages
WHERE date BETWEEN 'START_B' AND 'END_B'
  AND name IS NOT NULL
  AND chat_size = 1
GROUP BY name
ORDER BY messages DESC
LIMIT 10
```

Use these to identify contacts that appeared or disappeared between periods, and who grew or shrank.

### 4 — Contact-level shift

Identify which contacts changed most between the two periods:

```sql
WITH a AS (
  SELECT name, COUNT(*) AS n_a
  FROM messages
  WHERE date BETWEEN 'START_A' AND 'END_A'
    AND name IS NOT NULL AND chat_size = 1
  GROUP BY name
),
b AS (
  SELECT name, COUNT(*) AS n_b
  FROM messages
  WHERE date BETWEEN 'START_B' AND 'END_B'
    AND name IS NOT NULL AND chat_size = 1
  GROUP BY name
)
SELECT
  COALESCE(a.name, b.name) AS name,
  COALESCE(a.n_a, 0) AS period_a,
  COALESCE(b.n_b, 0) AS period_b,
  COALESCE(a.n_a, 0) - COALESCE(b.n_b, 0) AS delta
FROM a
FULL OUTER JOIN b ON a.name = b.name
ORDER BY ABS(COALESCE(a.n_a, 0) - COALESCE(b.n_b, 0)) DESC
LIMIT 15
```

### 5 — Day-by-day volume within each period

For shorter periods (weeks, months) — helps spot patterns:

```sql
SELECT
  CAST(date AS VARCHAR) AS day,
  SUM(CASE WHEN date BETWEEN 'START_A' AND 'END_A' THEN 1 ELSE 0 END) AS period_a,
  SUM(CASE WHEN date BETWEEN 'START_B' AND 'END_B' THEN 1 ELSE 0 END) AS period_b
FROM messages
WHERE date BETWEEN 'START_B' AND 'END_A'
GROUP BY date
ORDER BY date
```

For longer periods (years), aggregate by month instead:

```sql
SELECT
  year,
  month,
  COUNT(*) AS messages
FROM messages
WHERE date BETWEEN 'START_B' AND 'END_A'
GROUP BY year, month
ORDER BY year, month
```

### 6 — Reactions comparison

```sql
SELECT
  reaction,
  SUM(CASE WHEN date BETWEEN 'START_A' AND 'END_A' THEN 1 ELSE 0 END) AS period_a,
  SUM(CASE WHEN date BETWEEN 'START_B' AND 'END_B' THEN 1 ELSE 0 END) AS period_b
FROM messages
WHERE reaction != 'no-reaction'
  AND date BETWEEN 'START_B' AND 'END_A'
GROUP BY reaction
ORDER BY period_a DESC
LIMIT 8
```

## Synthesise

Lead with the headline number — did volume go up or down, by how much, and is that meaningful? Then work through:

- **Volume change** — total messages, percentage change, and whether the window lengths are comparable
- **Sent vs received balance** — did the ratio shift? (More sent = you were reaching out more; more received = others were reaching out)
- **Contact churn** — who appeared, who disappeared, who grew significantly
- **Biggest mover** — the single contact with the largest delta
- **Reactions and tone** — any shift in reaction usage as a proxy for conversational warmth
- **One interpretation** — what does this actually mean? ("You were more socially active in Q1" / "Messaging dropped because you stopped talking to X")

End by offering to zoom into a specific contact or drill into a shorter sub-period.
