use imessage_core::{error::Result, models::EtlConfig, query::QueryEngine};

use crate::output::{print_batches, Format};

pub fn run(config: &EtlConfig, sql: &str, limit: usize, fmt: &Format) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| imessage_core::error::Error::Config(e.to_string()))?;
    rt.block_on(async {
        let engine = QueryEngine::open(&config.data_dir).await?;
        let batches = engine.execute(sql).await?;
        print_batches(&batches, fmt, limit);
        Ok(())
    })
}
