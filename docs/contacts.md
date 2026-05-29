# Contact Name Resolution

By default, `imessage-analysis` maps phone numbers and email addresses to human-readable names by querying macOS Contacts.app. You can also provide manual overrides via a TOML config file.

## How it works

1. **Automatic** — on ETL, the tool reads all contacts from your macOS address book (`CNContactStore`) and builds a map of phone number / email → name.
2. **Overrides** — if you supply a TOML config file, those entries are merged on top of the automatic results. Overrides always win.
3. The merged map is applied during the transform step: the `name` column in the output dataset contains the resolved name, or falls back to the raw contact info if no name is found.

## Why you might need overrides

- The same person texts you from both their phone number and their Apple ID email — two separate handles in the iMessage database, but you want them grouped under one name.
- A contact is saved under a different name in your address book than you'd like to use in analysis.
- You've received messages from a number that isn't saved in Contacts.

## Config file format

```toml
[contacts]
"+14155551234" = "Alice"
"alice@example.com" = "Alice"   # same person, different handle
"+14155559999" = "Bob"
"+447700900000" = "Charlie"
```

- Keys are phone numbers (E.164 format preferred) or email addresses (case-insensitive).
- Values are the display names you want in your dataset.
- A copy of this example is at `config/contacts.example.toml`.

## Using the config file

Pass the path with `--contacts`:

```sh
imessage-analysis etl --contacts ~/my-contacts.toml
imessage-analysis refresh --contacts ~/my-contacts.toml
```

The contacts file is only read during ETL / refresh — it is not needed for query or analysis commands after the Parquet file is built.

## Disabling automatic lookup

If you do not want the tool to query Contacts.app (e.g. for privacy, or because you manage all names via the config file):

```sh
imessage-analysis etl --no-auto-contacts --contacts ~/my-contacts.toml
```

## Phone number normalization

iMessage stores handles in various formats (`+1 (415) 555-1234`, `+14155551234`, `4155551234`). The tool strips spaces, dashes, and parentheses when matching, but keeps the leading `+` if present. For best results, use E.164 format (`+<country code><number>`) in your config file.
