use datafusion::arrow::util::display::{ArrayFormatter, FormatOptions};
use imessage_core::{
    query::{built_in, QueryEngine},
    storage::{metadata::EtlMetadata, parquet::messages_path},
};
use serde_json::{json, Value};

use crate::server::ServerState;

pub fn list() -> Value {
    json!([
        {
            "name": "sync",
            "description": "Sync message history — builds the dataset on first run, updates incrementally after that. Always call this before querying if you are not sure the data is fresh.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "query",
            "description": "Execute arbitrary SQL against the messages dataset. The table is named `messages`.",
            "inputSchema": {
                "type": "object",
                "required": ["sql"],
                "properties": {
                    "sql":   { "type": "string", "description": "SQL query to run" },
                    "limit": { "type": "integer", "description": "Max rows (default 100)" }
                }
            }
        },
        {
            "name": "top_contacts",
            "description": "Most-messaged contacts.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit":       { "type": "integer" },
                    "year":        { "type": "integer" },
                    "direct_only": { "type": "boolean" }
                }
            }
        },
        {
            "name": "time_series",
            "description": "Daily message counts with rolling average.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "contact": { "type": "string" },
                    "window":  { "type": "integer", "description": "Rolling window in days (default 28)" },
                    "start":   { "type": "string", "description": "Start date YYYY-MM-DD" },
                    "end":     { "type": "string", "description": "End date YYYY-MM-DD" }
                }
            }
        },
        {
            "name": "reactions",
            "description": "Reaction type breakdown.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "contact": { "type": "string" },
                    "year":    { "type": "integer" }
                }
            }
        },
        {
            "name": "effects",
            "description": "Message effect breakdown.",
            "inputSchema": {
                "type": "object",
                "properties": { "year": { "type": "integer" } }
            }
        },
        {
            "name": "links",
            "description": "Top shared link domains.",
            "inputSchema": {
                "type": "object",
                "properties": { "limit": { "type": "integer" } }
            }
        },
        {
            "name": "seasonality",
            "description": "Message counts by day-of-week or month-of-year.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "kind": { "type": "string", "enum": ["dow", "month"], "description": "dow = day of week, month = month of year" }
                }
            }
        },
        {
            "name": "contact_stats",
            "description": "Per-contact statistics: total messages, first/last date, active days.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "contact": { "type": "string" },
                    "limit":   { "type": "integer" }
                }
            }
        },
        {
            "name": "search_contacts",
            "description": "Search for contacts by name, phone number, or email. Use this to find the exact name string to pass to other tools.",
            "inputSchema": {
                "type": "object",
                "required": ["query"],
                "properties": {
                    "query": { "type": "string", "description": "Substring to search for (case-insensitive)" },
                    "limit": { "type": "integer", "description": "Max results (default 20)" }
                }
            }
        },
        {
            "name": "status",
            "description": "Show dataset status — last sync time, total message count, and file size.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "schema",
            "description": "Return the dataset schema (column names and types).",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

pub async fn call(state: &ServerState, name: &str, args: Value) -> Result<Value, String> {
    match name {
        "sync" => {
            let meta = EtlMetadata::load(&state.config.data_dir).map_err(|e| e.to_string())?;
            let summary = match meta {
                None => imessage_core::run_etl(&state.config).map_err(|e| e.to_string())?,
                Some(m) => imessage_core::run_etl_since(&state.config, m.last_message_rowid)
                    .map_err(|e| e.to_string())?,
            };
            state.invalidate_engine().await;
            let mut msg = if summary.rows_written == 0 {
                "Already up to date.".to_string()
            } else {
                format!("{} messages synced.", summary.rows_written)
            };
            if summary.contacts_resolved == 0 {
                msg.push_str("\n\n⚠ Contact names were not resolved — names will show as phone numbers. Grant Contacts access in System Settings → Privacy & Security → Contacts, then re-run sync.");
            } else {
                msg.push_str(&format!("\n{} contacts resolved.", summary.contacts_resolved));
            }
            Ok(json!({ "content": [{ "type": "text", "text": msg }] }))
        }

        "status" => {
            let meta = EtlMetadata::load(&state.config.data_dir).map_err(|e| e.to_string())?;
            let result = match meta {
                None => json!({ "synced": false, "message": "No dataset found. Call sync first." }),
                Some(m) => {
                    let parquet = messages_path(&state.config.data_dir);
                    let size_bytes = std::fs::metadata(&parquet).map(|md| md.len()).unwrap_or(0);
                    json!({
                        "synced": true,
                        "total_messages": m.total_messages,
                        "last_sync": m.last_run_utc,
                        "size_bytes": size_bytes,
                        "schema_version": m.schema_version,
                        "contacts_resolved": m.contacts_resolved,
                        "contacts_warning": if m.contacts_resolved == 0 {
                            Some("Contact names were not resolved — names will show as phone numbers. Grant Contacts access in System Settings → Privacy & Security → Contacts, then re-run sync.")
                        } else {
                            None
                        }
                    })
                }
            };
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string(&result).unwrap() }] }))
        }

        "schema" => {
            state.ensure_engine().await?;
            let guard = state.engine.read().await;
            let engine = guard.as_ref().unwrap();
            let batches = engine.execute("SELECT * FROM messages LIMIT 0").await.map_err(|e| e.to_string())?;
            let schema_str = format!("{}", batches.first().map(|b| b.schema()).unwrap_or_else(|| std::sync::Arc::new(datafusion::arrow::datatypes::Schema::empty())));
            Ok(json!({ "content": [{ "type": "text", "text": schema_str }] }))
        }

        _ => {
            let sql = build_sql(name, &args)?;
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
            state.ensure_engine().await?;
            let guard = state.engine.read().await;
            let engine = guard.as_ref().unwrap();
            let batches = engine.execute(&sql).await.map_err(|e| e.to_string())?;
            let rows = batches_to_json(&batches, limit);
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string(&rows).unwrap() }] }))
        }
    }
}

