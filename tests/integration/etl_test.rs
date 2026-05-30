use std::collections::HashMap;
use std::path::PathBuf;

use imessage_core::etl::{sqlite_reader, transforms};
use imessage_core::query::built_in;

fn fixture_db() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/test_chat.db")
}

#[test]
fn reads_all_messages() {
    let raw = sqlite_reader::read_all(&fixture_db()).expect("read_all failed");
    assert_eq!(raw.messages.len(), 10, "expected 10 messages");
}

#[test]
fn max_rowid_is_correct() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    assert_eq!(raw.max_message_rowid, 10);
}

#[test]
fn read_since_filters_correctly() {
    let raw = sqlite_reader::read_since(&fixture_db(), 5).unwrap();
    assert_eq!(raw.messages.len(), 5, "expected 5 messages after ROWID 5");
    assert!(raw.messages.iter().all(|m| m.message_id > 5));
}

#[test]
fn chat_membership_built_correctly() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    // Chat 1: just Alice (handle 1)
    let chat1 = raw.chat_members.get(&1).expect("chat 1 missing");
    assert_eq!(chat1.handle_ids.len(), 1);
    assert_eq!(
        chat1.contact_infos.first().map(String::as_str),
        Some("+14155550001")
    );
    // Chat 2: Alice and Bob (handles 1 and 2)
    let chat2 = raw.chat_members.get(&2).expect("chat 2 missing");
    assert_eq!(chat2.handle_ids.len(), 2);
}

#[test]
fn transforms_produce_correct_row_count() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts: HashMap<String, String> = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();
    assert_eq!(batch.num_rows(), 10);
}

#[test]
fn transforms_schema_has_expected_columns() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();
    let schema = batch.schema();
    let names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    for col in &[
        "message_id",
        "is_from_me",
        "text_combined",
        "text",
        "inferred_text",
        "handle_id",
        "contact_info",
        "updated_contact_info",
        "chat_id",
        "chat_members_handles",
        "chat_members_contact_info",
        "chat_size",
        "is_audio_message",
        "message_effect",
        "reaction",
        "is_thread_reply",
        "link_domain",
        "name",
        "timestamp",
        "date",
        "month",
        "year",
    ] {
        assert!(names.contains(col), "missing column: {col}");
    }
}

#[test]
fn reaction_detected_correctly() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let reaction_col = batch
        .column_by_name("reaction")
        .expect("reaction column missing");
    let reactions = reaction_col
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();

    // msg 4 (index 3) is associated_message_type=2000 → "Loved"
    assert_eq!(reactions.value(3), "Loved");
    // msg 1 (index 0) has no reaction
    assert_eq!(reactions.value(0), "no-reaction");
}

#[test]
fn message_effect_detected_correctly() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let effect_col = batch
        .column_by_name("message_effect")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();

    // msg 5 (index 4) has Fireworks effect
    assert_eq!(effect_col.value(4), "Fireworks");
    // msg 1 (index 0) has no effect
    assert_eq!(effect_col.value(0), "no-effect");
}

#[test]
fn thread_reply_detected_correctly() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let col = batch
        .column_by_name("is_thread_reply")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::Int8Array>()
        .unwrap();

    // msg 6 (index 5) has thread_originator_guid → is_thread_reply=1
    assert_eq!(col.value(5), 1);
    // msg 1 (index 0) is not a thread reply
    assert_eq!(col.value(0), 0);
}

#[test]
fn audio_message_detected_correctly() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let col = batch
        .column_by_name("is_audio_message")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::Int8Array>()
        .unwrap();

    // msg 8 (index 7) is an audio message
    assert_eq!(col.value(7), 1);
    assert_eq!(col.value(0), 0);
}

#[test]
fn link_domain_extracted_correctly() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let col = batch
        .column_by_name("link_domain")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();

    // msg 7 (index 6) has a spotify link preview
    assert_eq!(col.value(6), "open.spotify.com");
}

#[test]
fn chat_size_correct() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let col = batch
        .column_by_name("chat_size")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::Int64Array>()
        .unwrap();

    // msg 1 (index 0) is in chat 1 → chat_size=1
    assert_eq!(col.value(0), 1);
    // msg 9 (index 8) is in chat 2 → chat_size=2
    assert_eq!(col.value(8), 2);
}

#[test]
fn updated_contact_info_for_sent_messages() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let col = batch
        .column_by_name("updated_contact_info")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();

    // msg 2 (index 1) is sent in 1-on-1 chat → recipient is Alice
    assert_eq!(col.value(1), "+14155550001");
    // msg 10 (index 9) is sent in group chat → "group-chat"
    assert_eq!(col.value(9), "group-chat");
}

#[test]
fn contact_name_resolved() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let mut contacts = HashMap::new();
    contacts.insert("+14155550001".to_string(), "Alice".to_string());
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let col = batch
        .column_by_name("name")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
        .unwrap();

    // msg 1 (index 0) is from Alice
    assert_eq!(col.value(0), "Alice");
}

#[test]
fn timestamp_converted_from_apple_epoch() {
    let raw = sqlite_reader::read_all(&fixture_db()).unwrap();
    let contacts = HashMap::new();
    let batch = transforms::transform(&raw.messages, &raw.chat_members, &contacts).unwrap();

    let col = batch
        .column_by_name("year")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::Int16Array>()
        .unwrap();

    // All test messages are in 2024
    assert_eq!(col.value(0), 2024);

    let month_col = batch
        .column_by_name("month")
        .unwrap()
        .as_any()
        .downcast_ref::<arrow::array::Int8Array>()
        .unwrap();
    assert_eq!(month_col.value(0), 1); // January
}

#[test]
fn search_contacts_escapes_percent() {
    let sql = built_in::search_contacts("50%", 10);
    assert!(sql.contains("50\\%"), "percent should be escaped");
    assert!(sql.contains("ESCAPE"), "ESCAPE clause should be present");
}

#[test]
fn search_contacts_escapes_underscore() {
    let sql = built_in::search_contacts("alice_b", 10);
    assert!(sql.contains("alice\\_b"), "underscore should be escaped");
}

#[test]
fn search_contacts_escapes_backslash() {
    let sql = built_in::search_contacts("C:\\Users", 10);
    assert!(
        sql.contains("c:\\\\users"),
        "backslash should be double-escaped and lowercased"
    );
}

#[test]
fn search_contacts_plain_query_unaffected() {
    let sql = built_in::search_contacts("alice", 10);
    assert!(sql.contains("alice"), "plain query should pass through");
}
