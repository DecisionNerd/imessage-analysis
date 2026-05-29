use datafusion::arrow::util::display::{ArrayFormatter, FormatOptions};
use imessage_core::{
    etl::incremental,
    query::{built_in, QueryEngine},
};
use serde_json::{json, Value};

use crate::server::ServerState;

pub fn list() -> Value {
    json!([
        {
            "name": "run_etl",
            "description": "Run full ETL pipeline — extract from chat.db and write Parquet dataset.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "db_path": { "type": "string", "description": "Path to chat.db (optional, uses default)" }
                }
            }
        },
        {
            "name": "refresh",
            "description": "Incremental update — only fetch messages since the last ETL run.",
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
            "name": "schema",
            "description": "Return the dataset schema (column names and types).",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

pub async fn call(state: &ServerState, name: &str, args: Value) -> Result<Value, String> {
    match name {
        "run_etl" => {
            imessage_core::run_etl(&state.config).map_err(|e| e.to_string())?;
            state.invalidate_engine().await;
            Ok(json!({ "content": [{ "type": "text", "text": "ETL complete." }] }))
        }

        "refresh" => {
            let summary = incremental::refresh(&state.config).map_err(|e| e.to_string())?;
            state.invalidate_engine().await;
            Ok(json!({ "content": [{ "type": "text", "text": format!("{} new messages added.", summary.rows_written) }] }))
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

    Ok(match name {
        "query" => str_arg("sql").ok_or("missing `sql` argument")?,
        "top_contacts" => built_in::top_contacts(usize_arg("limit", 10), int_arg("year"), bool_arg("direct_only")),
        "time_series" => built_in::time_series(
            str_arg("contact").as_deref().map(str::to_string).as_deref(),
            usize_arg("window", 28),
            str_arg("start").as_deref().map(str::to_string).as_deref(),
            str_arg("end").as_deref().map(str::to_string).as_deref(),
        ),
        "reactions" => built_in::reactions(str_arg("contact").as_deref().map(str::to_string).as_deref(), int_arg("year")),
        "effects" => built_in::effects(int_arg("year")),
        "links" => built_in::links(usize_arg("limit", 20)),
        "seasonality" => match args.get("kind").and_then(|v| v.as_str()).unwrap_or("dow") {
            "month" => built_in::seasonality_month().to_string(),
            _ => built_in::seasonality_dow().to_string(),
        },
        "contact_stats" => built_in::contact_stats(str_arg("contact").as_deref().map(str::to_string).as_deref()),
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
