use imessage_core::{error::Result, models::EtlConfig, storage::metadata::EtlMetadata};
use indicatif::{ProgressBar, ProgressStyle};

pub fn run(config: &EtlConfig) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let meta = EtlMetadata::load(&config.data_dir)?;

    let summary = match meta {
        None => {
            spinner.set_message("Building message history for the first time…");
            match imessage_core::run_etl(config) {
                Ok(s) => s,
                Err(e) => {
                    spinner.finish_and_clear();
                    return Err(e);
                }
            }
        }
        Some(ref m) => {
            spinner.set_message(format!(
                "Syncing new messages (last sync: {})…",
                m.last_run_utc.as_deref().unwrap_or("unknown")
            ));
            match imessage_core::run_etl_since(config, m.last_message_rowid) {
                Ok(s) => s,
                Err(e) => {
                    spinner.finish_and_clear();
                    return Err(e);
                }
            }
        }
    };

    spinner.finish_and_clear();

    if meta.is_none() {
        println!(
            "✓ Built — {} messages indexed in {}",
            summary.rows_written,
            config.data_dir.display()
        );
    } else if summary.rows_written == 0 {
        println!("✓ Already up to date");
    } else {
        println!("✓ Synced — {} new messages", summary.rows_written);
    }

    Ok(())
}
