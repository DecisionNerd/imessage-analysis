pub mod contacts;
pub mod error;
pub mod etl;
pub mod models;
pub mod query;
pub mod storage;

use crate::error::Result;
use crate::models::EtlConfig;
use crate::storage::metadata::EtlMetadata as Meta;

/// Run the full ETL pipeline: read chat.db, transform, write Parquet + metadata.
pub fn run_etl(config: &EtlConfig) -> Result<EtlSummary> {
    run_etl_since(config, 0)
}

/// Run an incremental ETL from a given ROWID watermark.
pub fn run_etl_since(config: &EtlConfig, since_rowid: i64) -> Result<EtlSummary> {
    tracing::info!(db = %config.db_path.display(), "Loading from SQLite");
    let raw = etl::sqlite_reader::read_since(&config.db_path, since_rowid)?;

    tracing::info!(count = raw.messages.len(), "Transforming rows");
    let contacts = contacts::resolve(config.auto_contacts, config.contacts_config.as_deref())?;
    let batch = etl::transforms::transform(&raw.messages, &raw.chat_members, &contacts)?;

    tracing::info!(rows = batch.num_rows(), "Writing Parquet");
    storage::parquet::write(&config.data_dir, &batch)?;

    let meta = Meta {
        last_message_rowid: raw.max_message_rowid,
        last_run_utc: Some(chrono::Utc::now().to_rfc3339()),
        total_messages: batch.num_rows() as u64,
        schema_version: Meta::CURRENT_SCHEMA_VERSION,
    };
    meta.save(&config.data_dir)?;

    tracing::info!("ETL complete");
    Ok(EtlSummary {
        rows_written: batch.num_rows(),
        max_rowid: raw.max_message_rowid,
    })
}

pub struct EtlSummary {
    pub rows_written: usize,
    pub max_rowid: i64,
}
