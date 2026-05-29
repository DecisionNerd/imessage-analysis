use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::Result;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EtlMetadata {
    pub last_message_rowid: i64,
    pub last_run_utc: Option<String>,
    pub total_messages: u64,
    pub schema_version: u32,
}

impl EtlMetadata {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    pub fn load(data_dir: &Path) -> Result<Option<Self>> {
        let path = data_dir.join("metadata.json");
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(path)?;
        let meta: EtlMetadata = serde_json::from_str(&contents)
            .map_err(|e| crate::error::Error::Config(e.to_string()))?;
        Ok(Some(meta))
    }

    pub fn save(&self, data_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(data_dir)?;
        let path = data_dir.join("metadata.json");
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::Error::Config(e.to_string()))?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}
