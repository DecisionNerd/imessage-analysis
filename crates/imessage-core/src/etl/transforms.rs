use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use chrono::{Datelike, TimeZone, Utc};

use crate::error::Result;
use crate::etl::blob_parser;
use crate::etl::sqlite_reader::{ChatMembership, MessageRow};
use crate::models::{detect_message_effect, detect_reaction, extract_link_domain};

pub fn schema() -> Schema {
    Schema::new(vec![
        Field::new("message_id", DataType::Int64, false),
        Field::new("is_from_me", DataType::Int8, false),
        Field::new("text", DataType::Utf8, true),
        Field::new("inferred_text", DataType::Utf8, true),
        Field::new("text_combined", DataType::Utf8, true),
        Field::new("handle_id", DataType::Int64, false),
        Field::new("contact_info", DataType::Utf8, true),
        Field::new("updated_contact_info", DataType::Utf8, true),
        Field::new("chat_id", DataType::Int64, true),
        Field::new("chat_members_handles", DataType::Utf8, true),
        Field::new("chat_members_contact_info", DataType::Utf8, true),
        Field::new("chat_size", DataType::Int64, false),
        Field::new("is_audio_message", DataType::Int8, false),
        Field::new("message_effect", DataType::Utf8, false),
        Field::new("reaction", DataType::Utf8, false),
        Field::new("is_thread_reply", DataType::Int8, false),
        Field::new("link_domain", DataType::Utf8, true),
        Field::new("name", DataType::Utf8, true),
        Field::new(
            "timestamp",
            DataType::Timestamp(TimeUnit::Second, Some(Arc::from("UTC"))),
            true,
        ),
        Field::new("date", DataType::Date32, true),
        Field::new("month", DataType::Int8, true),
        Field::new("year", DataType::Int16, true),
    ])
}

