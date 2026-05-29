use std::sync::Arc;

use imessage_core::{models::EtlConfig, query::QueryEngine};
use serde_json::{json, Value};
use tokio::sync::RwLock;

use crate::tools;

pub struct ServerState {
    pub config: EtlConfig,
    /// QueryEngine is held across requests so DataFusion doesn't re-parse the schema each time.
    pub engine: Arc<RwLock<Option<QueryEngine>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            config: EtlConfig::with_defaults(),
            engine: Arc::new(RwLock::new(None)),
        }
    }

    /// Ensure the QueryEngine is initialised (or re-initialised after ETL).
    pub async fn ensure_engine(&self) -> Result<(), String> {
        let mut guard = self.engine.write().await;
        if guard.is_none() {
            let engine = QueryEngine::open(&self.config.data_dir)
                .await
                .map_err(|e| e.to_string())?;
            *guard = Some(engine);
        }
        Ok(())
    }

    pub async fn invalidate_engine(&self) {
        let mut guard = self.engine.write().await;
        *guard = None;
    }
}

pub async fn handle(state: &ServerState, msg: Value) -> Option<Value> {
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    let method = msg.get("method")?.as_str()?;

    // Notifications (no id) get no response
    if msg.get("id").is_none() {
        return None;
    }

    let params = msg.get("params").cloned().unwrap_or(json!({}));

    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "imessage-analysis", "version": env!("CARGO_PKG_VERSION") }
        })),

        "tools/list" => Ok(json!({ "tools": tools::list() })),

        "tools/call" => {
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            tools::call(state, tool_name, args).await
        }

        _ => return Some(error_response(id, -32601, &format!("Method not found: {method}"))),
    };

    Some(match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err(e) => error_response(id, -32603, &e),
    })
}

pub fn error_response(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}
