use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Parquet error: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Cannot open database at {path}. Grant Full Disk Access to Terminal:\n  System Settings → Privacy & Security → Full Disk Access")]
    DbNotFound { path: String },

    #[error("Cannot open database at {path}: {reason}\n  If Messages is running, try copying the database first, or grant Full Disk Access to Terminal:\n  System Settings → Privacy & Security → Full Disk Access")]
    DbAccessDenied { path: String, reason: String },

    #[error("No dataset found at {path}. Run `imessage-analysis sync` first.")]
    DatasetNotFound { path: String },
}

pub type Result<T> = std::result::Result<T, Error>;