pub fn transform(
    rows: &[MessageRow],
    chat_members: &HashMap<i64, ChatMembership>,
    contacts: &HashMap<String, String>,
) -> Result<RecordBatch> {
    let len = rows.len();

    let mut message_id = Int64Builder::with_capacity(len);
    let mut is_from_me = Int8Builder::with_capacity(len);
    let mut col_text = StringBuilder::with_capacity(len, len * 32);
    let mut inferred_text = StringBuilder::with_capacity(len, len * 32);
    let mut text_combined = StringBuilder::with_capacity(len, len * 32);
    let mut handle_id = Int64Builder::with_capacity(len);
    let mut contact_info = StringBuilder::with_capacity(len, len * 16);
    let mut updated_contact_info = StringBuilder::with_capacity(len, len * 16);
    let mut chat_id = Int64Builder::with_capacity(len);
    let mut chat_members_handles = StringBuilder::with_capacity(len, len * 32);
    let mut chat_members_contact_info = StringBuilder::with_capacity(len, len * 32);
    let mut chat_size = Int64Builder::with_capacity(len);
    let mut is_audio_message = Int8Builder::with_capacity(len);
    let mut message_effect = StringBuilder::with_capacity(len, len * 8);
    let mut reaction = StringBuilder::with_capacity(len, len * 8);
    let mut is_thread_reply = Int8Builder::with_capacity(len);
    let mut link_domain = StringBuilder::with_capacity(len, len * 16);
    let mut col_name = StringBuilder::with_capacity(len, len * 16);
    let mut timestamp =
        TimestampSecondBuilder::with_capacity(len).with_timezone(Arc::from("UTC"));
    let mut date = Date32Builder::with_capacity(len);
    let mut month = Int8Builder::with_capacity(len);
    let mut year = Int16Builder::with_capacity(len);

    for row in rows {
        message_id.append_value(row.message_id);
        is_from_me.append_value(row.is_from_me as i8);
        is_audio_message.append_value(row.is_audio_message as i8);
        handle_id.append_value(row.handle_id);

        // Text fields
        let native_text = row.text.as_deref();
        let inf = row
            .attributed_body
            .as_deref()
            .and_then(blob_parser::parse);

        let combined = native_text.map(str::to_string).or_else(|| inf.clone());

        append_opt_str(&mut col_text, native_text);
        append_opt_str(&mut inferred_text, inf.as_deref());
        append_opt_str(&mut text_combined, combined.as_deref());

        // Contact info
        append_opt_str(&mut contact_info, row.contact_info.as_deref());

        // Chat membership
        let membership = row.chat_id.and_then(|cid| chat_members.get(&cid));
        let size = membership.map(|m| m.handle_ids.len() as i64).unwrap_or(0);
        chat_size.append_value(size);

        if let Some(m) = membership {
            let handles_json = format!(
                "[{}]",
                m.handle_ids
                    .iter()
                    .map(|h| h.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            let contacts_json = serde_json::to_string(&m.contact_infos)
                .unwrap_or_else(|_| "[]".to_string());
            chat_members_handles.append_value(handles_json);
            chat_members_contact_info.append_value(contacts_json);
        } else {
            chat_members_handles.append_null();
            chat_members_contact_info.append_null();
        }

        // Chat ID
        match row.chat_id {
            Some(cid) => chat_id.append_value(cid),
            None => chat_id.append_null(),
        }

        // updated_contact_info: for sent messages infer the recipient
        let uci = if row.is_from_me == 1 {
            if size == 1 {
                membership
                    .and_then(|m| m.contact_infos.first())
                    .map(|s| s.as_str())
            } else if size > 1 {
                Some("group-chat")
            } else {
                None
            }
        } else {
            row.contact_info.as_deref()
        };
        append_opt_str(&mut updated_contact_info, uci);

        // Derived feature columns
        let effect = row
            .expressive_send_style_id
            .as_deref()
            .map(detect_message_effect)
            .unwrap_or_else(|| "no-effect".to_string());
        message_effect.append_value(effect);

        reaction.append_value(detect_reaction(row.associated_message_type));

        is_thread_reply.append_value(row.thread_originator_guid.is_some() as i8);

        let domain = if row.balloon_bundle_id.is_some() {
            combined.as_deref().and_then(extract_link_domain)
        } else {
            None
        };
        append_opt_str(&mut link_domain, domain.as_deref());

        // Contact name resolution
        let resolved_name = row
            .contact_info
            .as_deref()
            .and_then(|ci| contacts.get(ci).map(|s| s.as_str()))
            .or(row.contact_info.as_deref());
        append_opt_str(&mut col_name, resolved_name);

        // Timestamp and date parts
        if row.unix_ts > 0 {
            if let Some(dt) = Utc.timestamp_opt(row.unix_ts, 0).single() {
                timestamp.append_value(row.unix_ts);
                let days = (dt.date_naive() - chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
                    .num_days() as i32;
                date.append_value(days);
                month.append_value(dt.month() as i8);
                year.append_value(dt.year() as i16);
            } else {
                timestamp.append_null();
                date.append_null();
                month.append_null();
                year.append_null();
            }
        } else {
            timestamp.append_null();
            date.append_null();
            month.append_null();
            year.append_null();
        }
    }

    let batch = RecordBatch::try_new(
        Arc::new(schema()),
        vec![
            Arc::new(message_id.finish()),
            Arc::new(is_from_me.finish()),
            Arc::new(col_text.finish()),
            Arc::new(inferred_text.finish()),
            Arc::new(text_combined.finish()),
            Arc::new(handle_id.finish()),
            Arc::new(contact_info.finish()),
            Arc::new(updated_contact_info.finish()),
            Arc::new(chat_id.finish()),
            Arc::new(chat_members_handles.finish()),
            Arc::new(chat_members_contact_info.finish()),
            Arc::new(chat_size.finish()),
            Arc::new(is_audio_message.finish()),
            Arc::new(message_effect.finish()),
            Arc::new(reaction.finish()),
            Arc::new(is_thread_reply.finish()),
            Arc::new(link_domain.finish()),
            Arc::new(col_name.finish()),
            Arc::new(timestamp.finish()),
            Arc::new(date.finish()),
            Arc::new(month.finish()),
            Arc::new(year.finish()),
        ],
    )?;

    Ok(batch)
}

fn append_opt_str(builder: &mut StringBuilder, val: Option<&str>) {
    match val {
        Some(s) => builder.append_value(s),
        None => builder.append_null(),
    }
}
