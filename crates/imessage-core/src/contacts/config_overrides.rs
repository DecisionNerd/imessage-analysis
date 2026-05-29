use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Deserialize)]
struct ContactsFile {
    contacts: HashMap<String, String>,
}

pub fn load(path: &Path) -> Result<HashMap<String, String>> {
    let contents = std::fs::read_to_string(path)?;
    let parsed: ContactsFile = toml::from_str(&contents)
        .map_err(|e| Error::Config(format!("Invalid contacts TOML: {e}")))?;
    Ok(parsed.contacts)
}
