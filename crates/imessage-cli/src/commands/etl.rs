use imessage_core::{error::Result, models::EtlConfig};
use indicatif::{ProgressBar, ProgressStyle};

pub fn run(config: &EtlConfig) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message("Reading from chat.db…");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let summary = match imessage_core::run_etl(config) {
        Ok(s) => s,
        Err(e) => {
            spinner.finish_and_clear();
            return Err(e);
        }
    };

    spinner.finish_and_clear();
    println!(
        "✓ ETL complete — {} messages written to {}",
        summary.rows_written,
        config.data_dir.display()
    );
    Ok(())
}
