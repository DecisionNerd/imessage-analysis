---
name: needs-reply
description: Find conversations where the last message was from someone else and you haven't replied — filtered to likely real contacts, sorted by how long they've been waiting.
---

# Needs Reply

Surface conversations that might be waiting on a response, filtered to real contacts and sorted by how long they've been waiting.

## When to Use

- "Who haven't I replied to?"
- "Do I owe anyone a response?"
- "What messages am I sitting on?"
- "Who's waiting to hear back from me?"

## Steps

### 1 — Sync

```
status()
```

Sync if stale. This skill is only useful on fresh data.

### 2 — Find unreplied conversations

The core query: for each contact, find the timestamp of their last message to you and your last reply to them. If their last message is more recent than your last reply (or you've never replied), they're waiting.

```sql
WITH last_received AS (
  SELECT
    name,
    contact_info,
    chat_id,
    MAX(timestamp) AS last_received_at
  FROM messages
  WHERE is_from_me = 0
    AND name IS NOT NULL
    AND chat_size = 1
  GROUP BY name, contact_info, chat_id
),
last_sent AS (
  SELECT
    chat_id,
    MAX(timestamp) AS last_sent_at
  FROM messages
  WHERE is_from_me = 1
    AND chat_size = 1
  GROUP BY chat_id
),
history AS (
  SELECT
    name,
    contact_info,
    COUNT(*) AS total_messages,
    SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS times_replied
  FROM messages
  WHERE chat_size = 1 AND name IS NOT NULL
  GROUP BY name, contact_info
)
SELECT
  r.name,
  CAST(r.last_received_at AS VARCHAR) AS waiting_since,
  COALESCE(CAST(s.last_sent_at AS VARCHAR), 'never') AS last_replied,
  h.total_messages,
  h.times_replied
FROM last_received r
LEFT JOIN last_sent s ON r.chat_id = s.chat_id
JOIN history h ON r.name = h.name
WHERE s.last_sent_at IS NULL
   OR r.last_received_at > s.last_sent_at
ORDER BY r.last_received_at DESC
LIMIT 25
```

### 3 — Filter out likely spam

Apply these heuristics to the results before presenting — drop any row that looks like spam:

- `total_messages <= 2` AND `times_replied = 0` — probably a one-way automated message
- `name` matches a pattern like "Alert", "Notify", "Update", "Confirm", "Verify", "Code", "Receipt", "Order", "Delivery", "Appointment" (case-insensitive) — automated services
- `waiting_since` is very old (> 6 months) AND `total_messages < 5` — likely stale/spam that was never relevant

When in doubt, include it — the user can dismiss false positives. The goal is to surface real people, not achieve perfect spam filtering.

### 4 — Add urgency context

For anything waiting more than a few days, also check if there are recent follow-up messages:

```sql
SELECT name, COUNT(*) AS follow_ups
FROM messages
WHERE is_from_me = 0
  AND chat_size = 1
  AND name IN ('NAME_1', 'NAME_2')
  AND timestamp > 'THEIR_LAST_MESSAGE_BEFORE_YOURS'
GROUP BY name
```

Multiple messages without a reply is a stronger signal.

### 5 — Present clearly

Group by urgency:

**Waiting a long time (> 1 week)** — list with "waiting since [date]"

**Waiting a few days (2–7 days)** — list with relative time ("3 days ago")

**Recent (today / yesterday)** — no alarm needed, just surface them

For each contact show: name, how long they've been waiting, and whether you've ever replied to them before (`times_replied = 0` means this is their first message to you — could be spam or could be someone reaching out cold).

End by asking: "Want me to do a quick recap of any of these conversations?"

## What this skill cannot do

- Read the content of messages (only metadata)
- Determine if a message actually requires a reply vs. was informational
- Detect group chats where you haven't replied (covered by a different query pattern)
- Know if you replied outside of iMessage (email, phone call, in person)