fn build_sql(name: &str, args: &Value) -> Result<String, String> {
    let str_arg = |key: &str| -> Option<String> {
        args.get(key)?.as_str().map(str::to_string)
    };
    let int_arg = |key: &str| -> Option<i32> {
        args.get(key)?.as_i64().map(|v| v as i32)
    };
    let bool_arg = |key: &str| -> bool {
        args.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    };
    let usize_arg = |key: &str, default: usize| -> usize {
        args.get(key).and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(default)
    };

    let direction: Option<String> = match (bool_arg("sent"), bool_arg("received")) {
        (true, false) => Some("is_from_me = 1".to_string()),
        (false, true) => Some("is_from_me = 0".to_string()),
        _ => None,
    };
    let dir = direction.as_deref();

    // Expand --year shorthand for time_series
    let ts_start: Option<String> = int_arg("year").map(|y| format!("{y}-01-01")).or_else(|| str_arg("start"));
    let ts_end: Option<String> = int_arg("year").map(|y| format!("{y}-12-31")).or_else(|| str_arg("end"));

    Ok(match name {
        "query" => str_arg("sql").ok_or("missing `sql` argument")?,
        "top_contacts" => built_in::top_contacts(usize_arg("limit", 10), int_arg("year"), bool_arg("direct_only"), dir),
        "time_series" => built_in::time_series(
            str_arg("contact").as_deref().map(str::to_string).as_deref(),
            usize_arg("window", 28),
            ts_start.as_deref(),
            ts_end.as_deref(),
            dir,
        ),
        "reactions" => built_in::reactions(str_arg("contact").as_deref().map(str::to_string).as_deref(), int_arg("year"), dir),
        "effects" => built_in::effects(int_arg("year")),
        "links" => built_in::links(usize_arg("limit", 20)),
        "seasonality" => match args.get("kind").and_then(|v| v.as_str()).unwrap_or("dow") {
            "month" => built_in::seasonality_month(dir),
            _ => built_in::seasonality_dow(dir),
        },
        "contact_stats" => built_in::contact_stats(str_arg("contact").as_deref().map(str::to_string).as_deref()),
        "search_contacts" => {
            let q = str_arg("query").ok_or("missing `query` argument")?;
            built_in::search_contacts(&q, usize_arg("limit", 20))
        }
        _ => return Err(format!("Unknown tool: {name}")),
    })
}

fn batches_to_json(
    batches: &[datafusion::arrow::record_batch::RecordBatch],
    limit: usize,
) -> Vec<serde_json::Map<String, Value>> {
    let opts = FormatOptions::default();
    let mut rows = Vec::new();
    let mut count = 0;

    'outer: for batch in batches {
        let schema = batch.schema();
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        let formatters: Vec<ArrayFormatter> = match (0..batch.num_columns())
            .map(|i| ArrayFormatter::try_new(batch.column(i).as_ref(), &opts))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(f) => f,
            Err(_) => continue,
        };

        for row in 0..batch.num_rows() {
            if count >= limit {
                break 'outer;
            }
            let mut map = serde_json::Map::new();
            for (name, fmt) in field_names.iter().zip(formatters.iter()) {
                map.insert(name.to_string(), Value::String(fmt.value(row).to_string()));
            }
            rows.push(map);
            count += 1;
        }
    }
    rows
}
