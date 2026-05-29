mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "imessage-analysis",
    about = "Analyse your iMessage history",
    version
)]
struct Cli {
    /// Path to chat.db [default: ~/Library/Messages/chat.db]
    #[arg(long, global = true)]
    db_path: Option<PathBuf>,

    /// Directory for Parquet output [default: ~/.imessage-analysis/]
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,

    /// Path to contacts override TOML
    #[arg(long, global = true)]
    contacts: Option<PathBuf>,

    /// Disable automatic macOS Contacts.app lookup
    #[arg(long, global = true, default_value_t = false)]
    no_auto_contacts: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run full ETL pipeline (chat.db → Parquet)
    Etl,
    /// Incremental update — only new messages since last run
    Refresh,
    /// Execute arbitrary SQL against the dataset
    Query {
        sql: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();

    let config = commands::build_config(
        cli.db_path,
        cli.data_dir,
        cli.contacts,
        cli.no_auto_contacts,
    );

    let result = match cli.command {
        Commands::Etl => commands::etl::run(&config),
        Commands::Refresh => commands::refresh::run(&config),
        Commands::Query { sql, limit } => commands::query::run(&config, &sql, limit),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
