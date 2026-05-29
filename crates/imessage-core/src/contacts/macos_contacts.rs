use std::collections::HashMap;

use crate::error::Result;

// macOS Contacts.app integration — Phase 3.
// Placeholder: returns an empty map until the objc2-contacts bindings are wired up.
pub fn fetch() -> Result<HashMap<String, String>> {
    Ok(HashMap::new())
}
