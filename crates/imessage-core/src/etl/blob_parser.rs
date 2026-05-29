const JUNK_PREFIXES: &[&str] = &["+2", "iI", "+ ", "+\n", "   2", "+!", "+(", "+*", "+<"];

/// Extract human-readable text from Apple's attributedBody binary blob.
///
/// Apple encodes message text inside a binary plist-like structure. We extract
/// it by: filtering bytes to printable ASCII, then pulling the substring between
/// the "NSString" and "NSDictionary" markers, then stripping known junk prefixes.
pub fn parse(blob: &[u8]) -> Option<String> {
    let ascii = filter_to_ascii(blob);
    let extracted = extract_between(&ascii, "NSString", "NSDictionary")?;
    let cleaned = strip_junk_prefixes(extracted.trim());
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn filter_to_ascii(blob: &[u8]) -> String {
    blob.iter()
        .map(|&b| b as char)
        .map(|c| {
            if c.is_ascii_graphic() || c == ' ' || c == '\n' || c == '\t' {
                c
            } else {
                ' '
            }
        })
        .collect()
}

fn extract_between<'a>(s: &'a str, start_marker: &str, end_marker: &str) -> Option<&'a str> {
    let start = s.find(start_marker)? + start_marker.len();
    let end = s[start..].find(end_marker)? + start;
    Some(&s[start..end])
}

fn strip_junk_prefixes(s: &str) -> &str {
    let mut result = s;
    for prefix in JUNK_PREFIXES {
        if let Some(stripped) = result.strip_prefix(prefix) {
            result = stripped.trim_start();
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_for_empty_blob() {
        assert_eq!(parse(&[]), None);
    }

    #[test]
    fn returns_none_when_markers_absent() {
        assert_eq!(parse(b"hello world"), None);
    }

    #[test]
    fn extracts_text_between_markers() {
        let blob = b"...NSStringhello world NSDictionary...";
        assert_eq!(parse(blob), Some("hello world".to_string()));
    }

    #[test]
    fn strips_junk_prefix() {
        let blob = b"...NSString+2actual messageNSDictionary...";
        assert_eq!(parse(blob), Some("actual message".to_string()));
    }

    #[test]
    fn tolerates_non_ascii_bytes() {
        let mut blob = b"...NSString".to_vec();
        blob.extend_from_slice(&[0x80, 0x9F, 0xFE]); // non-printable bytes
        blob.extend_from_slice(b"hello NSDictionary...");
        let result = parse(&blob);
        assert!(result.is_some());
        assert!(result.unwrap().contains("hello"));
    }
}
