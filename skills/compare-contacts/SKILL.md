---
name: compare-contacts
description: Compare your messaging patterns with two or more contacts side by side — volume, frequency, reactions, and trajectory.
---

# Compare Contacts

Compare your relationship with two or more people across key dimensions.

## When to Use

- "Compare my messaging with Alice and Bob"
- "Who do I text more — [person A] or [person B]?"
- "How do my top contacts compare?"

## Steps

### 1 — Resolve all contact names

```
search_contacts({ query: "<name 1>" })
search_contacts({ query: "<name 2>" })
```

Confirm names with the user if ambiguous.

### 2 — Pull stats for each contact

For each contact NAME_A and NAME_B, run `contact_stats` and the sent/received breakdown:

```sql
SELECT
  name,
  COUNT(*) AS total,
  SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
  SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received,
  MIN(CAST(date AS VARCHAR)) AS first_message,
  MAX(CAST(date AS VARCHAR)) AS last_message,
  COUNT(DISTINCT CAST(date AS VARCHAR)) AS active_days
FROM messages
WHERE name IN ('NAME_A', 'NAME_B')
GROUP BY name
ORDER BY total DESC
```

```sql
-- Year-by-year comparison
SELECT year, name, COUNT(*) AS n
FROM messages
WHERE name IN ('NAME_A', 'NAME_B')
GROUP BY year, name
ORDER BY year, name
```

```sql
-- Most active reaction per contact
SELECT name, reaction, COUNT(*) AS n
FROM messages
WHERE name IN ('NAME_A', 'NAME_B') AND reaction != 'no-reaction'
GROUP BY name, reaction
ORDER BY name, n DESC
```

### 3 — Present the comparison

Lead with the clearest contrast — usually total volume or trajectory. Then cover:

- **Volume** — who you message more, by how much
- **Balance** — who initiates more in each relationship (sent % vs received %)
- **Longevity** — how long each relationship spans
- **Consistency** — active days as a proxy for how regular contact is
- **Vibe** — reactions as a loose proxy for the tone of each relationship
- **Trend** — is one relationship growing while the other fades?

End with the most interesting observation — a sentence that captures what the comparison actually means.
