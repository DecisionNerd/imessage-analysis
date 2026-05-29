pub mod config_overrides;
pub mod macos_contacts;

use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;

/// Build the merged contact map: macOS Contacts (if enabled) with TOML overrides on top.
pub fn resolve(
    auto_contacts: bool,
    contacts_config: Option<&Path>,
) -> Result<HashMap<String, String>> {
    let auto = if auto_contacts {
        macos_contacts::fetch().unwrap_or_else(|e| {
            tracing::warn!("Could not load macOS Contacts: {e}");
            HashMap::new()
        })
    } else {
        HashMap::new()
    };

    let overrides = match contacts_config {
        Some(path) => config_overrides::load(path)?,
        None => HashMap::new(),
    };

    let mut merged = auto;
    for (k, v) in overrides {
        merged.insert(k, v);
    }
    Ok(merged)
}
