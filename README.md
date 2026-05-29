# iMessage Analysis

Extract, transform, and analyse your Mac iMessage database. The ETL notebook converts Apple's SQLite database into a clean pandas DataFrame; the analysis notebook provides ready-to-run visualisations and statistics.

## Requirements

- macOS (the iMessage database is Mac-only)
- Python 3.11+
- Jupyter Notebooks

## Getting Started

1. Fork and clone the repo
2. Install dependencies (`pip install -e .` or install the packages listed in `pyproject.toml`)
3. Run the two notebooks in order:
   - `notebooks/imessages-extract-transform-load.ipynb` — run once to build and save the DataFrame
   - `notebooks/imessages-analysis.ipynb` — explore and visualise your data

---

## How Apple Stores iMessages

The iMessage database lives at `~/Library/Messages/chat.db` and uses SQLite. Four tables are relevant:

| Table | Description |
|---|---|
| `message` | One row per message: text, timestamp, `is_from_me`, and metadata |
| `handle` | Maps `handle_id` → contact info (phone number or email) |
| `chat_message_join` | Maps `message_id` → `chat_id` |
| `chat_handle_join` | Maps `chat_id` → the `handle_id`s of all participants |

---

## ETL Notebook (`imessages-extract-transform-load.ipynb`)

Reads the raw SQLite database and produces `data/df_messages.csv`.

### Steps

**Step 1 — Connect and load**
Opens `~/Library/Messages/chat.db` and loads the four core tables. You must update the username in the database path.

**Step 2 — Rename columns**
`ROWID` → `message_id`; `id` (on the handle table) → `contact_info`.

**Step 3 — Join tables and infer text**
Merges messages with `chat_message_join` to add `chat_id`. For messages where Apple's `text` column is `NULL`, the text is inferred from the `attributedBody` binary column using `clean_text()` in `src/helper.py`. The best available text is stored in `text_combined`.

**Step 4 — Add chat membership**
For each message, looks up all participants in the chat via `chat_handle_join` and `handle`. Adds `chat_members_handles`, `chat_members_contact_info`, and `chat_size`.

**Step 5 — Add message features**

| Feature | Column | Method |
|---|---|---|
| Message effect (Fireworks, Confetti, etc.) | `message_effect` | Parse `expressive_send_style_id` |
| Thread reply | `is_thread_reply` | Non-null `thread_originator_guid` |
| Shared link domain | `link_domain` | Parse `text_combined` when `balloon_bundle_id` is set |
| Reaction type | `reaction` | Map `associated_message_type` code via `detect_reaction()` |

**Step 6 — Map contacts to names**
You provide a dictionary that maps phone numbers / emails to human-readable names (e.g. `{'+447700900000': 'Alice'}`). This handles the case where the same person appears under multiple handles (SMS vs iMessage, phone vs email). The result is stored in the `name` column.

**Step 7 — Add date columns**
Extracts `date`, `month`, and `year` from the Apple epoch timestamp (seconds since 2001-01-01).

**Step 8 — Fix recipient for sent messages**
Apple leaves the recipient `NULL` on sent messages. This step infers it: for 1-1 chats the recipient is the other participant; for group chats it is set to `'group-chat'`. Result stored in `updated_contact_info`.

**Step 9 — Save**
Exports to `data/df_messages.csv`.

---

## Output DataFrame

22 columns:

| Column | Type | Description |
|---|---|---|
| `message_id` | int | Unique message identifier (`ROWID` from Apple) |
| `is_from_me` | int | `1` = sent by you, `0` = received |
| `text_combined` | str | Best available message text: native `text` if non-null, otherwise `inferred_text` |
| `text` | str | Raw text from Apple's database (may be `NULL`) |
| `inferred_text` | str | Text extracted from `attributedBody` when `text` is `NULL` (English only) |
| `handle_id` | int | ID of the sender/receiver handle |
| `contact_info` | str | Phone number or email of the sender (null for sent messages before Step 8) |
| `updated_contact_info` | str | Corrected recipient: other person for 1-1 chats, `'group-chat'` for group chats |
| `chat_id` | int | ID of the conversation thread |
| `chat_members_contact_info` | str | Serialised list of contact info for all chat participants (use `ast.literal_eval()` to parse) |
| `chat_members_handles` | str | Serialised list of handle IDs for all chat participants |
| `chat_size` | int | Number of participants excluding yourself (`1` = private chat) |
| `is_audio_message` | int | `1` if this is an audio message |
| `message_effect` | str | Effect the message was sent with, or `'no-effect'`. Possible values: `Confetti`, `Echo`, `Fireworks`, `HappyBirthday`, `Heart`, `Lasers`, `Spotlight`, `Sparkles`, `ShootingStar`, `gentle`, `impact`, `invisibleink`, `loud` |
| `reaction` | str | Reaction type or `'no-reaction'`. Possible values: `Loved`, `Liked`, `Laughed`, `Emphasized`, `Questioned`, `Disliked`, `Removed heart`, `Removed like`, `Removed laugh`, `Removed emphasis`, `Removed question mark`, `Removed dislike` |
| `is_thread_reply` | int | `1` if this message is a reply to a specific message in a thread |
| `link_domain` | str | Domain of a shared link (e.g. `'spotify.com'`), or `'no-link'` |
| `name` | str | Human-readable contact name from the mapping you provide in Step 6 |
| `timestamp` | str | Full datetime string, e.g. `'2024-04-26 14:35:03'`. Convert with `pd.Timestamp(x)` |
| `date` | str | Date only, e.g. `'2024-04-26'` |
| `month` | int | Month extracted from `date` (1–12) |
| `year` | int | Year extracted from `date` |

