use std::path::Path;

use crate::error::Result;
use crate::models::EtlConfig;
use crate::storage::metadata::EtlMetadata;
use crate::{run_etl_since, EtlSummary};

/// Run an incremental refresh: only fetch messages newer than the last ETL run.
/// Falls back to a full ETL if no metadata exists.
pub fn refresh(config: &EtlConfig) -> Result<EtlSummary> {
    let since = last_rowid(&config.data_dir)?;
    run_etl_since(config, since)
}

pub fn last_rowid(data_dir: &Path) -> Result<i64> {
    Ok(EtlMetadata::load(data_dir)?
        .map(|m| m.last_message_rowid)
        .unwrap_or(0))
}
