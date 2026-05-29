use imessage_core::{
    error::Result,
    models::EtlConfig,
    storage::metadata::EtlMetadata,
};
use indicatif::{ProgressBar, ProgressStyle};

pub fn run(config: &EtlConfig) -> Result<()> {
    let meta = EtlMetadata::load(&config.data_dir)?;
    let since = match meta {
        Some(ref m) => m.last_message_rowid,
        None => {
            eprintln!("No metadata found — performing full ETL (same as `etl` command).");
            0
        }
    };

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message(format!("Refreshing messages after ROWID {since}…"));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let summary = imessage_core::run_etl_since(config, since)?;

    spinner.finish_and_clear();
    println!("✓ Refresh complete — {} new messages", summary.rows_written);
    Ok(())
}