---

## Analysis Notebook (`imessages-analysis.ipynb`)

Loads `data/df_messages.csv` and produces plots saved to `plots/`.

### Sections

**1 — Messages over time**
- Daily message count with a 28-day rolling average
- Sent vs received trends on the same chart
- Per-person message trend (customise the contact name)
- Annual totals bar chart
- Sent/received ratio over time

**2 — Top contacts**
- Counts messages per person (direct chats only, `chat_size == 1`)
- Bar chart of the top 10 most-texted people

**3 — Message features**
- Breakdown of reaction types (Loved, Liked, Laughed, …)
- Breakdown of message effects (Fireworks, Confetti, …)
- Thread reply rate
- Top shared link domains

**4–5 — Per-contact statistics**
- For each contact: total messages, first/last date, active days, average messages per day, percentage of days texted since first message

**6 — Seasonality**
- Messages by day of week (sent vs received)
- Messages by day of month (grouped into thirds)
- Messages by month of year

---

## Source Module (`src/helper.py`)

Utility functions used by the ETL notebook.

| Function | Description |
|---|---|
| `clean_text(byte_string)` | Infers message text from `attributedBody` when `text` is `NULL` |
| `extract_ascii_text(byte_string)` | Converts bytes to ASCII, preserving whitespace |
| `extract_substring(s, x1, x2)` | Extracts the substring between two delimiter strings |
| `convert_handle_id_to_contact_info(handle_id, handles)` | Looks up contact info for a given handle ID |
| `update_contact_info(contact_info, contact_info_list, message_id)` | Fills recipient for sent messages |
| `get_handle_and_contact_list(chat_id, chat_handle_join, handles)` | Returns all handle IDs and contact info for a chat |
| `get_chat_size(handles_list)` | Returns the number of participants in a chat |
| `get_rolling_avg(daily_count, column_name, window_size)` | Computes a rolling average, filling missing dates with zero |
| `detect_reaction(associated_message_type)` | Maps Apple's reaction code integer to a readable string |
| `detect_message_effect(x)` | Parses `expressive_send_style_id` into a readable effect name |
| `extract_domain(url)` | Extracts the domain from a URL string |
| `apply_function(row)` | Row-level wrapper for `extract_domain`; only fires when a link preview is present |

---

## Known Limitations

- **NULL text fields**: Some messages from Apple's database have a `NULL` `text` field. The `inferred_text` column attempts to recover the text from `attributedBody`, but this is imperfect and only works for English.
- **Group chat recipients**: Sent messages in group chats record only `'group-chat'` as the recipient rather than the full participant list.
- **Multiple handles per person**: The same person can have separate handle IDs for SMS vs iMessage, or for phone number vs Apple ID email. Use the name-mapping dictionary in Step 6 of the ETL notebook to consolidate them.
- **Non-person messages**: The dataset includes automated messages (2FA codes, booking confirmations, location-sharing notifications, etc.).

---

## Version History

**v3.3** — Performance: Step 3 (adding chat membership) reduced from ~5 minutes to near-instant.

**v3.2** — Added `is_audio_message`, `is_thread_reply`, `message_effect`. Replaced text-based reaction detection with the more accurate and complete `associated_message_type` integer mapping (now covers reaction removals and works regardless of device language).

**v3.1** — Added `reaction` column (English-only text-pattern detection).

**v3** — Introduced `inferred_text` via `attributedBody` parsing, substantially increasing the number of messages with recoverable text. Added `updated_contact_info` to fill recipients on sent messages.

---

## Further Reading

- [Accessing your iMessage history on Mac](https://medium.com/@yaskalidis/heres-how-you-can-access-your-entire-imessage-history-on-your-mac-f8878276c6e9) — explains the database fundamentals
- [Analysis ideas](https://medium.com/@yaskalidis/fun-things-you-can-learn-about-yourself-and-from-your-messages-5101631a8e20) — examples of what you can do with the clean DataFrame

---

[![License: CC BY-NC 4.0](https://img.shields.io/badge/License-CC%20BY--NC%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc/4.0/)

Licensed under the [Creative Commons Attribution-NonCommercial 4.0 International License](https://creativecommons.org/licenses/by-nc/4.0/).
