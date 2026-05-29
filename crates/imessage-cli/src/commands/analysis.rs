use imessage_core::{
    error::Result,
    models::EtlConfig,
    query::{built_in, QueryEngine},
};

use crate::output::{print_batches, Format};

macro_rules! run_query {
    ($config:expr, $sql:expr, $limit:expr, $fmt:expr) => {{
        let rt = tokio::runtime::Runtime::new().map_err(|e| imessage_core::error::Error::Config(e.to_string()))?;
        rt.block_on(async {
            let engine = QueryEngine::open(&$config.data_dir).await?;
            let batches = engine.execute(&$sql).await?;
            print_batches(&batches, $fmt, $limit);
            Ok(())
        })
    }};
}

pub fn top_contacts(
    config: &EtlConfig,
    limit: usize,
    year: Option<i32>,
    direct_only: bool,
    fmt: &Format,
) -> Result<()> {
    run_query!(config, built_in::top_contacts(limit, year, direct_only), limit, fmt)
}

pub fn time_series(
    config: &EtlConfig,
    contact: Option<&str>,
    window: usize,
    start: Option<&str>,
    end: Option<&str>,
    limit: usize,
    fmt: &Format,
) -> Result<()> {
    run_query!(config, built_in::time_series(contact, window, start, end), limit, fmt)
}

pub fn reactions(config: &EtlConfig, contact: Option<&str>, year: Option<i32>, fmt: &Format) -> Result<()> {
    run_query!(config, built_in::reactions(contact, year), 100, fmt)
}

pub fn effects(config: &EtlConfig, year: Option<i32>, fmt: &Format) -> Result<()> {
    run_query!(config, built_in::effects(year), 50, fmt)
}

pub fn links(config: &EtlConfig, limit: usize, fmt: &Format) -> Result<()> {
    run_query!(config, built_in::links(limit), limit, fmt)
}

pub fn seasonality(config: &EtlConfig, kind: &str, fmt: &Format) -> Result<()> {
    let sql = match kind {
        "month" => built_in::seasonality_month().to_string(),
        _ => built_in::seasonality_dow().to_string(),
    };
    run_query!(config, sql, 50, fmt)
}

pub fn contact_stats(config: &EtlConfig, contact: Option<&str>, limit: usize, fmt: &Format) -> Result<()> {
    run_query!(config, built_in::contact_stats(contact), limit, fmt)
}
