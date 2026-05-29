use imessage_core::{error::Result, models::EtlConfig};

pub fn run(_config: &EtlConfig, _sql: &str, _limit: usize) -> Result<()> {
    // Phase 2 — DataFusion query layer not yet implemented.
    eprintln!("Query support coming in Phase 2 (DataFusion integration).");
    Ok(())
}
