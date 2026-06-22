const JUNK_PREFIXES: &[&str] = &["+2", "iI", "+ ", "+\n", "   2", "+!", "+(", "+*", "+<"];

use crabstep::TypedStreamDeserializer;

/// Extract human-readable text from Apple's attributedBody binary blob.
///
/// Apple encodes modern message text inside a typedstream NSAttributedString
/// graph. We prefer typedstream decoding and keep the old marker-based parser
/// for older/synthetic blobs.
pub fn parse(blob: &[u8]) -> Option<String> {
    parse_typedstream(blob).or_else(|| parse_legacy_marker_blob(blob))
}

fn parse_typedstream(blob: &[u8]) -> Option<String> {
    let mut typedstream = TypedStreamDeserializer::new(blob);
    let root = typedstream.iter_root().ok()?;

    root.primitives()
        .into_iter()
        .filter_map(|value| value.as_str())
        .map(str::trim)
        .find(|value| is_message_body_candidate(value))
        .map(ToOwned::to_owned)
}

fn is_message_body_candidate(value: &str) -> bool {
    !value.is_empty() && !value.starts_with("__") && !value.ends_with("AttributeName")
}

fn parse_legacy_marker_blob(blob: &[u8]) -> Option<String> {
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
    loop {
        let mut stripped_any = false;
        for prefix in JUNK_PREFIXES {
            if let Some(stripped) = result.strip_prefix(prefix) {
                result = stripped.trim_start();
                stripped_any = true;
                break;
            }
        }
        if !stripped_any {
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
    fn extracts_text_from_modern_typedstream_blob() {
        let blob = [
            0x04, 0x0b, 0x73, 0x74, 0x72, 0x65, 0x61, 0x6d, 0x74, 0x79, 0x70, 0x65, 0x64, 0x81,
            0xe8, 0x03, 0x84, 0x01, 0x40, 0x84, 0x84, 0x84, 0x19, 0x4e, 0x53, 0x4d, 0x75, 0x74,
            0x61, 0x62, 0x6c, 0x65, 0x41, 0x74, 0x74, 0x72, 0x69, 0x62, 0x75, 0x74, 0x65, 0x64,
            0x53, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x84, 0x84, 0x12, 0x4e, 0x53, 0x41, 0x74,
            0x74, 0x72, 0x69, 0x62, 0x75, 0x74, 0x65, 0x64, 0x53, 0x74, 0x72, 0x69, 0x6e, 0x67,
            0x00, 0x84, 0x84, 0x08, 0x4e, 0x53, 0x4f, 0x62, 0x6a, 0x65, 0x63, 0x74, 0x00, 0x85,
            0x92, 0x84, 0x84, 0x84, 0x0f, 0x4e, 0x53, 0x4d, 0x75, 0x74, 0x61, 0x62, 0x6c, 0x65,
            0x53, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x01, 0x84, 0x84, 0x08, 0x4e, 0x53, 0x53, 0x74,
            0x72, 0x69, 0x6e, 0x67, 0x01, 0x95, 0x84, 0x01, 0x2b, 0x0a, 0x4e, 0x6f, 0x74, 0x65,
            0x72, 0x20, 0x74, 0x65, 0x73, 0x74, 0x86, 0x84, 0x02, 0x69, 0x49, 0x01, 0x0a, 0x92,
            0x84, 0x84, 0x84, 0x0c, 0x4e, 0x53, 0x44, 0x69, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x61,
            0x72, 0x79, 0x00, 0x95, 0x84, 0x01, 0x69, 0x01, 0x92, 0x84, 0x98, 0x98, 0x1d, 0x5f,
            0x5f, 0x6b, 0x49, 0x4d, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x50, 0x61, 0x72,
            0x74, 0x41, 0x74, 0x74, 0x72, 0x69, 0x62, 0x75, 0x74, 0x65, 0x4e, 0x61, 0x6d, 0x65,
            0x86, 0x92, 0x84, 0x84, 0x84, 0x08, 0x4e, 0x53, 0x4e, 0x75, 0x6d, 0x62, 0x65, 0x72,
            0x00, 0x84, 0x84, 0x07, 0x4e, 0x53, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x00, 0x95, 0x84,
            0x01, 0x2a, 0x84, 0x9b, 0x9b, 0x00, 0x86, 0x86, 0x86,
        ];

        assert_eq!(parse(&blob), Some("Noter test".to_string()));
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
    fn repeatedly_strips_legacy_junk_prefixes() {
        let blob = b"...NSString+\niIactual messageNSDictionary...";
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
