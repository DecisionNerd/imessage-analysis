# Apple's iMessage Database

Understanding how Apple stores iMessages helps explain some of the quirks in the dataset.

## Location

```
~/Library/Messages/chat.db
```

This is a SQLite database. Reading it requires **Full Disk Access** (see [installation](installation.md)).

## Core tables

### `message`

One row per message. Key columns:

| Column | Description |
|---|---|
| `ROWID` | Auto-incrementing unique ID (becomes `message_id`) |
| `text` | Plain-text body — can be NULL (see below) |
| `attributedBody` | Binary encoded message body — fallback when `text` is NULL |
| `date` | Nanoseconds since **2001-01-01 00:00:00 UTC** (Apple epoch) |
| `handle_id` | FK → `handle.ROWID` — the other party |
| `is_from_me` | `1` = sent, `0` = received |
| `associated_message_type` | Non-zero for reactions; encodes reaction type |
| `expressive_send_style_id` | Non-null for messages sent with an effect |
| `thread_originator_guid` | Non-null for thread replies |
| `balloon_bundle_id` | Non-null for link previews and other rich content |
| `is_audio_message` | `1` for audio messages |

### `handle`

Maps internal handle IDs to contact information.

| Column | Description |
|---|---|
| `ROWID` | Handle ID |
| `id` | Phone number or email address |

### `chat_message_join`

Links each message to its conversation thread.

| Column | Description |
|---|---|
| `message_id` | FK → `message.ROWID` |
| `chat_id` | FK → `chat.ROWID` |

### `chat_handle_join`

Lists all participants in each conversation.

| Column | Description |
|---|---|
| `chat_id` | FK → `chat.ROWID` |
| `handle_id` | FK → `handle.ROWID` |

## Known quirks

### NULL text fields

Apple migrated to runtime-encoded message bodies at some point, meaning the `text` column is NULL for many messages even though they had visible content. `imessage-analysis` works around this by extracting the text from the `attributedBody` binary column. This extraction is best-effort: modern typedstream bodies are decoded directly, while older or unusual blobs fall back to marker-based extraction.

The `text` and `inferred_text` columns preserve where the body came from. For analysis and retrieval, use `body_text`: it contains the best parsed message body across SMS, RCS, and iMessage-native rows. The older `text_combined` column is kept as a compatibility alias.

### Multiple handles per person

The same person can appear under multiple `handle_id` values:
- SMS (green bubble) vs iMessage (blue bubble) from the same phone number
- Phone number vs Apple ID email (e.g. when they have iMessage configured with both)

Use the [contacts config](contacts.md) to merge these under a single name.

### Sent message recipients

When `is_from_me = 1`, the `handle_id` column is 0 (no sender handle). The recipient must be inferred from the chat's participant list:
- 1-on-1 chat: the recipient is the other participant
- Group chat: there is no single recipient — the `updated_contact_info` column is set to `group-chat`

### Timestamps

Apple stores timestamps as nanoseconds since **2001-01-01 00:00:00 UTC** (the Apple Cocoa epoch), not the Unix epoch (1970-01-01). Conversion: `unix_ts = apple_ts / 1_000_000_000 + 978307200`.

Older macOS versions (pre-High Sierra) stored timestamps as seconds rather than nanoseconds.

### Non-person messages

The `messages` table includes automated messages: 2FA codes, booking confirmations, location-sharing notifications, Apple Watch activity competitions, etc. These are present in the dataset and will appear in contact stats if the originating number is saved as a contact.
