use imessage_core::{error::Result, models::EtlConfig, storage::{metadata::EtlMetadata, parquet::messages_path}};

pub fn run(config: &EtlConfig) -> Result<()> {
    let meta = EtlMetadata::load(&config.data_dir)?;

    match meta {
        None => {
            println!("No dataset found at {}", config.data_dir.display());
            println!("Run `imessage-analysis sync` to build it.");
        }
        Some(m) => {
            let parquet = messages_path(&config.data_dir);
            let size = std::fs::metadata(&parquet)
                .map(|md| format_bytes(md.len()))
                .unwrap_or_else(|_| "unknown".to_string());

            println!("Dataset:      {}", parquet.display());
            println!("Messages:     {}", format_count(m.total_messages));
            println!("Last sync:    {}", m.last_run_utc.as_deref().unwrap_or("unknown"));
            println!("Size:         {size}");
            println!("Schema:       v{}", m.schema_version);

            if m.contacts_resolved == 0 {
                println!("Contacts:     ⚠ not resolved (names showing as phone numbers)");
                println!("              → System Settings → Privacy & Security → Contacts");
                println!("              → or use --contacts contacts.toml");
            } else {
                println!("Contacts:     {} resolved", m.contacts_resolved);
            }
        }
    }

    Ok(())
}

fn format_count(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn format_bytes(n: u64) -> String {
    const MB: u64 = 1024 * 1024;
    const KB: u64 = 1024;
    if n >= MB {
        format!("{:.1} MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1} KB", n as f64 / KB as f64)
    } else {
        format!("{n} B")
    }
}
