use imessage_core::{error::Result, models::EtlConfig, storage::metadata::EtlMetadata};
use indicatif::{ProgressBar, ProgressStyle};

pub fn run(config: &EtlConfig, force: bool, quiet: bool) -> Result<()> {
    let meta = EtlMetadata::load(&config.data_dir)?;

    let is_first_run = force || meta.is_none();

    let spinner = if quiet {
        None
    } else {
        let s = ProgressBar::new_spinner();
        s.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        s.enable_steady_tick(std::time::Duration::from_millis(80));
        Some(s)
    };

    if is_first_run {
        if let Some(s) = &spinner { s.set_message("Building message history…"); }
    } else if let Some(ref m) = meta {
        if let Some(s) = &spinner {
            s.set_message(format!(
                "Syncing new messages (last sync: {})…",
                m.last_run_utc.as_deref().unwrap_or("unknown")
            ));
        }
    }

    let summary = if is_first_run {
        match imessage_core::run_etl(config) {
            Ok(s) => s,
            Err(e) => {
                if let Some(s) = &spinner { s.finish_and_clear(); }
                return Err(e);
            }
        }
    } else {
        let since = meta.as_ref().map(|m| m.last_message_rowid).unwrap_or(0);
        match imessage_core::run_etl_since(config, since) {
            Ok(s) => s,
            Err(e) => {
                if let Some(s) = &spinner { s.finish_and_clear(); }
                return Err(e);
            }
        }
    };

    if let Some(s) = &spinner { s.finish_and_clear(); }

    if !quiet {
        if is_first_run {
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
    }

    Ok(())
}
