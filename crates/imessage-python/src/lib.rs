use std::sync::Arc;

use arrow::datatypes::Schema;
use imessage_core::{
    etl::incremental,
    models::EtlConfig,
    query::{built_in, QueryEngine},
};
use pyo3::prelude::*;
use pyo3_arrow::PyTable;
use std::path::PathBuf;

fn make_config(
    db_path: Option<String>,
    data_dir: Option<String>,
    contacts_config: Option<String>,
    auto_contacts: bool,
) -> EtlConfig {
    let mut config = EtlConfig::with_defaults();
    if let Some(p) = db_path {
        config.db_path = PathBuf::from(p);
    }
    if let Some(p) = data_dir {
        config.data_dir = PathBuf::from(p);
    }
    config.contacts_config = contacts_config.map(PathBuf::from);
    config.auto_contacts = auto_contacts;
    config
}

fn rt() -> PyResult<tokio::runtime::Runtime> {
    tokio::runtime::Runtime::new().map_err(to_py_err)
}

fn to_py_err(e: impl std::fmt::Display) -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
}

fn run_sql<'py>(py: Python<'py>, config: &EtlConfig, sql: &str) -> PyResult<Bound<'py, PyAny>> {
    let batches = rt()?.block_on(async {
        let engine = QueryEngine::open(&config.data_dir).await.map_err(to_py_err)?;
        engine.execute(sql).await.map_err(to_py_err)
    })?;
    let schema = if batches.is_empty() {
        Arc::new(Schema::empty())
    } else {
        batches[0].schema()
    };
    PyTable::try_new(batches, schema)
        .map_err(to_py_err)?
        .into_pyarrow(py)
}

/// Sync message history — full build on first run, incremental update after that.
#[pyfunction]
#[pyo3(signature = (db_path=None, data_dir=None, contacts_config=None, auto_contacts=true))]
fn sync(
    db_path: Option<String>,
    data_dir: Option<String>,
    contacts_config: Option<String>,
    auto_contacts: bool,
) -> PyResult<()> {
    use imessage_core::storage::metadata::EtlMetadata;
    let config = make_config(db_path, data_dir, contacts_config, auto_contacts);
    let meta = EtlMetadata::load(&config.data_dir).map_err(to_py_err)?;
    match meta {
        None => imessage_core::run_etl(&config).map_err(to_py_err).map(|_| ()),
        Some(m) => imessage_core::run_etl_since(&config, m.last_message_rowid)
            .map_err(to_py_err)
            .map(|_| ()),
    }
}

/// Run the full ETL pipeline — read chat.db, transform, write Parquet.
#[pyfunction]
#[pyo3(signature = (db_path=None, data_dir=None, contacts_config=None, auto_contacts=true))]
fn run_etl(
    db_path: Option<String>,
    data_dir: Option<String>,
    contacts_config: Option<String>,
    auto_contacts: bool,
) -> PyResult<()> {
    let config = make_config(db_path, data_dir, contacts_config, auto_contacts);
    imessage_core::run_etl(&config).map_err(to_py_err).map(|_| ())
}

/// Incremental update — only messages since the last ETL run.
#[pyfunction]
#[pyo3(signature = (db_path=None, data_dir=None, contacts_config=None, auto_contacts=true))]
fn refresh(
    db_path: Option<String>,
    data_dir: Option<String>,
    contacts_config: Option<String>,
    auto_contacts: bool,
) -> PyResult<()> {
    let config = make_config(db_path, data_dir, contacts_config, auto_contacts);
    incremental::refresh(&config).map_err(to_py_err).map(|_| ())
}

/// Execute arbitrary SQL against the messages dataset. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (sql, data_dir=None))]
fn query<'py>(py: Python<'py>, sql: String, data_dir: Option<String>) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    run_sql(py, &config, &sql)
}

/// Most-messaged contacts. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (limit=10, year=None, direct_only=true, data_dir=None))]
fn top_contacts<'py>(
    py: Python<'py>,
    limit: usize,
    year: Option<i32>,
    direct_only: bool,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    run_sql(py, &config, &built_in::top_contacts(limit, year, direct_only, None))
}

/// Daily message counts with rolling average. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (contact=None, window=28, start=None, end=None, data_dir=None))]
fn time_series<'py>(
    py: Python<'py>,
    contact: Option<String>,
    window: usize,
    start: Option<String>,
    end: Option<String>,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    run_sql(py, &config, &built_in::time_series(
        contact.as_deref(),
        window,
        start.as_deref(),
        end.as_deref(),
        None,
    ))
}

/// Reaction type breakdown. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (contact=None, year=None, data_dir=None))]
fn reactions<'py>(
    py: Python<'py>,
    contact: Option<String>,
    year: Option<i32>,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    run_sql(py, &config, &built_in::reactions(contact.as_deref(), year, None))
}

/// Message effect breakdown. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (year=None, data_dir=None))]
fn effects<'py>(
    py: Python<'py>,
    year: Option<i32>,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    run_sql(py, &config, &built_in::effects(year))
}

/// Top shared link domains. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (limit=20, data_dir=None))]
fn links<'py>(
    py: Python<'py>,
    limit: usize,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    run_sql(py, &config, &built_in::links(limit))
}

/// Messages by day-of-week or month-of-year. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (kind="dow", data_dir=None))]
fn seasonality<'py>(
    py: Python<'py>,
    kind: &str,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    let sql = match kind {
        "month" => built_in::seasonality_month(None).to_string(),
        _ => built_in::seasonality_dow(None).to_string(),
    };
    run_sql(py, &config, &sql)
}

/// Search contacts by name, phone, or email. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (query, limit=20, data_dir=None))]
fn search_contacts<'py>(
    py: Python<'py>,
    query: String,
    limit: usize,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    run_sql(py, &config, &built_in::search_contacts(&query, limit))
}

/// Per-contact statistics. Returns a pyarrow.Table.
#[pyfunction]
#[pyo3(signature = (contact=None, data_dir=None))]
fn contact_stats<'py>(
    py: Python<'py>,
    contact: Option<String>,
    data_dir: Option<String>,
) -> PyResult<Bound<'py, PyAny>> {
    let config = make_config(None, data_dir, None, false);
    run_sql(py, &config, &built_in::contact_stats(contact.as_deref()))
}

#[pymodule]
fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sync, m)?)?;
    m.add_function(wrap_pyfunction!(run_etl, m)?)?;
    m.add_function(wrap_pyfunction!(refresh, m)?)?;
    m.add_function(wrap_pyfunction!(query, m)?)?;
    m.add_function(wrap_pyfunction!(top_contacts, m)?)?;
    m.add_function(wrap_pyfunction!(time_series, m)?)?;
    m.add_function(wrap_pyfunction!(reactions, m)?)?;
    m.add_function(wrap_pyfunction!(effects, m)?)?;
    m.add_function(wrap_pyfunction!(links, m)?)?;
    m.add_function(wrap_pyfunction!(seasonality, m)?)?;
    m.add_function(wrap_pyfunction!(search_contacts, m)?)?;
    m.add_function(wrap_pyfunction!(contact_stats, m)?)?;
    Ok(())
}
