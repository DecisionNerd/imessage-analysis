# CLI Reference

## Global flags

These flags apply to every subcommand:

| Flag | Default | Description |
|---|---|---|
| `--db-path <PATH>` | `~/Library/Messages/chat.db` | Path to the iMessage SQLite database |
| `--data-dir <PATH>` | `~/.imessage-analysis/` | Directory where Parquet output and metadata are stored |
| `--contacts <PATH>` | — | Path to a [contacts override TOML](contacts.md) |
| `--no-auto-contacts` | false | Disable automatic name resolution from macOS Contacts.app |
| `--format <FORMAT>` | `table` | Output format: `table`, `json`, or `csv` |

---

## ETL commands

### `etl`

Run the full extract-transform-load pipeline. Reads `chat.db`, applies all transforms, and writes `messages.parquet` to `--data-dir`.

```sh
imessage-analysis etl
imessage-analysis etl --db-path /Volumes/Backup/chat.db --data-dir ~/my-data/
```

On first run this takes a few seconds for a typical chat history (300K–500K messages). Subsequent runs should use `refresh` instead.

### `refresh`

Incremental update — only processes messages with a higher ROWID than the last ETL run. Much faster than a full `etl` for day-to-day use.

```sh
imessage-analysis refresh
```

If no metadata exists (i.e. `etl` has never been run), `refresh` performs a full ETL automatically.

---

## Query commands

### `query`

Execute arbitrary SQL against the `messages` table.

```sh
imessage-analysis query "SELECT year, COUNT(*) AS n FROM messages GROUP BY year ORDER BY year"
imessage-analysis query "SELECT text_combined FROM messages WHERE name = 'Alice' LIMIT 20" --format json
imessage-analysis query "SELECT * FROM messages WHERE reaction != 'no-reaction'" --limit 100
```

The table is named `messages`. See [data model](data-model.md) for the full column list.

---

## Analysis commands

### `top-contacts`

Most-messaged contacts by total message count.

```sh
imessage-analysis top-contacts
imessage-analysis top-contacts --limit 20
imessage-analysis top-contacts --year 2024 --direct-only
```

| Flag | Default | Description |
|---|---|---|
| `--limit <N>` | 10 | Number of contacts to show |
| `--year <YEAR>` | — | Filter to a specific year |
| `--direct-only` | false | Only include 1-on-1 chats (exclude group chats) |

---

### `time-series`

Daily message counts with a rolling average. Useful for spotting trends over time.

```sh
imessage-analysis time-series
imessage-analysis time-series --contact "Alice" --window 7
imessage-analysis time-series --start 2023-01-01 --end 2023-12-31
```

| Flag | Default | Description |
|---|---|---|
| `--contact <NAME>` | — | Filter to a specific contact |
| `--window <DAYS>` | 28 | Rolling average window size in days |
| `--start <DATE>` | — | Start date (`YYYY-MM-DD`) |
| `--end <DATE>` | — | End date (`YYYY-MM-DD`) |
| `--limit <N>` | 200 | Max rows |

---

### `reactions`

Breakdown of reaction types (Loved, Liked, Laughed, etc.).

```sh
imessage-analysis reactions
imessage-analysis reactions --contact "Alice" --year 2024
```

---

### `effects`

Breakdown of message effects (Fireworks, Confetti, etc.).

```sh
imessage-analysis effects
imessage-analysis effects --year 2024
```

---

### `links`

Top shared link domains.

```sh
imessage-analysis links
imessage-analysis links --limit 30
```

---

### `seasonality`

Message counts broken down by day-of-week or month-of-year, split by sent vs received.

```sh
imessage-analysis seasonality           # day-of-week (default)
imessage-analysis seasonality --kind month
```

---

### `contact-stats`

Per-contact statistics: total messages, first and last message date, number of days with at least one message.

```sh
imessage-analysis contact-stats
imessage-analysis contact-stats --contact "Alice"
imessage-analysis contact-stats --limit 25
```

---

## Output formats

All analysis commands support `--format table` (default), `--format json`, and `--format csv`.

```sh
# Pipe to jq
imessage-analysis top-contacts --format json | jq '.'

# Export to CSV
imessage-analysis time-series --format csv > time-series.csv
```
