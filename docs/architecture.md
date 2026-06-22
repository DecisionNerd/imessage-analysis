# Architecture

## Overview

The project is a Cargo workspace with four crates and two distinct processing layers.

```
imessage-analysis/
├── crates/
│   ├── imessage-core/      # Library: all data logic
│   ├── imessage-cli/       # Binary: command-line interface
│   ├── imessage-mcp/       # Binary: MCP server
│   └── imessage-python/    # cdylib: Python bindings (built with maturin)
├── .github/workflows/
│   └── release.yml         # Tag-triggered release + Homebrew formula update
└── Formula/
    └── imessage-analysis.rb  # Reference Homebrew formula
```

## Two-layer design

### ETL layer — pure Rust, synchronous

Reads from `~/Library/Messages/chat.db` and writes Parquet. No DataFusion involved.

```
rusqlite
  └── SQL JOIN (message + chat_message_join + handle)
        └── stream rows → per-row Rust transforms
              ├── blob_parser    — attributedBody BLOB → inferred_text
              ├── body_text       — analysis-ready body text for search/NLP/retrieval
              ├── detect_reaction         — associated_message_type → reaction string
              ├── detect_message_effect   — expressive_send_style_id → effect name
              ├── extract_link_domain     — URL parsing via `url` crate
              ├── chat membership lookup  — HashMap from chat_handle_join
              └── contact name resolution — HashMap from Contacts.app + TOML overrides
                    └── Arrow RecordBatch builders → Parquet (Snappy)
```

Rationale: the ETL is a sequential pipeline over a small number of well-defined tables. Running it through a query planner would add latency and complexity without benefit. Plain Rust is faster, simpler, and easier to test.

### Query layer — DataFusion, async

Reads the Parquet output and executes SQL.

```
Parquet file(s) in ~/.imessage-analysis/
  └── DataFusion SessionContext (ListingTable)
        ├── ad-hoc SQL (CLI: query subcommand, MCP: query tool)
        └── named analysis queries (built_in.rs)
              └── Arrow RecordBatches → output formatter (table / JSON / CSV)
```

Rationale: once data is in columnar Parquet format, DataFusion's vectorised engine is the right tool. Predicate pushdown means `WHERE year = 2024` avoids scanning all columns. The `SessionContext` is held in memory in the MCP server so queries are instant.

## Core library (`imessage-core`)

```
src/
├── lib.rs              — run_etl(), run_etl_since()
├── error.rs            — unified Error type
├── models.rs           — EtlConfig, detect_reaction(), detect_message_effect(), extract_link_domain()
├── etl/
│   ├── sqlite_reader.rs  — reads chat.db → MessageRow + ChatMembership
│   ├── blob_parser.rs    — attributedBody BLOB → Option<String>
│   ├── transforms.rs     — MessageRow[] → Arrow RecordBatch
│   └── incremental.rs    — ROWID-watermark refresh
├── contacts/
│   ├── macos_contacts.rs — CNContactStore via objc2-contacts (macOS only)
│   └── config_overrides.rs — TOML file → HashMap<String, String>
├── query/
│   ├── engine.rs         — DataFusion SessionContext setup
│   └── built_in.rs       — named analysis SQL generators
└── storage/
    ├── parquet.rs        — write RecordBatch → Parquet
    └── metadata.rs       — read/write metadata.json (watermark + schema version)
```

## Key dependencies

| Crate | Layer | Role |
|---|---|---|
| `rusqlite` (bundled) | ETL | Read chat.db without requiring a system SQLite |
| `arrow` / `parquet` | ETL + Query | In-memory columnar format and Parquet I/O |
| `datafusion` | Query | SQL engine over Parquet |
| `objc2` + `objc2-contacts` | ETL | macOS Contacts.app access (target_os = "macos" only) |
| `block2` | ETL | Objective-C block support for `CNContactStore` enumeration |
| `url` | ETL | URL parsing for link domain extraction |
| `clap` | CLI | Argument parsing |
| `comfy-table` | CLI | Terminal table rendering |
| `indicatif` | CLI | Progress spinners |
| `tokio` | Query / MCP | Async runtime required by DataFusion |
| `pyo3` | Python | Python extension module |

## MCP server

The MCP server (`imessage-mcp`) is a long-lived process that speaks JSON-RPC 2.0 over stdio. It holds a single `DataFusion SessionContext` in memory across all requests, avoiding re-parsing the Parquet schema on every tool call.

After `sync`, the engine is invalidated so the next query re-initialises against the updated Parquet file.

```
stdin → JSON-RPC message → server::handle()
                               ├── initialize         → return capabilities
                               ├── tools/list         → return tool schemas
                               └── tools/call         → tools::call()
                                                           ├── sync (ETL layer)
                                                           └── query / analysis  (DataFusion)
                                                                 → JSON rows → stdout
```

## Incremental refresh

Apple's `message` table uses auto-incrementing `ROWID`. New messages always have a higher ROWID than existing ones.

1. `metadata.json` stores `last_message_rowid` after each ETL run
2. `sync` runs `SELECT ... FROM message WHERE ROWID > :watermark`
3. Transforms and writes a new Parquet file
4. Updates the watermark

The current implementation performs a full rewrite of the Parquet file on each refresh (simple and correct). A future optimisation could append new rows to a separate fragment file and use DataFusion's `ListingTable` to read them all as one logical table.

## Python bindings

The `imessage-python` crate is a `cdylib` built with [maturin](https://github.com/PyO3/maturin). It is excluded from the default `cargo build` workspace build and must be built with `maturin develop` or `maturin build`. The Python package name is `imessage_analysis`.

Python functions return `pyarrow.Table` objects via zero-copy Arrow FFI, so `.to_pandas()` works without copying data.

The Python package is **query-only**. `sync()`, `run_etl()`, and `refresh()` raise `RuntimeError` — the Python interpreter process cannot obtain macOS Contacts permission, so syncing from Python would produce a dataset with phone numbers instead of names. Users must sync via the CLI binary, which has the required entitlement embedded.

## Parquet schema versioning

`metadata.json` includes a `schema_version` integer. If the Parquet schema changes between releases (new or renamed columns), `schema_version` is incremented and the tool will prompt the user to re-run `sync` rather than silently producing incorrect results.
