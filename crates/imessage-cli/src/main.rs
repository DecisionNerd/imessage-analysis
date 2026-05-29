mod commands;
mod output;

use clap_complete::Shell;

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
    #[arg(long, global = true, hide = true)]
    db_path: Option<PathBuf>,

    /// Directory for Parquet output [default: ~/.imessage-analysis/]
    #[arg(long, global = true, hide = true)]
    data_dir: Option<PathBuf>,

    /// Path to contacts override TOML
    #[arg(long, global = true, hide = true)]
    contacts: Option<PathBuf>,

    /// Disable automatic macOS Contacts.app lookup
    #[arg(long, global = true, hide = true, default_value_t = false)]
    no_auto_contacts: bool,

    /// Output format: table, json, csv
    #[arg(long, global = true, hide = true, default_value = "table")]
    format: String,

    /// Suppress progress indicators
    #[arg(short, long, global = true, hide = true, default_value_t = false)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync your message history — builds the dataset on first run, updates incrementally after that
    Sync {
        /// Force a full rebuild even if a dataset already exists
        #[arg(long)]
        force: bool,
    },
    /// Show dataset status — last sync time, message count, and storage size
    Status,
    /// Execute arbitrary SQL against the dataset
    Query {
        sql: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Search for contacts by name or phone/email — use results as input to other commands
    SearchContacts {
        /// Name, phone number, or email to search for (case-insensitive substring match)
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Most-messaged contacts
    TopContacts {
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        year: Option<i32>,
        /// Only 1-on-1 chats
        #[arg(long)]
        direct_only: bool,
        /// Only sent messages
        #[arg(long, conflicts_with = "received")]
        sent: bool,
        /// Only received messages
        #[arg(long, conflicts_with = "sent")]
        received: bool,
    },
    /// Daily message counts with rolling average
    TimeSeries {
        #[arg(long)]
        contact: Option<String>,
        /// Rolling average window in days
        #[arg(long, default_value_t = 28)]
        window: usize,
        /// Filter to a specific year (shorthand for --start / --end)
        #[arg(long, conflicts_with_all = ["start", "end"])]
        year: Option<i32>,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
        /// Only sent messages
        #[arg(long, conflicts_with = "received")]
        sent: bool,
        /// Only received messages
        #[arg(long, conflicts_with = "sent")]
        received: bool,
        #[arg(long, default_value_t = 200)]
        limit: usize,
    },
    /// Reaction type breakdown
    Reactions {
        #[arg(long)]
        contact: Option<String>,
        #[arg(long)]
        year: Option<i32>,
        /// Only reactions on sent messages
        #[arg(long, conflicts_with = "received")]
        sent: bool,
        /// Only reactions on received messages
        #[arg(long, conflicts_with = "sent")]
        received: bool,
    },
    /// Message effect breakdown
    Effects {
        #[arg(long)]
        year: Option<i32>,
    },
    /// Top shared link domains
    Links {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Message seasonality (day-of-week or month-of-year)
    Seasonality {
        /// dow or month
        #[arg(long, default_value = "dow")]
        kind: String,
        /// Only sent messages
        #[arg(long, conflicts_with = "received")]
        sent: bool,
        /// Only received messages
        #[arg(long, conflicts_with = "sent")]
        received: bool,
    },
    /// Per-contact statistics
    ContactStats {
        #[arg(long)]
        contact: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Generate shell completions
    Completions {
        shell: Shell,
    },
}

fn sent_received_filter(sent: bool, received: bool) -> Option<&'static str> {
    match (sent, received) {
        (true, false) => Some("is_from_me = 1"),
        (false, true) => Some("is_from_me = 0"),
        _ => None,
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();
    let fmt = output::Format::from_str(&cli.format);
    let quiet = cli.quiet;

    let config = commands::build_config(
        cli.db_path,
        cli.data_dir,
        cli.contacts,
        cli.no_auto_contacts,
    );

    let result = match cli.command {
        Commands::Sync { force } => commands::sync::run(&config, force, quiet),
        Commands::Status => commands::status::run(&config),
        Commands::Query { sql, limit } => commands::query::run(&config, &sql, limit, &fmt),
        Commands::SearchContacts { query, limit } => {
            commands::analysis::search_contacts(&config, &query, limit, &fmt)
        }
        Commands::TopContacts { limit, year, direct_only, sent, received } => {
            let direction = sent_received_filter(sent, received);
            commands::analysis::top_contacts(&config, limit, year, direct_only, direction, &fmt)
        }
        Commands::TimeSeries { contact, window, year, start, end, sent, received, limit } => {
            let direction = sent_received_filter(sent, received);
            let (start, end) = if let Some(y) = year {
                (Some(format!("{y}-01-01")), Some(format!("{y}-12-31")))
            } else {
                (start, end)
            };
            commands::analysis::time_series(
                &config,
                contact.as_deref(),
                window,
                start.as_deref(),
                end.as_deref(),
                direction,
                limit,
                &fmt,
            )
        }
        Commands::Reactions { contact, year, sent, received } => {
            let direction = sent_received_filter(sent, received);
            commands::analysis::reactions(&config, contact.as_deref(), year, direction, &fmt)
        }
        Commands::Effects { year } => commands::analysis::effects(&config, year, &fmt),
        Commands::Links { limit } => commands::analysis::links(&config, limit, &fmt),
        Commands::Seasonality { kind, sent, received } => {
            let direction = sent_received_filter(sent, received);
            commands::analysis::seasonality(&config, &kind, direction, &fmt)
        }
        Commands::ContactStats { contact, limit } => {
            commands::analysis::contact_stats(&config, contact.as_deref(), limit, &fmt)
        }
        Commands::Completions { shell } => {
            use clap::CommandFactory;
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "imessage-analysis",
                &mut std::io::stdout(),
            );
            let hint = match shell {
                Shell::Zsh => "# Add to ~/.zshrc:\n#   source <(imessage-analysis completions zsh)",
                Shell::Bash => "# Add to ~/.bashrc:\n#   source <(imessage-analysis completions bash)",
                Shell::Fish => "# Save to completions dir:\n#   imessage-analysis completions fish > ~/.config/fish/completions/imessage-analysis.fish",
                _ => "",
            };
            if !hint.is_empty() {
                eprintln!("\n{hint}");
            }
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
