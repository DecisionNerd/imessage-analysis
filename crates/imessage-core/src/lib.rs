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
    run_etl_with_progress(config, |_| {})
}

/// Like [`run_etl`] but calls `progress` with the current row count every 10,000 rows
/// while reading from SQLite, enabling a CLI spinner to show live progress.
pub fn run_etl_with_progress(
    config: &EtlConfig,
    progress: impl Fn(usize) + Send + Sync,
) -> Result<EtlSummary> {
    run_etl_since_with_progress(config, 0, progress)
}

/// Run an incremental ETL from a given ROWID watermark.
pub fn run_etl_since(config: &EtlConfig, since_rowid: i64) -> Result<EtlSummary> {
    run_etl_since_with_progress(config, since_rowid, |_| {})
}

fn run_etl_since_with_progress(
    config: &EtlConfig,
    since_rowid: i64,
    progress: impl Fn(usize) + Send + Sync,
) -> Result<EtlSummary> {
    tracing::info!(db = %config.db_path.display(), "Loading from SQLite");
    let raw = etl::sqlite_reader::read_since_with_progress(
        &config.db_path,
        since_rowid,
        Some(&progress),
    )?;

    tracing::info!(count = raw.messages.len(), "Transforming rows");
    let contacts = contacts::resolve(config.auto_contacts, config.contacts_config.as_deref())?;
    let contacts_resolved = contacts.len();
    let batch = etl::transforms::transform(&raw.messages, &raw.chat_members, &contacts)?;

    tracing::info!(rows = batch.num_rows(), "Writing Parquet");
    storage::parquet::write(&config.data_dir, &batch)?;

    let meta = Meta {
        last_message_rowid: raw.max_message_rowid,
        last_run_utc: Some(chrono::Utc::now().to_rfc3339()),
        total_messages: batch.num_rows() as u64,
        schema_version: Meta::CURRENT_SCHEMA_VERSION,
        contacts_resolved,
    };
    meta.save(&config.data_dir)?;

    tracing::info!("ETL complete");
    Ok(EtlSummary {
        rows_written: batch.num_rows(),
        max_rowid: raw.max_message_rowid,
        contacts_resolved,
    })
}

pub struct EtlSummary {
    pub rows_written: usize,
    pub max_rowid: i64,
    pub contacts_resolved: usize,
}
