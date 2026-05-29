use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

use crate::error::Result;

pub fn messages_path(data_dir: &Path) -> PathBuf {
    data_dir.join("messages.parquet")
}

pub fn write(data_dir: &Path, batch: &RecordBatch) -> Result<()> {
    fs::create_dir_all(data_dir)?;
    let path = messages_path(data_dir);
    let file = fs::File::create(&path)?;

    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();

    let mut writer = ArrowWriter::try_new(file, Arc::clone(batch.schema_ref()), Some(props))?;
    writer.write(batch)?;
    writer.close()?;

    Ok(())
}
