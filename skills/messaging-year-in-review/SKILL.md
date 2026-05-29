---
name: messaging-year-in-review
description: A full year-in-review of your iMessage activity — top contacts, busiest periods, reactions, shared links, and how the year compared to previous ones.
---

# Messaging Year in Review

Generate a personal year-in-review from your message history.

## When to Use

- "Give me my messaging year in review"
- "How was my texting year in [year]?"
- "Summarise my messages for [year]"
- "Year in review for my messages"

## Steps

### 1 — Confirm the year

Ask the user which year if not specified. Default to the most recent complete year.

### 2 — Gather the data

Run all of these:

**Overall volume**
```
time_series({ year: YEAR, window: 28 })
top_contacts({ year: YEAR, limit: 10, direct_only: true })
reactions({ year: YEAR })
effects({ year: YEAR })
links({ limit: 10 })
seasonality({ kind: "month", year: YEAR })  -- note: filter by year in query
seasonality({ kind: "dow" })
```

**Year-over-year comparison**
```sql
SELECT year, COUNT(*) AS total,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received
FROM messages
GROUP BY year ORDER BY year
```

**Busiest single day**
```sql
SELECT CAST(date AS VARCHAR) AS day, COUNT(*) AS n
FROM messages WHERE year = YEAR
GROUP BY date ORDER BY n DESC LIMIT 1
```

**New contacts that year (first message ever)**
```sql
SELECT name, MIN(CAST(date AS VARCHAR)) AS first_message
FROM messages
WHERE name IS NOT NULL
GROUP BY name
HAVING MIN(year) = YEAR
ORDER BY first_message
LIMIT 10
```

**Most-used reaction**
```sql
SELECT reaction, COUNT(*) AS n
FROM messages
WHERE year = YEAR AND reaction != 'no-reaction'
GROUP BY reaction ORDER BY n DESC LIMIT 1
```

### 3 — Write the year in review

Structure it like an end-of-year wrap-up. Cover:

1. **The number** — total messages, how it compares to the year before (up/down/stable)
2. **Your top people** — top 5 contacts with total counts
3. **The busiest day** — what date had the most messages (you can speculate on the occasion)
4. **How you communicate** — sent vs received ratio, favourite reaction, any effects used
5. **What you shared** — top link domains
6. **Seasonal patterns** — busiest month, most active day of week
7. **New connections** — people you texted for the first time that year
8. **One sentence to sum it up** — a human takeaway from the data

Make it warm and personal, not a stats report.
