# Data Model

After running `imessage-analysis etl`, all data is stored as a single Parquet file at `~/.imessage-analysis/messages.parquet`. The `messages` table has the following columns:

| Column | Type | Nullable | Description |
|---|---|---|---|
| `message_id` | Int64 | No | Unique message identifier (`ROWID` from Apple's database) |
| `is_from_me` | Int8 | No | `1` = sent by you, `0` = received |
| `text` | Utf8 | Yes | Raw message text from Apple's database (can be NULL — see `text_combined`) |
| `inferred_text` | Utf8 | Yes | Text extracted from the `attributedBody` binary column when `text` is NULL (English only) |
| `text_combined` | Utf8 | Yes | Best available text: `text` if non-null, otherwise `inferred_text` |
| `handle_id` | Int64 | No | Internal ID of the sender/receiver handle |
| `contact_info` | Utf8 | Yes | Phone number or email address of the other party |
| `updated_contact_info` | Utf8 | Yes | Corrected recipient field: for sent messages in 1-on-1 chats this is the other person; for group chats it is `group-chat` |
| `chat_id` | Int64 | Yes | Internal ID of the conversation thread |
| `chat_members_handles` | Utf8 | Yes | JSON array of handle IDs for all participants, e.g. `[12,34]` |
| `chat_members_contact_info` | Utf8 | Yes | JSON array of contact info strings for all participants, e.g. `["+14155551234","alice@example.com"]` |
| `chat_size` | Int64 | No | Number of participants excluding yourself. `1` = private chat, `>1` = group chat |
| `is_audio_message` | Int8 | No | `1` if this is an audio message, `0` otherwise |
| `message_effect` | Utf8 | No | Effect the message was sent with, or `no-effect`. See [effects](#message-effects) below. |
| `reaction` | Utf8 | No | Reaction type, or `no-reaction`. See [reactions](#reactions) below. |
| `is_thread_reply` | Int8 | No | `1` if this message is a reply to a specific message in a thread, `0` otherwise |
| `link_domain` | Utf8 | Yes | Domain of a shared link preview (e.g. `spotify.com`), or NULL if none |
| `name` | Utf8 | Yes | Resolved contact name from macOS Contacts.app or your [contacts config](contacts.md) |
| `timestamp` | Timestamp (UTC) | Yes | Full timestamp of the message |
| `date` | Date32 | Yes | Date portion of the timestamp |
| `month` | Int8 | Yes | Month (1–12) |
| `year` | Int16 | Yes | Year |

## Parsing list columns

`chat_members_handles` and `chat_members_contact_info` are stored as JSON arrays serialised to strings. In Python:

```python
import ast, json
df['chat_members_contact_info'].apply(json.loads)
```

In SQL:
```sql
-- Count messages in group chats
SELECT COUNT(*) FROM messages WHERE chat_size > 1
```

## Message effects

Possible values of `message_effect`:

`Confetti`, `Echo`, `Fireworks`, `HappyBirthday`, `Heart`, `Lasers`, `ShootingStar`, `Sparkles`, `Spotlight`, `gentle`, `impact`, `invisibleink`, `loud`, `no-effect`

## Reactions

Possible values of `reaction`:

`Loved`, `Liked`, `Laughed`, `Emphasized`, `Questioned`, `Disliked`, `Removed heart`, `Removed like`, `Removed laugh`, `Removed emphasis`, `Removed question mark`, `Removed dislike`, `no-reaction`

## Querying the dataset directly

The Parquet file can be queried with any tool that supports the format:

```sh
# DuckDB
duckdb -c "SELECT name, COUNT(*) FROM '~/.imessage-analysis/messages.parquet' GROUP BY name ORDER BY 2 DESC LIMIT 10"

# Python / pandas
import pandas as pd
df = pd.read_parquet("~/.imessage-analysis/messages.parquet")

# imessage-analysis CLI
imessage-analysis query "SELECT name, COUNT(*) AS n FROM messages GROUP BY name ORDER BY n DESC LIMIT 10"
```

## Metadata

Alongside the Parquet file, `~/.imessage-analysis/metadata.json` tracks ETL state:

```json
{
  "last_message_rowid": 375000,
  "last_run_utc": "2026-05-29T10:30:00Z",
  "total_messages": 374804,
  "schema_version": 1
}
```

The `last_message_rowid` watermark is used by `imessage-analysis refresh` to fetch only new messages.
