use std::path::Path;
use std::sync::Arc;

use datafusion::arrow::record_batch::RecordBatch;
use datafusion::datasource::file_format::parquet::ParquetFormat;
use datafusion::datasource::listing::{
    ListingOptions, ListingTable, ListingTableConfig, ListingTableUrl,
};
use datafusion::prelude::*;

use crate::error::{Error, Result};
use crate::storage::parquet::messages_path;

pub struct QueryEngine {
    ctx: SessionContext,
}

impl QueryEngine {
    pub async fn open(data_dir: &Path) -> Result<Self> {
        let parquet_path = messages_path(data_dir);
        if !parquet_path.exists() {
            return Err(Error::DatasetNotFound {
                path: data_dir.display().to_string(),
            });
        }

        let ctx = SessionContext::new();

        let table_url = ListingTableUrl::parse(parquet_path.to_string_lossy().as_ref())
            .map_err(|e| Error::Config(e.to_string()))?;

        let file_format = Arc::new(ParquetFormat::default().with_enable_pruning(true));
        let listing_options = ListingOptions::new(file_format).with_file_extension(".parquet");

        let config = ListingTableConfig::new(table_url)
            .with_listing_options(listing_options)
            .infer_schema(&ctx.state())
            .await
            .map_err(|e| Error::Config(e.to_string()))?;

        let table =
            Arc::new(ListingTable::try_new(config).map_err(|e| Error::Config(e.to_string()))?);

        ctx.register_table("messages", table)
            .map_err(|e| Error::Config(e.to_string()))?;

        Ok(Self { ctx })
    }

    pub async fn execute(&self, sql: &str) -> Result<Vec<RecordBatch>> {
        let df = self.ctx.sql(sql).await?;
        let batches = df.collect().await?;
        Ok(batches)
    }
}

impl From<datafusion::error::DataFusionError> for Error {
    fn from(e: datafusion::error::DataFusionError) -> Self {
        Error::Config(e.to_string())
    }
}
