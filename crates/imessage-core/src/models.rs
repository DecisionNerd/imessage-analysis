use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtlConfig {
    /// Path to chat.db. Defaults to ~/Library/Messages/chat.db.
    pub db_path: PathBuf,
    /// Directory where Parquet output and metadata are written.
    pub data_dir: PathBuf,
    /// Optional path to a TOML file with contact name overrides.
    pub contacts_config: Option<PathBuf>,
    /// Whether to auto-resolve names from macOS Contacts.app.
    pub auto_contacts: bool,
}

impl EtlConfig {
    pub fn with_defaults() -> Self {
        let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            db_path: home.join("Library/Messages/chat.db"),
            data_dir: home.join(".imessage-analysis"),
            contacts_config: None,
            auto_contacts: true,
        }
    }
}

pub fn detect_reaction(associated_message_type: i64) -> &'static str {
    match associated_message_type {
        2000 => "Loved",
        2001 => "Liked",
        2002 => "Disliked",
        2003 => "Laughed",
        2004 => "Emphasized",
        2005 => "Questioned",
        3000 => "Removed heart",
        3001 => "Removed like",
        3002 => "Removed dislike",
        3003 => "Removed laugh",
        3004 => "Removed emphasis",
        3005 => "Removed question mark",
        _ => "no-reaction",
    }
}

pub fn detect_message_effect(expressive_send_style_id: &str) -> String {
    let segment = expressive_send_style_id
        .split('.')
        .next_back()
        .unwrap_or("");
    if segment.is_empty() {
        return "no-effect".to_string();
    }
    segment.replace("CK", "").replace("Effect", "")
}

pub fn extract_link_domain(text: &str) -> Option<String> {
    let start = text.find("https://").or_else(|| text.find("http://"))?;
    let url_str = text[start..]
        .split(|c: char| c.is_whitespace() || c == ')' || c == '>' || c == ',' || c == ';')
        .next()?
        .trim_end_matches(['.', '!', '?']);
    let parsed = url::Url::parse(url_str).ok()?;
    let host = parsed.host_str()?;
    let domain = host.strip_prefix("www.").unwrap_or(host);
    Some(domain.to_string())
}
