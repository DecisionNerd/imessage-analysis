pub mod analysis;
pub mod query;
pub mod sync;

use imessage_core::models::EtlConfig;
use std::path::PathBuf;

pub fn build_config(
    db_path: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    contacts: Option<PathBuf>,
    no_auto_contacts: bool,
) -> EtlConfig {
    let mut config = EtlConfig::with_defaults();
    if let Some(p) = db_path {
        config.db_path = p;
    }
    if let Some(p) = data_dir {
        config.data_dir = p;
    }
    config.contacts_config = contacts;
    config.auto_contacts = !no_auto_contacts;
    config
}
